use futures::future::{join_all, poll_fn};
use futures::task::{Context, Poll};
use std::collections::HashMap;
use std::convert::Infallible;
use std::future::Future;
use std::hash::Hasher;
use std::net::SocketAddr;
use std::pin::Pin;
use std::process::Command;
use std::str;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use byteorder::{ByteOrder, LittleEndian};
use dashmap::DashMap;
use hyper::header::{HeaderName, HeaderValue};
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server, StatusCode};
use num_cpus;
use once_cell::sync::Lazy;
use rand::RngCore;
use rand::rngs::OsRng;
use regex::Regex;
use tokio::sync::RwLock;
use tokio::task;
use tokio::time::delay_for;
use twox_hash::XxHash64;

use crate::vm::event::{BuiltInEvents, EventEmit, HandlerFragment};
use crate::vm::memory::{CLOSURE_ARG_MEM_START, HandlerMemory};
use crate::vm::run::{EVENT_TX};

static HTTP_RESPONSES: Lazy<Arc<Mutex<HashMap<i64, HandlerMemory>>>> = Lazy::new(|| {
  Arc::new(Mutex::new(HashMap::<i64, HandlerMemory>::new()))
});

static DS: Lazy<Arc<DashMap<String, HandlerMemory>>> = Lazy::new(|| {
  Arc::new(DashMap::<String, HandlerMemory>::new())
});

// type aliases
/// Futures implement an Unpin marker that guarantees to the compiler that the future will not move while it is running
/// so it can be polled. If it is moved, the implementation would be unsafe. We have to manually pin the future because
/// we are creating it dynamically. We must also specify that the `Box`ed Future can be moved across threads with a `+ Send`.
/// For more information see:
/// https://stackoverflow.com/questions/58354633/cannot-use-impl-future-to-store-async-function-in-a-vector
/// https://stackoverflow.com/questions/51485410/unable-to-tokiorun-a-boxed-future-because-the-trait-bound-send-is-not-satisfie
pub type HMFuture = Pin<Box<dyn Future<Output = HandlerMemory> + Send>>;
/// Function pointer for io bound opcodes
type AsyncFnPtr = fn(
  Vec<i64>,
  HandlerMemory,
) -> HMFuture;
/// Function pointer for cpu bound opcodes
type FnPtr = fn(
  &[i64],
  &mut HandlerMemory,
) -> Option<EventEmit>;

/// To allow concise definition of opcodes we have a struct that stores all the information
/// about an opcode and how to run it.
/// To define CPU-bound opcodes we use a function pointer type which describes a function whose identity
/// is not necessarily known at compile-time. A closure without context is a function pointer since it can run anywhere.
/// To define IO-bound opcodes it is trickier because `async` fns returns an opaque `impl Future` type so we have to jump through some Rust hoops
/// to be able to define this behaviour
/// For more information see:
/// https://stackoverflow.com/questions/27831944/how-do-i-store-a-closure-in-a-struct-in-rust
/// https://stackoverflow.com/questions/59035366/how-do-i-store-a-variable-of-type-impl-trait-in-a-struct
pub struct ByteOpcode {
  /// Opcode value as an i64 number
  pub(crate) _id: i64,
  /// Human readable name for id
  pub(crate) _name: String,
  /// Boolean that is true if this opcode has predictable execution
  pub(crate) pred_exec: bool,
  /// void function pointer that describes the side-effect of cpu bound opcode
  pub(crate) func: Option<FnPtr>,
  /// void async function pointer that describes the side-effect of io bound opcode
  pub(crate) async_func: Option<AsyncFnPtr>,
}

pub fn opcode_id(name: &str) -> i64 {
  let mut name_vec = vec!(b' ', b' ', b' ', b' ', b' ', b' ', b' ', b' ');
  // Now insert the new name characters
  for (i, c) in name.chars().enumerate() {
    std::mem::replace(&mut name_vec[i], c as u8);
  }
  let id = LittleEndian::read_i64(&name_vec);
  return id;
}

pub static OPCODES: Lazy<HashMap<i64, ByteOpcode>> = Lazy::new(|| {
  let mut o = HashMap::new();

  macro_rules! io {
    ($name:expr, $async_func:expr) => {
      let id = opcode_id($name);
      let opcode = ByteOpcode {
        _id: id,
        _name: $name.to_string(),
        pred_exec: false,
        func: None,
        async_func: Some($async_func),
      };
      o.insert(id, opcode);
    };
  }

  macro_rules! cpu {
    ($name:expr, $func:expr) => {
      let id = opcode_id($name);
      let opcode = ByteOpcode {
        _id: id,
        _name: $name.to_string(),
        pred_exec: true,
        func: Some($func),
        async_func: None,
      };
      o.insert(id, opcode);
    };
  }

  // Type conversion opcodes
  cpu!("i8f64", |args, hand_mem| {
    let out = hand_mem.read_fixed(args[0]) as f64;
    hand_mem.write_fixed(args[2], i64::from_ne_bytes(out.to_ne_bytes()));
    None
  });
  cpu!("i16f64", |args, hand_mem| {
    let out = hand_mem.read_fixed(args[0]) as f64;
    hand_mem.write_fixed(args[2], i64::from_ne_bytes(out.to_ne_bytes()));
    None
  });
  cpu!("i32f64", |args, hand_mem| {
    let out = hand_mem.read_fixed(args[0]) as f64;
    hand_mem.write_fixed(args[2], i64::from_ne_bytes(out.to_ne_bytes()));
    None
  });
  cpu!("i64f64", |args, hand_mem| {
    let out = hand_mem.read_fixed(args[0]) as f64;
    hand_mem.write_fixed(args[2], i64::from_ne_bytes(out.to_ne_bytes()));
    None
  });
  cpu!("f32f64", |args, hand_mem| {
    let out = f32::from_ne_bytes((hand_mem.read_fixed(args[0]) as i32).to_ne_bytes());
    hand_mem.write_fixed(args[2], i32::from_ne_bytes(out.to_ne_bytes()) as i64);
    None
  });
  cpu!("strf64", |args, hand_mem| {
    let s = HandlerMemory::fractal_to_string(hand_mem.read_fractal(args[0]));
    let out: f64 = s.parse().unwrap();
    hand_mem.write_fixed(args[2], i64::from_ne_bytes(out.to_ne_bytes()));
    None
  });
  cpu!("boolf64", |args, hand_mem| {
    let out = hand_mem.read_fixed(args[0]) as f64;
    hand_mem.write_fixed(args[2], i64::from_ne_bytes(out.to_ne_bytes()));
    None
  });

  cpu!("i8f32", |args, hand_mem| {
    let num = hand_mem.read_fixed(args[0]) as f32;
    let out = i32::from_ne_bytes(num.to_ne_bytes()) as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("i16f32", |args, hand_mem| {
    let num = hand_mem.read_fixed(args[0]) as f32;
    let out = i32::from_ne_bytes(num.to_ne_bytes()) as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("i32f32", |args, hand_mem| {
    let num = hand_mem.read_fixed(args[0]) as f32;
    let out = i32::from_ne_bytes(num.to_ne_bytes()) as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("i64f32", |args, hand_mem| {
    let num = hand_mem.read_fixed(args[0]) as f32;
    let out = i32::from_ne_bytes(num.to_ne_bytes()) as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("f64f32", |args, hand_mem| {
    let num = f64::from_ne_bytes(hand_mem.read_fixed(args[0]).to_ne_bytes()) as f32;
    let out = i32::from_ne_bytes(num.to_ne_bytes()) as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("strf32", |args, hand_mem| {
    let s = HandlerMemory::fractal_to_string(hand_mem.read_fractal(args[0]));
    let num: f32 = s.parse().unwrap();
    let out = i32::from_ne_bytes(num.to_ne_bytes()) as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("boolf32", |args, hand_mem| {
    let num = hand_mem.read_fixed(args[0]) as f32;
    let out = i32::from_ne_bytes(num.to_ne_bytes()) as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });

  cpu!("i8i64", |args, hand_mem| {
    let out = hand_mem.read_fixed(args[0]);
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("i16i64", |args, hand_mem| {
    let out = hand_mem.read_fixed(args[0]);
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("i32i64", |args, hand_mem| {
    let out = hand_mem.read_fixed(args[0]);
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("f64i64", |args, hand_mem| {
    let out = f64::from_ne_bytes(hand_mem.read_fixed(args[0]).to_ne_bytes()) as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("f32i64", |args, hand_mem| {
    let out = f32::from_ne_bytes((hand_mem.read_fixed(args[0]) as i32).to_ne_bytes()) as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("stri64", |args, hand_mem| {
    let s = HandlerMemory::fractal_to_string(hand_mem.read_fractal(args[0]));
    let out: i64 = s.parse().unwrap();
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("booli64", |args, hand_mem| {
    let out = hand_mem.read_fixed(args[0]);
    hand_mem.write_fixed(args[2], out);
    None
  });

  cpu!("i8i32", |args, hand_mem| {
    let out = hand_mem.read_fixed(args[0]);
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("i16i32", |args, hand_mem| {
    let out = hand_mem.read_fixed(args[0]);
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("i64i32", |args, hand_mem| {
    let out = (hand_mem.read_fixed(args[0]) as i32) as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("f64i32", |args, hand_mem| {
    let out = (f64::from_ne_bytes(hand_mem.read_fixed(args[0]).to_ne_bytes()) as i32) as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("f32i32", |args, hand_mem| {
    let out = (f32::from_ne_bytes((hand_mem.read_fixed(args[0]) as i32).to_ne_bytes()) as i32) as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("stri32", |args, hand_mem| {
    let s = HandlerMemory::fractal_to_string(hand_mem.read_fractal(args[0]));
    let num: i32 = s.parse().unwrap();
    let out = num as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("booli32", |args, hand_mem| {
    let out = hand_mem.read_fixed(args[0]);
    hand_mem.write_fixed(args[2], out);
    None
  });

  cpu!("i8i16", |args, hand_mem| {
    let out = hand_mem.read_fixed(args[0]);
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("i32i16", |args, hand_mem| {
    let out = (hand_mem.read_fixed(args[0]) as i16) as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("i64i16", |args, hand_mem| {
    let out = (hand_mem.read_fixed(args[0]) as i16) as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("f64i16", |args, hand_mem| {
    let out = (f64::from_ne_bytes(hand_mem.read_fixed(args[0]).to_ne_bytes()) as i16) as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("f32i16", |args, hand_mem| {
    let out = (f32::from_ne_bytes((hand_mem.read_fixed(args[0]) as i32).to_ne_bytes()) as i16) as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("stri16", |args, hand_mem| {
    let s = HandlerMemory::fractal_to_string(hand_mem.read_fractal(args[0]));
    let num: i16 = s.parse().unwrap();
    let out = num as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("booli16", |args, hand_mem| {
    let out = hand_mem.read_fixed(args[0]);
    hand_mem.write_fixed(args[2], out);
    None
  });

  cpu!("i16i8", |args, hand_mem| {
    let out = (hand_mem.read_fixed(args[0]) as i8) as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("i32i8", |args, hand_mem| {
    let out = (hand_mem.read_fixed(args[0]) as i8) as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("i64i8", |args, hand_mem| {
    let out = (hand_mem.read_fixed(args[0]) as i8) as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("f64i8", |args, hand_mem| {
    let out = (f64::from_ne_bytes(hand_mem.read_fixed(args[0]).to_ne_bytes()) as i8) as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("f32i8", |args, hand_mem| {
    let out = (f32::from_ne_bytes((hand_mem.read_fixed(args[0]) as i32).to_ne_bytes()) as i8) as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("stri8", |args, hand_mem| {
    let s = HandlerMemory::fractal_to_string(hand_mem.read_fractal(args[0]));
    let num: i8 = s.parse().unwrap();
    let out = num as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("booli8", |args, hand_mem| {
    let out = (hand_mem.read_fixed(args[0]) as i8) as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });

  cpu!("i8bool", |args, hand_mem| {
    let a = hand_mem.read_fixed(args[0]) as i8;
    let out = if a != 0 { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("i16bool", |args, hand_mem| {
    let a = hand_mem.read_fixed(args[0]) as i16;
    let out = if a != 0 { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("i32bool", |args, hand_mem| {
    let a = hand_mem.read_fixed(args[0]) as i32;
    let out = if a != 0 { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("i64bool", |args, hand_mem| {
    let a = hand_mem.read_fixed(args[0]);
    let out = if a != 0 { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("f64bool", |args, hand_mem| {
    let a = f64::from_ne_bytes(hand_mem.read_fixed(args[0]).to_ne_bytes());
    let out = if a != 0.0 { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("f32bool", |args, hand_mem| {
    let a = f32::from_ne_bytes((hand_mem.read_fixed(args[0]) as i32).to_ne_bytes());
    let out = if a != 0.0 { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("strbool", |args, hand_mem| {
    let s = HandlerMemory::fractal_to_string(hand_mem.read_fractal(args[0]));
    let out = if s == "true" { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out);
    None
  });

  cpu!("i8str", |args, hand_mem| {
    let a = hand_mem.read_fixed(args[0]) as i8;
    let a_str = a.to_string();
    hand_mem.write_fractal(args[2], &HandlerMemory::str_to_fractal(&a_str));
    None
  });
  cpu!("i16str", |args, hand_mem| {
    let a = hand_mem.read_fixed(args[0]) as i16;
    let a_str = a.to_string();
    hand_mem.write_fractal(args[2], &HandlerMemory::str_to_fractal(&a_str));
    None
  });
  cpu!("i32str", |args, hand_mem| {
    let a = hand_mem.read_fixed(args[0]) as i32;
    let a_str = a.to_string();
    hand_mem.write_fractal(args[2], &HandlerMemory::str_to_fractal(&a_str));
    None
  });
  cpu!("i64str", |args, hand_mem| {
    let a = hand_mem.read_fixed(args[0]);
    let a_str = a.to_string();
    hand_mem.write_fractal(args[2], &HandlerMemory::str_to_fractal(&a_str));
    None
  });
  cpu!("f64str", |args, hand_mem| {
    let a = f64::from_ne_bytes(hand_mem.read_fixed(args[0]).to_ne_bytes());
    let a_str = a.to_string();
    hand_mem.write_fractal(args[2], &HandlerMemory::str_to_fractal(&a_str));
    None
  });
  cpu!("f32str", |args, hand_mem| {
    let a = f32::from_ne_bytes((hand_mem.read_fixed(args[0]) as i32).to_ne_bytes());
    let a_str = a.to_string();
    hand_mem.write_fractal(args[2], &HandlerMemory::str_to_fractal(&a_str));
    None
  });
  cpu!("boolstr", |args, hand_mem| {
    let a = hand_mem.read_fixed(args[0]);
    let a_str = if a == 1 { "true" } else { "false" };
    hand_mem.write_fractal(args[2], &HandlerMemory::str_to_fractal(&a_str));
    None
  });

  // Arithmetic opcodes
  cpu!("addi8", |args, hand_mem| {
    let a = hand_mem.read_fixed(args[0]) as i8;
    let b = hand_mem.read_fixed(args[1]) as i8;
    let out = (a + b) as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("addi16", |args, hand_mem| {
    let a = hand_mem.read_fixed(args[0]) as i16;
    let b = hand_mem.read_fixed(args[1]) as i16;
    let out = (a + b) as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("addi32", |args, hand_mem| {
    let a = hand_mem.read_fixed(args[0]) as i32;
    let b = hand_mem.read_fixed(args[1]) as i32;
    let out = (a + b) as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("addi64", |args, hand_mem| {
    let a = hand_mem.read_fixed(args[0]);
    let b = hand_mem.read_fixed(args[1]);
    let out = a + b;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("addf32", |args, hand_mem| {
    let a = f32::from_ne_bytes((hand_mem.read_fixed(args[0]) as i32).to_ne_bytes());
    let b = f32::from_ne_bytes((hand_mem.read_fixed(args[1]) as i32).to_ne_bytes());
    let out = i32::from_ne_bytes((a + b).to_ne_bytes()) as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("addf64", |args, hand_mem| {
    let a = f64::from_ne_bytes(hand_mem.read_fixed(args[0]).to_ne_bytes());
    let b = f64::from_ne_bytes(hand_mem.read_fixed(args[1]).to_ne_bytes());
    let out = i64::from_ne_bytes((a + b).to_ne_bytes());
    hand_mem.write_fixed(args[2], out);
    None
  });

  cpu!("subi8", |args, hand_mem| {
    let a = hand_mem.read_fixed(args[0]) as i8;
    let b = hand_mem.read_fixed(args[1]) as i8;
    let out = (a - b) as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("subi16", |args, hand_mem| {
    let a = hand_mem.read_fixed(args[0]) as i16;
    let b = hand_mem.read_fixed(args[1]) as i16;
    let out = (a - b) as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("subi32", |args, hand_mem| {
    let a = hand_mem.read_fixed(args[0]) as i32;
    let b = hand_mem.read_fixed(args[1]) as i32;
    let out = (a - b) as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("subi64", |args, hand_mem| {
    let a = hand_mem.read_fixed(args[0]);
    let b = hand_mem.read_fixed(args[1]);
    let out = a - b;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("subf32", |args, hand_mem| {
    let a = f32::from_ne_bytes((hand_mem.read_fixed(args[0]) as i32).to_ne_bytes());
    let b = f32::from_ne_bytes((hand_mem.read_fixed(args[1]) as i32).to_ne_bytes());
    let out = i32::from_ne_bytes((a - b).to_ne_bytes()) as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("subf64", |args, hand_mem| {
    let a = f64::from_ne_bytes(hand_mem.read_fixed(args[0]).to_ne_bytes());
    let b = f64::from_ne_bytes(hand_mem.read_fixed(args[1]).to_ne_bytes());
    let out = i64::from_ne_bytes((a - b).to_ne_bytes());
    hand_mem.write_fixed(args[2], out);
    None
  });

  cpu!("negi8", |args, hand_mem| {
    let a = hand_mem.read_fixed(args[0]) as i8;
    let out = (0 - a) as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("negi16", |args, hand_mem| {
    let a = hand_mem.read_fixed(args[0]) as i16;
    let out = (0 - a) as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("negi32", |args, hand_mem| {
    let a = hand_mem.read_fixed(args[0]) as i32;
    let out = (0 - a) as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("negi64", |args, hand_mem| {
    let a = hand_mem.read_fixed(args[0]);
    let out = 0 - a;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("negf32", |args, hand_mem| {
    let a = f32::from_ne_bytes((hand_mem.read_fixed(args[0]) as i32).to_ne_bytes());
    let out = i32::from_ne_bytes((0.0 - a).to_ne_bytes()) as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("negf64", |args, hand_mem| {
    let a = f64::from_ne_bytes(hand_mem.read_fixed(args[0]).to_ne_bytes());
    let out = i64::from_ne_bytes((0.0 - a).to_ne_bytes());
    hand_mem.write_fixed(args[2], out);
    None
  });

  cpu!("absi8", |args, hand_mem| {
    let a = hand_mem.read_fixed(args[0]) as i8;
    let out = a.abs() as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("absi16", |args, hand_mem| {
    let a = hand_mem.read_fixed(args[0]) as i16;
    let out = a.abs() as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("absi32", |args, hand_mem| {
    let a = hand_mem.read_fixed(args[0]) as i32;
    let out = a.abs() as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("absi64", |args, hand_mem| {
    let a = hand_mem.read_fixed(args[0]);
    let out = a.abs();
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("absf32", |args, hand_mem| {
    let a = f32::from_ne_bytes((hand_mem.read_fixed(args[0]) as i32).to_ne_bytes());
    let out = i32::from_ne_bytes(a.abs().to_ne_bytes()) as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("absf64", |args, hand_mem| {
    let a = f64::from_ne_bytes(hand_mem.read_fixed(args[0]).to_ne_bytes());
    let out = i64::from_ne_bytes(a.abs().to_ne_bytes());
    hand_mem.write_fixed(args[2], out);
    None
  });

  cpu!("muli8", |args, hand_mem| {
    let a = hand_mem.read_fixed(args[0]) as i8;
    let b = hand_mem.read_fixed(args[1]) as i8;
    let out = (a * b) as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("muli16", |args, hand_mem| {
    let a = hand_mem.read_fixed(args[0]) as i16;
    let b = hand_mem.read_fixed(args[1]) as i16;
    let out = (a * b) as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("muli32", |args, hand_mem| {
    let a = hand_mem.read_fixed(args[0]) as i32;
    let b = hand_mem.read_fixed(args[1]) as i32;
    let out = (a * b) as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("muli64", |args, hand_mem| {
    let a = hand_mem.read_fixed(args[0]);
    let b = hand_mem.read_fixed(args[1]);
    let out = a * b;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("mulf32", |args, hand_mem| {
    let a = f32::from_ne_bytes((hand_mem.read_fixed(args[0]) as i32).to_ne_bytes());
    let b = f32::from_ne_bytes((hand_mem.read_fixed(args[1]) as i32).to_ne_bytes());
    let out = i32::from_ne_bytes((a * b).to_ne_bytes()) as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("mulf64", |args, hand_mem| {
    let a = f64::from_ne_bytes(hand_mem.read_fixed(args[0]).to_ne_bytes());
    let b = f64::from_ne_bytes(hand_mem.read_fixed(args[1]).to_ne_bytes());
    let out = i64::from_ne_bytes((a * b).to_ne_bytes());
    hand_mem.write_fixed(args[2], out);
    None
  });

  cpu!("divi8", |args, hand_mem| {
    let a = hand_mem.read_fixed(args[0]) as i8;
    let b = hand_mem.read_fixed(args[1]) as i8;
    let out = (a / b) as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("divi16", |args, hand_mem| {
    let a = hand_mem.read_fixed(args[0]) as i16;
    let b = hand_mem.read_fixed(args[1]) as i16;
    let out = (a / b) as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("divi32", |args, hand_mem| {
    let a = hand_mem.read_fixed(args[0]) as i32;
    let b = hand_mem.read_fixed(args[1]) as i32;
    let out = (a / b) as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("divi64", |args, hand_mem| {
    let a = hand_mem.read_fixed(args[0]);
    let b = hand_mem.read_fixed(args[1]);
    let out = a / b;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("divf32", |args, hand_mem| {
    let a = f32::from_ne_bytes((hand_mem.read_fixed(args[0]) as i32).to_ne_bytes());
    let b = f32::from_ne_bytes((hand_mem.read_fixed(args[1]) as i32).to_ne_bytes());
    let out = i32::from_ne_bytes((a / b).to_ne_bytes()) as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("divf64", |args, hand_mem| {
    let a = f64::from_ne_bytes(hand_mem.read_fixed(args[0]).to_ne_bytes());
    let b = f64::from_ne_bytes(hand_mem.read_fixed(args[1]).to_ne_bytes());
    let out = i64::from_ne_bytes((a / b).to_ne_bytes());
    hand_mem.write_fixed(args[2], out);
    None
  });

  cpu!("modi8", |args, hand_mem| {
    let a = hand_mem.read_fixed(args[0]) as i8;
    let b = hand_mem.read_fixed(args[1]) as i8;
    let out = (a % b) as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("modi16", |args, hand_mem| {
    let a = hand_mem.read_fixed(args[0]) as i16;
    let b = hand_mem.read_fixed(args[1]) as i16;
    let out = (a % b) as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("modi32", |args, hand_mem| {
    let a = hand_mem.read_fixed(args[0]) as i32;
    let b = hand_mem.read_fixed(args[1]) as i32;
    let out = (a % b) as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("modi64", |args, hand_mem| {
    let a = hand_mem.read_fixed(args[0]);
    let b = hand_mem.read_fixed(args[1]);
    let out = a % b;
    hand_mem.write_fixed(args[2], out);
    None
  });

  cpu!("powi8", |args, hand_mem| {
    let a = hand_mem.read_fixed(args[0]) as i8;
    let b = hand_mem.read_fixed(args[1]) as i8;
    let out = if b < 0 { 0i64 } else { i8::pow(a, b as u32) as i64 };
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("powi16", |args, hand_mem| {
    let a = hand_mem.read_fixed(args[0]) as i16;
    let b = hand_mem.read_fixed(args[1]) as i16;
    let out = if b < 0 { 0i64 } else { i16::pow(a, b as u32) as i64 };
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("powi32", |args, hand_mem| {
    let a = hand_mem.read_fixed(args[0]) as i32;
    let b = hand_mem.read_fixed(args[1]) as i32;
    let out = if b < 0 { 0i64 } else { i32::pow(a, b as u32) as i64 };
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("powi64", |args, hand_mem| {
    // The inputs may be from local memory or global
    let a = hand_mem.read_fixed(args[0]);
    let b = hand_mem.read_fixed(args[1]);
    // Rust's pow implementation is i64 to the u32 power, which is close to what we want to do. If
    // the exponent is gigantic if would overflow the integer, so this is likely not desired, and if
    // the exponent is negative, is will be between 0 to 1 and basically always be zero for integer
    // calculations, unless the original number is 1. So we're gonna cover all of those branches.
    let out: i64 = if a == 0 {
      0
    } else if a == 1 {
      1
    } else if b > std::u32::MAX as i64 {
      std::i64::MAX
    } else {
      let bu32 = b as u32;
      i64::pow(a, bu32)
    };
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("powf32", |args, hand_mem| {
    let a = f32::from_ne_bytes((hand_mem.read_fixed(args[0]) as i32).to_ne_bytes());
    let b = f32::from_ne_bytes((hand_mem.read_fixed(args[1]) as i32).to_ne_bytes());
    let out = i32::from_ne_bytes(f32::powf(a, b).to_ne_bytes()) as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("powf64", |args, hand_mem| {
    let a = f64::from_ne_bytes(hand_mem.read_fixed(args[0]).to_ne_bytes());
    let b = f64::from_ne_bytes(hand_mem.read_fixed(args[1]).to_ne_bytes());
    let out = i64::from_ne_bytes(f64::powf(a, b).to_ne_bytes());
    hand_mem.write_fixed(args[2], out);
    None
  });

  cpu!("sqrtf32", |args, hand_mem| {
    let a = f32::from_ne_bytes((hand_mem.read_fixed(args[0]) as i32).to_ne_bytes());
    let out = i32::from_ne_bytes(f32::sqrt(a).to_ne_bytes()) as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("sqrtf64", |args, hand_mem| {
    let a = f64::from_ne_bytes(hand_mem.read_fixed(args[0]).to_ne_bytes());
    let out = i64::from_ne_bytes(f64::sqrt(a).to_ne_bytes());
    hand_mem.write_fixed(args[2], out);
    None
  });

  // Boolean and bitwise opcodes
  cpu!("andi8", |args, hand_mem| {
    let a = hand_mem.read_fixed(args[0]) as i8;
    let b = hand_mem.read_fixed(args[1]) as i8;
    let out = (a & b) as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("andi16", |args, hand_mem| {
    let a = hand_mem.read_fixed(args[0]) as i16;
    let b = hand_mem.read_fixed(args[1]) as i16;
    let out = (a & b) as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("andi32", |args, hand_mem| {
    let a = hand_mem.read_fixed(args[0]) as i32;
    let b = hand_mem.read_fixed(args[1]) as i32;
    let out = (a & b) as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("andi64", |args, hand_mem| {
    let a = hand_mem.read_fixed(args[0]);
    let b = hand_mem.read_fixed(args[1]);
    let out = a & b;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("andbool", |args, hand_mem| {
    let a = hand_mem.read_fixed(args[0]);
    let b = hand_mem.read_fixed(args[1]);
    let a_bool = if a == 1 { true } else { false };
    let b_bool = if b == 1 { true } else { false };
    let out = if a_bool & b_bool { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out);
    None
  });

  cpu!("ori8", |args, hand_mem| {
    let a = hand_mem.read_fixed(args[0]) as i8;
    let b = hand_mem.read_fixed(args[1]) as i8;
    let out = (a | b) as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("ori16", |args, hand_mem| {
    let a = hand_mem.read_fixed(args[0]) as i16;
    let b = hand_mem.read_fixed(args[1]) as i16;
    let out = (a | b) as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("ori32", |args, hand_mem| {
    let a = hand_mem.read_fixed(args[0]) as i32;
    let b = hand_mem.read_fixed(args[1]) as i32;
    let out = (a | b) as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("ori64", |args, hand_mem| {
    let a = hand_mem.read_fixed(args[0]);
    let b = hand_mem.read_fixed(args[1]);
    let out = a | b;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("orbool", |args, hand_mem| {
    let a = hand_mem.read_fixed(args[0]);
    let b = hand_mem.read_fixed(args[1]);
    let a_bool = if a == 1 { true } else { false };
    let b_bool = if b == 1 { true } else { false };
    let out = if a_bool | b_bool { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out);
    None
  });

  cpu!("xori8", |args, hand_mem| {
    let a = hand_mem.read_fixed(args[0]) as i8;
    let b = hand_mem.read_fixed(args[1]) as i8;
    let out = (a ^ b) as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("xori16", |args, hand_mem| {
    let a = hand_mem.read_fixed(args[0]) as i16;
    let b = hand_mem.read_fixed(args[1]) as i16;
    let out = (a ^ b) as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("xori32", |args, hand_mem| {
    let a = hand_mem.read_fixed(args[0]) as i32;
    let b = hand_mem.read_fixed(args[1]) as i32;
    let out = (a ^ b) as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("xori64", |args, hand_mem| {
    let a = hand_mem.read_fixed(args[0]);
    let b = hand_mem.read_fixed(args[1]);
    let out = a ^ b;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("xorbool", |args, hand_mem| {
    let a = hand_mem.read_fixed(args[0]);
    let b = hand_mem.read_fixed(args[1]);
    let a_bool = if a == 1 { true } else { false };
    let b_bool = if b == 1 { true } else { false };
    let out = if a_bool ^ b_bool { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out);
    None
  });

  cpu!("noti8", |args, hand_mem| {
    let a = hand_mem.read_fixed(args[0]) as i8;
    let out = !a as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("noti16", |args, hand_mem| {
    let a = hand_mem.read_fixed(args[0]) as i16;
    let out = !a as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("noti32", |args, hand_mem| {
    let a = hand_mem.read_fixed(args[0]) as i32;
    let out = !a as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("noti64", |args, hand_mem| {
    let a = hand_mem.read_fixed(args[0]);
    let out = !a;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("notbool", |args, hand_mem| {
    let a = hand_mem.read_fixed(args[0]);
    let a_bool = if a == 1 { true } else { false };
    let out = if !a_bool { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out);
    None
  });

  cpu!("nandi8", |args, hand_mem| {
    let a = hand_mem.read_fixed(args[0]) as i8;
    let b = hand_mem.read_fixed(args[1]) as i8;
    let out = !(a & b) as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("nandi16", |args, hand_mem| {
    let a = hand_mem.read_fixed(args[0]) as i16;
    let b = hand_mem.read_fixed(args[1]) as i16;
    let out = !(a & b) as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("nandi32", |args, hand_mem| {
    let a = hand_mem.read_fixed(args[0]) as i32;
    let b = hand_mem.read_fixed(args[1]) as i32;
    let out = !(a & b) as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("nandi64", |args, hand_mem| {
    let a = hand_mem.read_fixed(args[0]);
    let b = hand_mem.read_fixed(args[1]);
    let out = !(a & b);
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("nandboo", |args, hand_mem| {
    let a = hand_mem.read_fixed(args[0]);
    let b = hand_mem.read_fixed(args[1]);
    let a_bool = if a == 1 { true } else { false };
    let b_bool = if b == 1 { true } else { false };
    let out = if !(a_bool & b_bool) { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out);
    None
  });

  cpu!("nori8", |args, hand_mem| {
    let a = hand_mem.read_fixed(args[0]) as i8;
    let b = hand_mem.read_fixed(args[1]) as i8;
    let out = !(a | b) as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("nori16", |args, hand_mem| {
    let a = hand_mem.read_fixed(args[0]) as i16;
    let b = hand_mem.read_fixed(args[1]) as i16;
    let out = !(a | b) as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("nori32", |args, hand_mem| {
    let a = hand_mem.read_fixed(args[0]) as i32;
    let b = hand_mem.read_fixed(args[1]) as i32;
    let out = !(a | b) as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("nori64", |args, hand_mem| {
    let a = hand_mem.read_fixed(args[0]);
    let b = hand_mem.read_fixed(args[1]);
    let out = !(a | b);
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("norbool", |args, hand_mem| {
    let a = hand_mem.read_fixed(args[0]);
    let b = hand_mem.read_fixed(args[1]);
    let a_bool = if a == 1 { true } else { false };
    let b_bool = if b == 1 { true } else { false };
    let out = if !(a_bool | b_bool) { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out);
    None
  });

  cpu!("xnori8", |args, hand_mem| {
    let a = hand_mem.read_fixed(args[0]) as i8;
    let b = hand_mem.read_fixed(args[1]) as i8;
    let out = !(a ^ b) as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("xnori16", |args, hand_mem| {
    let a = hand_mem.read_fixed(args[0]) as i16;
    let b = hand_mem.read_fixed(args[1]) as i16;
    let out = !(a ^ b) as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("xnori32", |args, hand_mem| {
    let a = hand_mem.read_fixed(args[0]) as i32;
    let b = hand_mem.read_fixed(args[1]) as i32;
    let out = !(a ^ b) as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("xnori64", |args, hand_mem| {
    let a = hand_mem.read_fixed(args[0]);
    let b = hand_mem.read_fixed(args[1]);
    let out = !(a ^ b);
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("xnorboo", |args, hand_mem| {
    let a = hand_mem.read_fixed(args[0]);
    let b = hand_mem.read_fixed(args[1]);
    let a_bool = if a == 1 { true } else { false };
    let b_bool = if b == 1 { true } else { false };
    let out = if !(a_bool ^ b_bool) { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out);
    None
  });

  // Equality and order opcodes
  cpu!("eqi8", |args, hand_mem| {
    let a = hand_mem.read_fixed(args[0]) as i8;
    let b = hand_mem.read_fixed(args[1]) as i8;
    let out = if a == b { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("eqi16", |args, hand_mem| {
    let a = hand_mem.read_fixed(args[0]) as i16;
    let b = hand_mem.read_fixed(args[1]) as i16;
    let out = if a == b { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("eqi32", |args, hand_mem| {
    let a = hand_mem.read_fixed(args[0]) as i32;
    let b = hand_mem.read_fixed(args[1]) as i32;
    let out = if a == b { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("eqi64", |args, hand_mem| {
    let a = hand_mem.read_fixed(args[0]);
    let b = hand_mem.read_fixed(args[1]);
    let out = if a == b { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("eqf32", |args, hand_mem| {
    let a = f32::from_ne_bytes((hand_mem.read_fixed(args[0]) as i32).to_ne_bytes());
    let b = f32::from_ne_bytes((hand_mem.read_fixed(args[1]) as i32).to_ne_bytes());
    let out = if a == b { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("eqf64", |args, hand_mem| {
    let a = f64::from_ne_bytes(hand_mem.read_fixed(args[0]).to_ne_bytes());
    let b = f64::from_ne_bytes(hand_mem.read_fixed(args[1]).to_ne_bytes());
    let out = if a == b { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("eqstr", |args, hand_mem| {
    let a_pascal_string = hand_mem.read_fractal(args[0]);
    let b_pascal_string = hand_mem.read_fractal(args[1]);
    let out = if a_pascal_string == b_pascal_string { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("eqbool", |args, hand_mem| {
    let a = hand_mem.read_fixed(args[0]) as i8;
    let b = hand_mem.read_fixed(args[1]) as i8;
    let out = if a == b { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out);
    None
  });

  cpu!("neqi8", |args, hand_mem| {
    let a = hand_mem.read_fixed(args[0]) as i8;
    let b = hand_mem.read_fixed(args[1]) as i8;
    let out = if a != b { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("neqi16", |args, hand_mem| {
    let a = hand_mem.read_fixed(args[0]) as i16;
    let b = hand_mem.read_fixed(args[1]) as i16;
    let out = if a != b { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("neqi32", |args, hand_mem| {
    let a = hand_mem.read_fixed(args[0]) as i32;
    let b = hand_mem.read_fixed(args[1]) as i32;
    let out = if a != b { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("neqi64", |args, hand_mem| {
    let a = hand_mem.read_fixed(args[0]);
    let b = hand_mem.read_fixed(args[1]);
    let out = if a != b { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("neqf32", |args, hand_mem| {
    let a = f32::from_ne_bytes((hand_mem.read_fixed(args[0]) as i32).to_ne_bytes());
    let b = f32::from_ne_bytes((hand_mem.read_fixed(args[1]) as i32).to_ne_bytes());
    let out = if a != b { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("neqf64", |args, hand_mem| {
    let a = f64::from_ne_bytes(hand_mem.read_fixed(args[0]).to_ne_bytes());
    let b = f64::from_ne_bytes(hand_mem.read_fixed(args[1]).to_ne_bytes());
    let out = if a != b { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("neqstr", |args, hand_mem| {
    let a_pascal_string = hand_mem.read_fractal(args[0]);
    let b_pascal_string = hand_mem.read_fractal(args[1]);
    let out = if a_pascal_string != b_pascal_string { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("neqbool", |args, hand_mem| {
    let a = hand_mem.read_fixed(args[0]) as i8;
    let b = hand_mem.read_fixed(args[1]) as i8;
    let out = if a != b { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out);
    None
  });

  cpu!("lti8", |args, hand_mem| {
    let a = hand_mem.read_fixed(args[0]) as i8;
    let b = hand_mem.read_fixed(args[1]) as i8;
    let out = if a < b { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("lti16", |args, hand_mem| {
    let a = hand_mem.read_fixed(args[0]) as i16;
    let b = hand_mem.read_fixed(args[1]) as i16;
    let out = if a < b { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("lti32", |args, hand_mem| {
    let a = hand_mem.read_fixed(args[0]) as i32;
    let b = hand_mem.read_fixed(args[1]) as i32;
    let out = if a < b { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("lti64", |args, hand_mem| {
    let a = hand_mem.read_fixed(args[0]);
    let b = hand_mem.read_fixed(args[1]);
    let out = if a < b { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("ltf32", |args, hand_mem| {
    let a = f32::from_ne_bytes((hand_mem.read_fixed(args[0]) as i32).to_ne_bytes());
    let b = f32::from_ne_bytes((hand_mem.read_fixed(args[1]) as i32).to_ne_bytes());
    let out = if a < b { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("ltf64", |args, hand_mem| {
    let a = f64::from_ne_bytes(hand_mem.read_fixed(args[0]).to_ne_bytes());
    let b = f64::from_ne_bytes(hand_mem.read_fixed(args[1]).to_ne_bytes());
    let out = if a < b { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("ltstr", |args, hand_mem| {
    let a_str = HandlerMemory::fractal_to_string(hand_mem.read_fractal(args[0]));
    let b_str = HandlerMemory::fractal_to_string(hand_mem.read_fractal(args[1]));
    let out = if a_str < b_str { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out);
    None
  });

  cpu!("ltei8", |args, hand_mem| {
    let a = hand_mem.read_fixed(args[0]) as i8;
    let b = hand_mem.read_fixed(args[1]) as i8;
    let out = if a <= b { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("ltei16", |args, hand_mem| {
    let a = hand_mem.read_fixed(args[0]) as i16;
    let b = hand_mem.read_fixed(args[1]) as i16;
    let out = if a <= b { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("ltei32", |args, hand_mem| {
    let a = hand_mem.read_fixed(args[0]) as i32;
    let b = hand_mem.read_fixed(args[1]) as i32;
    let out = if a <= b { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("ltei64", |args, hand_mem| {
    let a = hand_mem.read_fixed(args[0]);
    let b = hand_mem.read_fixed(args[1]);
    let out = if a <= b { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("ltef32", |args, hand_mem| {
    let a = f32::from_ne_bytes((hand_mem.read_fixed(args[0]) as i32).to_ne_bytes());
    let b = f32::from_ne_bytes((hand_mem.read_fixed(args[1]) as i32).to_ne_bytes());
    let out = if a <= b { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("ltef64", |args, hand_mem| {
    let a = f64::from_ne_bytes(hand_mem.read_fixed(args[0]).to_ne_bytes());
    let b = f64::from_ne_bytes(hand_mem.read_fixed(args[1]).to_ne_bytes());
    let out = if a <= b { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("ltestr", |args, hand_mem| {
    let a_str = HandlerMemory::fractal_to_string(hand_mem.read_fractal(args[0]));
    let b_str = HandlerMemory::fractal_to_string(hand_mem.read_fractal(args[1]));
    let out = if a_str <= b_str { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out);
    None
  });

  cpu!("gti8", |args, hand_mem| {
    let a = hand_mem.read_fixed(args[0]) as i8;
    let b = hand_mem.read_fixed(args[1]) as i8;
    let out = if a > b { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("gti16", |args, hand_mem| {
    let a = hand_mem.read_fixed(args[0]) as i16;
    let b = hand_mem.read_fixed(args[1]) as i16;
    let out = if a > b { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("gti32", |args, hand_mem| {
    let a = hand_mem.read_fixed(args[0]) as i32;
    let b = hand_mem.read_fixed(args[1]) as i32;
    let out = if a > b { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("gti64", |args, hand_mem| {
    let a = hand_mem.read_fixed(args[0]);
    let b = hand_mem.read_fixed(args[1]);
    let out = if a > b { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("gtf32", |args, hand_mem| {
    let a = f32::from_ne_bytes((hand_mem.read_fixed(args[0]) as i32).to_ne_bytes());
    let b = f32::from_ne_bytes((hand_mem.read_fixed(args[1]) as i32).to_ne_bytes());
    let out = if a > b { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("gtf64", |args, hand_mem| {
    let a = f64::from_ne_bytes(hand_mem.read_fixed(args[0]).to_ne_bytes());
    let b = f64::from_ne_bytes(hand_mem.read_fixed(args[1]).to_ne_bytes());
    let out = if a > b { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("gtstr", |args, hand_mem| {
    let a_str = HandlerMemory::fractal_to_string(hand_mem.read_fractal(args[0]));
    let b_str = HandlerMemory::fractal_to_string(hand_mem.read_fractal(args[1]));
    let out = if a_str > b_str { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out);
    None
  });

  cpu!("gtei8", |args, hand_mem| {
    let a = hand_mem.read_fixed(args[0]) as i8;
    let b = hand_mem.read_fixed(args[1]) as i8;
    let out = if a >= b { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("gtei16", |args, hand_mem| {
    let a = hand_mem.read_fixed(args[0]) as i16;
    let b = hand_mem.read_fixed(args[1]) as i16;
    let out = if a >= b { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("gtei32", |args, hand_mem| {
    let a = hand_mem.read_fixed(args[0]) as i32;
    let b = hand_mem.read_fixed(args[1]) as i32;
    let out = if a >= b { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("gtei64", |args, hand_mem| {
    let a = hand_mem.read_fixed(args[0]);
    let b = hand_mem.read_fixed(args[1]);
    let out = if a >= b { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("gtef32", |args, hand_mem| {
    let a = f32::from_ne_bytes((hand_mem.read_fixed(args[0]) as i32).to_ne_bytes());
    let b = f32::from_ne_bytes((hand_mem.read_fixed(args[1]) as i32).to_ne_bytes());
    let out = if a >= b { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("gtef64", |args, hand_mem| {
    let a = f64::from_ne_bytes(hand_mem.read_fixed(args[0]).to_ne_bytes());
    let b = f64::from_ne_bytes(hand_mem.read_fixed(args[1]).to_ne_bytes());
    let out = if a >= b { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("gtestr", |args, hand_mem| {
    let a_str = HandlerMemory::fractal_to_string(hand_mem.read_fractal(args[0]));
    let b_str = HandlerMemory::fractal_to_string(hand_mem.read_fractal(args[1]));
    let out = if a_str >= b_str { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out);
    None
  });

  // String opcodes
  cpu!("catstr", |args, hand_mem| {
    let a_str = HandlerMemory::fractal_to_string(hand_mem.read_fractal(args[0]));
    let b_str = HandlerMemory::fractal_to_string(hand_mem.read_fractal(args[1]));
    let out_str = format!("{}{}", a_str, b_str);
    hand_mem.write_fractal(args[2], &HandlerMemory::str_to_fractal(&out_str));
    None
  });
  cpu!("split", |args, hand_mem| {
    let a_str = HandlerMemory::fractal_to_string(hand_mem.read_fractal(args[0]));
    let b_str = HandlerMemory::fractal_to_string(hand_mem.read_fractal(args[1]));
    let out_hms = a_str.split(&b_str).map(|out_str| HandlerMemory::str_to_fractal(&out_str));
    hand_mem.write_fractal(args[2], &Vec::new());
    for out in out_hms {
      hand_mem.push_fractal(args[2], &out);
    }
    None
  });
  cpu!("repstr", |args, hand_mem| {
    let a_str = HandlerMemory::fractal_to_string(hand_mem.read_fractal(args[0]));
    let n = hand_mem.read_fixed(args[1]);
    let out_str = a_str.repeat(n as usize);
    hand_mem.write_fractal(args[2], &HandlerMemory::str_to_fractal(&out_str));
    None
  });
  cpu!("matches", |args, hand_mem| {
    let a_str = HandlerMemory::fractal_to_string(hand_mem.read_fractal(args[0]));
    let b_str = HandlerMemory::fractal_to_string(hand_mem.read_fractal(args[1]));
    let b_regex = Regex::new(&b_str).unwrap();
    let out = if b_regex.is_match(&a_str) { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("indstr", |args, hand_mem| {
    let a_str = HandlerMemory::fractal_to_string(hand_mem.read_fractal(args[0]));
    let b_str = HandlerMemory::fractal_to_string(hand_mem.read_fractal(args[1]));
    let out_option = a_str.find(&b_str);
    hand_mem.write_fractal(args[2], &Vec::new());
    if out_option.is_none() {
      hand_mem.push_fixed(args[2], 0i64);
      hand_mem.write_fractal(args[2], &HandlerMemory::str_to_fractal("substring not found"));
    } else {
      hand_mem.push_fixed(args[2], 1i64);
      let out = out_option.unwrap() as i64;
      hand_mem.push_fixed(args[2], out);
    }
    None
  });
  cpu!("lenstr", |args, hand_mem| {
    let pascal_string = hand_mem.read_fractal(args[0]);
    let out = pascal_string[0];
    hand_mem.write_fixed(args[2], out.1);
    None
  });
  cpu!("trim", |args, hand_mem| {
    let in_str = HandlerMemory::fractal_to_string(hand_mem.read_fractal(args[0]));
    let out_str = in_str.trim();
    hand_mem.write_fractal(args[2], &HandlerMemory::str_to_fractal(&out_str));
    None
  });

  // Array opcodes
  cpu!("register", |args, hand_mem| {
    // args[2] is the register address
    // args[0] point to an array in memory
    // args[1] is the address within the array to register
    let inner_addr = hand_mem.read_fixed(args[1]);
    hand_mem.register_out(args[0], inner_addr, args[2]);
    None
  });
  cpu!("copyfrom", |args, hand_mem| {
    // args = [arr_addr, arr_idx_addr, outer_addr]
    // copy data from outer_addr to inner_addr of the array in reg_addr
    // The array index instead of inner address is provided to keep interaction with the js-runtime
    // sane.
    let inner_addr = hand_mem.read_fixed(args[1]);
    hand_mem.register_out(args[0], inner_addr, args[2]);
    None
  });
  cpu!("copytof", |args, hand_mem| {
    // args = [arr_addr, inner_addr, outer_addr]
    // copy data from outer_addr to inner_addr in arr_addr
    let inner = hand_mem.read_fixed(args[1]);
    hand_mem.register_in(args[2], args[0], inner);
    None
  });
  cpu!("copytov", |args, hand_mem| {
    // args = [arr_addr, inner_addr, outer_addr]
    // copy data from outer_addr to inner_addr in arr_addr
    let inner = hand_mem.read_fixed(args[1]);
    hand_mem.register_in(args[2], args[0], inner);
    None
  });
  cpu!("lenarr", |args, hand_mem| {
    let arr = hand_mem.read_fractal(args[0]);
    let len = arr.len() as i64;
    hand_mem.write_fixed(args[2], len);
    None
  });
  cpu!("indarrf", |args, hand_mem| {
    let val = hand_mem.read_fixed(args[1]);
    let mem = hand_mem.read_fractal(args[0]);
    let len = mem.len();
    let mut idx = -1i64;
    for i in 0..len {
      let check = mem[i].1;
      if val == check {
        idx = i as i64;
        break
      }
    }
    hand_mem.write_fractal(args[2], &Vec::new());
    if idx == -1i64 {
      hand_mem.push_fixed(args[2], 0i64);
      hand_mem.write_fractal(args[2], &HandlerMemory::str_to_fractal("element not found"));
    } else {
      hand_mem.push_fixed(args[2], 1i64);
      hand_mem.push_fixed(args[2], idx);
    }
    None
  });
  cpu!("indarrv", |args, hand_mem| {
    let val = hand_mem.read_fractal(args[1]);
    let mem = hand_mem.read_fractal(args[0]);
    let len = mem.len();
    let mut idx = -1i64;
    for i in 0..len {
      let (a, b) = mem[i];
      let (check, is_fractal) = hand_mem.read_either_idxs(a, b as usize);
      if is_fractal {
        // TODO: equality comparisons for nested arrays, for now, assume it's string-like
        if val.len() != check.len() {
          continue
        }
        let mut matches = true;
        for j in 0..val.len() {
          if val[j] != check[j] {
            matches = false;
            break
          }
        }
        if matches {
          idx = i as i64;
          break
        }
      } else {
        continue
      }
    }
    hand_mem.write_fractal(args[2], &Vec::new());
    if idx == -1i64 {
      hand_mem.push_fixed(args[2], 0i64);
      hand_mem.write_fractal(args[2], &HandlerMemory::str_to_fractal("element not found"));
    } else {
      hand_mem.push_fixed(args[2], 1i64);
      hand_mem.push_fixed(args[2], idx);
    }
    None
  });
  cpu!("join", |args, hand_mem| {
    let sep_str = HandlerMemory::fractal_to_string(hand_mem.read_fractal(args[1]));
    let mem = hand_mem.read_fractal(args[0]);
    let len = mem.len();
    let mut strs: Vec<String> = Vec::new();
    for i in 0..len {
      let (a, b) = mem[i];
      let (data, is_fractal) = hand_mem.read_either_idxs(a, b as usize);
      if is_fractal {
        let v_str = HandlerMemory::fractal_to_string(&data);
        strs.push(v_str);
      } else {
        // TODO: Skip for now
      }
    }
    let out_str = strs.join(&sep_str);
    hand_mem.write_fractal(args[2], &HandlerMemory::str_to_fractal(&out_str));
    None
  });
  cpu!("pusharr", |args, hand_mem| {
    let val_size = hand_mem.read_fixed(args[2]);
    if val_size == 0 {
      hand_mem.push_register(args[0], args[1]);
    } else {
      let val = hand_mem.read_fixed(args[1]);
      hand_mem.push_fixed(args[0], val);
    }
    None
  });
  cpu!("poparr", |args, hand_mem| {
    let last = hand_mem.pop(args[0]);
    hand_mem.write_fractal(args[2], &Vec::new());
    if last.is_ok() {
      hand_mem.push_fixed(args[2], 1i64);
      let record = last.ok().unwrap();
      let (a, b) = record;
      hand_mem.push_idxs(args[2], a, b as usize);
    } else {
      hand_mem.push_fixed(args[2], 0i64);
      let error_string = last.err().unwrap();
      hand_mem.push_fractal(args[2], &HandlerMemory::str_to_fractal(&error_string));
    }
    None
  });
  cpu!("delindx", |args, hand_mem| {
    let idx = hand_mem.read_fixed(args[1]) as usize;
    let el = hand_mem.delete(args[0], idx);
    hand_mem.write_fractal(args[2], &Vec::new());
    if el.is_ok() {
      hand_mem.push_fixed(args[2], 1i64);
      let record = el.ok().unwrap();
      let (a, b) = record;
      hand_mem.push_idxs(args[2], a, b as usize);
    } else {
      hand_mem.push_fixed(args[2], 0i64);
      let error_string = el.err().unwrap();
      hand_mem.push_fractal(args[2], &HandlerMemory::str_to_fractal(&error_string));
    }
    None
  });
  cpu!("newarr", |args, hand_mem| {
    hand_mem.write_fractal(args[2], &Vec::new());
    None
  });
  io!("map", |args, mut hand_mem| {
    Box::pin(async move {
      let arr = hand_mem.read_fractal(args[0]);
      let len = arr.len();
      let subhandler = HandlerFragment::new(args[1], 0);
      let mut mappers = Vec::new();
      for i in 0..len {
        let mut hm = hand_mem.fork();
        hm.set_addr(CLOSURE_ARG_MEM_START + 1, arr[i].0, arr[i].1 as usize);
        hm.write_fixed(CLOSURE_ARG_MEM_START + 2, i as i64);
        mappers.push(subhandler.clone().run(hm));
      }
      let hms = join_all(mappers).await;
      hand_mem.write_fractal(args[2], &Vec::new());
      for hm in hms {
        hand_mem.join(hm);
        let (a, b) = hand_mem.addr_to_idxs(CLOSURE_ARG_MEM_START);
        hand_mem.push_idxs(args[2], a, b);
      }
      hand_mem
    })
  });
  io!("mapl", |args, mut hand_mem| {
    Box::pin(async move {
      let arr = hand_mem.read_fractal(args[0]);
      let arrv = arr.to_vec();
      let len = arrv.len();
      drop(arr);
      let subhandler = HandlerFragment::new(args[1], 0);
      hand_mem.write_fractal(args[2], &Vec::new());
      for i in 0..len {
        hand_mem.set_addr(CLOSURE_ARG_MEM_START + 1, arrv[i].0, arrv[i].1 as usize);
        hand_mem.write_fixed(CLOSURE_ARG_MEM_START + 2, i as i64);
        hand_mem = subhandler.clone().run(hand_mem).await;
        let (a, b) = hand_mem.addr_to_idxs(CLOSURE_ARG_MEM_START);
        hand_mem.push_idxs(args[2], a, b);
      }
      hand_mem
    })
  });
  cpu!("reparr", |args, hand_mem| {
    hand_mem.write_fractal(args[2], &Vec::new());
    let n = hand_mem.read_fixed(args[1]);
    let arr = hand_mem.read_fractal(args[0]);
    let l = arr.len();
    let mut is_fixed = true;
    let mut output: Vec<Vec<(usize, i64)>> = vec![];
    for i in 0..n {
      for j in 0..l {
        let (a, b) = arr[j];
        let (val, is_fractal) = hand_mem.read_either_idxs(a, b as usize);
        output.push(val);
        if i == 0 && is_fractal {
          is_fixed = false;
        }
      }
    }
    for val in output {
      if is_fixed {
        hand_mem.push_fixed(args[2], val[0].1);
      } else {
        hand_mem.push_fractal(args[2], &val);
      }
    }
    None
  });
  io!("each", |args, mut hand_mem| {
    Box::pin(async move {
      let arr = hand_mem.read_fractal(args[0]);
      let len = arr.len();
      let subhandler = HandlerFragment::new(args[1], 0);
      let mut runners = Vec::new();
      for i in 0..len {
        let mut hm = hand_mem.fork();
        hm.set_addr(CLOSURE_ARG_MEM_START + 1, arr[i].0, arr[i].1 as usize);
        hm.write_fixed(CLOSURE_ARG_MEM_START + 2, i as i64);
        runners.push(subhandler.clone().run(hm));
      }
      join_all(runners).await;
      hand_mem
    })
  });
  io!("eachl", |args, mut hand_mem| {
    Box::pin(async move {
      let arr = hand_mem.read_fractal(args[0]).to_vec();
      let len = arr.len();
      let subhandler = HandlerFragment::new(args[1], 0);
      for i in 0..len {
        hand_mem.set_addr(CLOSURE_ARG_MEM_START + 1, arr[i].0, arr[i].1 as usize);
        hand_mem.write_fixed(CLOSURE_ARG_MEM_START + 2, i as i64);
        hand_mem = subhandler.clone().run_local(hand_mem).await;
      }
      hand_mem
    })
  });
  io!("find", |args, mut hand_mem| {
    Box::pin(async move {
      let arr = hand_mem.read_fractal(args[0]);
      let len = arr.len();
      let subhandler = HandlerFragment::new(args[1], 0);
      let mut finders = Vec::new();
      for i in 0..len {
        let mut hm = hand_mem.fork();
        hm.set_addr(CLOSURE_ARG_MEM_START + 1, arr[i].0, arr[i].1 as usize);
        finders.push(subhandler.clone().run(hm));
      }
      let hms = join_all(finders).await;
      for i in 0..len {
        let hm = &hms[i];
        let val = hm.read_fixed(CLOSURE_ARG_MEM_START);
        if val == 1 {
          let (a, b) = arr[i];
          hand_mem.write_fractal(args[2], &Vec::new());
          hand_mem.push_fixed(args[2], 1);
          hand_mem.push_idxs(args[2], a, b as usize);
          return hand_mem;
        }
      }
      hand_mem.write_fractal(args[2], &Vec::new());
      hand_mem.push_fixed(args[2], 0);
      hand_mem.push_fractal(args[2], &HandlerMemory::str_to_fractal("no element matches"));
      hand_mem
    })
  });
  io!("findl", |args, mut hand_mem| {
    Box::pin(async move {
      let arr = hand_mem.read_fractal(args[0]).to_vec();
      let len = arr.len();
      let subhandler = HandlerFragment::new(args[1], 0);
      for i in 0..len {
        hand_mem.set_addr(CLOSURE_ARG_MEM_START + 1, arr[i].0, arr[i].1 as usize);
        hand_mem = subhandler.clone().run(hand_mem).await;
        let val = hand_mem.read_fixed(CLOSURE_ARG_MEM_START);
        if val == 1 {
          let (a, b) = arr[i];
          hand_mem.write_fractal(args[2], &Vec::new());
          hand_mem.push_fixed(args[2], 1);
          hand_mem.push_idxs(args[2], a, b as usize);
          return hand_mem;
        }
      }
      hand_mem.write_fractal(args[2], &Vec::new());
      hand_mem.push_fixed(args[2], 0);
      hand_mem.push_fractal(args[2], &HandlerMemory::str_to_fractal("no element matches"));
      hand_mem
    })
  });
  io!("some", |args, mut hand_mem| {
    Box::pin(async move {
      let arr = hand_mem.read_fractal(args[0]);
      let len = arr.len();
      let subhandler = HandlerFragment::new(args[1], 0);
      let mut somers = Vec::new();
      for i in 0..len {
        let mut hm = hand_mem.fork();
        hm.set_addr(CLOSURE_ARG_MEM_START + 1, arr[i].0, arr[i].1 as usize);
        somers.push(subhandler.clone().run(hm));
      }
      let hms = join_all(somers).await;
      for hm in hms {
        let val = hm.read_fixed(CLOSURE_ARG_MEM_START);
        if val == 1 {
          hand_mem.write_fixed(args[2], 1);
          return hand_mem;
        }
      }
      hand_mem.write_fixed(args[2], 0);
      hand_mem
    })
  });
  io!("somel", |args, mut hand_mem| {
    Box::pin(async move {
      let arr = hand_mem.read_fractal(args[0]).to_vec();
      let len = arr.len();
      let subhandler = HandlerFragment::new(args[1], 0);
      for i in 0..len {
        hand_mem.set_addr(CLOSURE_ARG_MEM_START + 1, arr[i].0, arr[i].1 as usize);
        hand_mem = subhandler.clone().run(hand_mem).await;
        let val = hand_mem.read_fixed(CLOSURE_ARG_MEM_START);
        if val == 1 {
          hand_mem.write_fixed(args[2], 1);
          return hand_mem;
        }
      }
      hand_mem.write_fixed(args[2], 0);
      hand_mem
    })
  });
  io!("every", |args, mut hand_mem| {
    Box::pin(async move {
      let arr = hand_mem.read_fractal(args[0]);
      let len = arr.len();
      let subhandler = HandlerFragment::new(args[1], 0);
      let mut somers = Vec::new();
      for i in 0..len {
        let mut hm = hand_mem.fork();
        hm.set_addr(CLOSURE_ARG_MEM_START + 1, arr[i].0, arr[i].1 as usize);
        somers.push(subhandler.clone().run(hm));
      }
      let hms = join_all(somers).await;
      for hm in hms {
        let val = hm.read_fixed(CLOSURE_ARG_MEM_START);
        if val == 0 {
          hand_mem.write_fixed(args[2], 0);
          return hand_mem;
        }
      }
      hand_mem.write_fixed(args[2], 1);
      hand_mem
    })
  });
  io!("everyl", |args, mut hand_mem| {
    Box::pin(async move {
      let arr = hand_mem.read_fractal(args[0]).to_vec();
      let len = arr.len();
      let subhandler = HandlerFragment::new(args[1], 0);
      for i in 0..len {
        hand_mem.set_addr(CLOSURE_ARG_MEM_START + 1, arr[i].0, arr[i].1 as usize);
        hand_mem = subhandler.clone().run(hand_mem).await;
        let val = hand_mem.read_fixed(CLOSURE_ARG_MEM_START);
        if val == 0 {
          hand_mem.write_fixed(args[2], 0);
          return hand_mem;
        }
      }
      hand_mem.write_fixed(args[2], 1);
      hand_mem
    })
  });
  cpu!("catarr", |args, hand_mem| {
    let arr1 = hand_mem.read_fractal(args[0]).to_vec();
    let arr2 = hand_mem.read_fractal(args[1]).to_vec();
    let arr1len = arr1.len();
    let arr2len = arr2.len();
    hand_mem.write_fractal(args[2], &Vec::new());
    for i in 0..arr1len {
      let (a, b) = arr1[i];
      hand_mem.push_idxs(args[2], a, b as usize);
    }
    for i in 0..arr2len {
      let (a, b) = arr2[i];
      hand_mem.push_idxs(args[2], a, b as usize);
    }
    None
  });
  io!("reducep", |args, mut hand_mem| {
    Box::pin(async move {
      let arr = hand_mem.read_fractal(args[0]).to_vec();
      let mut vals: Vec<HandlerMemory> = vec![];
      for i in 0..arr.len() {
        let mut hm = HandlerMemory::new(None, 1);
        hand_mem.set_addr(CLOSURE_ARG_MEM_START, arr[i].0, arr[i].1 as usize);
        HandlerMemory::transfer(&hand_mem, CLOSURE_ARG_MEM_START, &mut hm, 0);
        vals.push(hm);
      }
      let subhandler = HandlerFragment::new(args[1], 0);
      // Log-n parallelism. First n/2 in parallel, then n/4, then n/8, etc
      while vals.len() > 1 {
        let mut reducers = Vec::new();
        while vals.len() > 1 {
          let mut hm = hand_mem.clone();
          let a = vals.remove(0);
          let b = vals.remove(0);
          HandlerMemory::transfer(&a, 0, &mut hm, CLOSURE_ARG_MEM_START + 1);
          HandlerMemory::transfer(&b, 0, &mut hm, CLOSURE_ARG_MEM_START + 2);
          reducers.push(subhandler.clone().run(hm));
        }
        let hms = join_all(reducers).await;
        for mut hm in hms {
          let (a, b) = hm.addr_to_idxs(CLOSURE_ARG_MEM_START);
          hm.set_addr(0, a, b as usize);
          vals.push(hm);
        }
      }
      // There can be only one
      HandlerMemory::transfer(&vals[0], 0, &mut hand_mem, args[2]);
      hand_mem
    })
  });
  io!("reducel", |args, mut hand_mem| {
    Box::pin(async move {
      let arr = hand_mem.read_fractal(args[0]).to_vec();
      if arr.len() == 0 {
        return hand_mem;
      }
      let mut vals: Vec<HandlerMemory> = vec![];
      for i in 0..arr.len() {
        let mut hm = HandlerMemory::new(None, 1);
        hand_mem.set_addr(CLOSURE_ARG_MEM_START, arr[i].0, arr[i].1 as usize);
        HandlerMemory::transfer(&hand_mem, CLOSURE_ARG_MEM_START, &mut hm, 0);
        vals.push(hm);
      }
      let subhandler = HandlerFragment::new(args[1], 0);
      let mut cumulative = vals.remove(0);
      for i in 0..vals.len() {
        let current = &vals[i];
        HandlerMemory::transfer(&cumulative, 0, &mut hand_mem, CLOSURE_ARG_MEM_START + 1);
        HandlerMemory::transfer(&current, 0, &mut hand_mem, CLOSURE_ARG_MEM_START + 2);
        hand_mem = subhandler.clone().run(hand_mem).await;
        HandlerMemory::transfer(&hand_mem, CLOSURE_ARG_MEM_START, &mut cumulative, 0);
      }
      HandlerMemory::transfer(&cumulative, 0, &mut hand_mem, args[2]);
      hand_mem
    })
  });
  io!("foldp", |args, mut hand_mem| {
    Box::pin(async move {
      let obj = hand_mem.read_fractal(args[0]);
      let (a, b) = obj[0];
      let (c, d) = obj[1];
      let (arr, _) = hand_mem.read_either_idxs(a, b as usize);
      let arrv = arr.to_vec();
      let mut vals: Vec<HandlerMemory> = vec![];
      for i in 0..arrv.len() {
        let mut hm = HandlerMemory::new(None, 1);
        hand_mem.set_addr(CLOSURE_ARG_MEM_START, arrv[i].0, arrv[i].1 as usize);
        HandlerMemory::transfer(&hand_mem, CLOSURE_ARG_MEM_START, &mut hm, 0);
        vals.push(hm);
      }
      let subhandler = HandlerFragment::new(args[1], 0);
      hand_mem.set_addr(CLOSURE_ARG_MEM_START, c, d as usize);
      let mut init = HandlerMemory::new(None, 1);
      HandlerMemory::transfer(&hand_mem, CLOSURE_ARG_MEM_START, &mut init, 0);
      // We can only go up to 'n' parallel sequential computations here
      let n = num_cpus::get();
      let l = vals.len();
      let s = l / n;
      let mut reducers = Vec::new();
      for i in 0..n {
        let subvals = if i == n - 1 {
          vals[i*s..].to_vec()
        } else {
          vals[i*s..(i+1)*s].to_vec()
        };
        let mem = hand_mem.clone();
        let init2 = init.clone();
        let subhandler2 = subhandler.clone();
        reducers.push(task::spawn(async move {
          let mut cumulative = init2.clone();
          for i in 0..subvals.len() {
            let current = &subvals[i];
            let mut hm = mem.clone();
            HandlerMemory::transfer(&cumulative, 0, &mut hm, CLOSURE_ARG_MEM_START + 1);
            HandlerMemory::transfer(current, 0, &mut hm, CLOSURE_ARG_MEM_START + 2);
            hm = subhandler2.clone().run(hm).await;
            HandlerMemory::transfer(&hm, CLOSURE_ARG_MEM_START, &mut cumulative, 0);
          }
          cumulative
        }));
      }
      hand_mem.write_fractal(args[2], &Vec::new());
      let hms = join_all(reducers).await;
      for i in 0..n {
        let hm = hms[i].as_ref().unwrap();
        HandlerMemory::transfer(&hm, 0, &mut hand_mem, CLOSURE_ARG_MEM_START);
        let (a, b) = hand_mem.addr_to_idxs(CLOSURE_ARG_MEM_START);
        hand_mem.push_idxs(args[2], a, b);
      }
      hand_mem
    })
  });
  io!("foldl", |args, mut hand_mem| {
    Box::pin(async move {
      let obj = hand_mem.read_fractal(args[0]);
      let (a, b) = obj[0];
      let (c, d) = obj[1];
      let (arr, _) = hand_mem.read_either_idxs(a, b as usize);
      let arrv = arr.to_vec();
      let mut vals: Vec<HandlerMemory> = vec![];
      for i in 0..arrv.len() {
        let mut hm = HandlerMemory::new(None, 1);
        hand_mem.set_addr(CLOSURE_ARG_MEM_START, arrv[i].0, arrv[i].1 as usize);
        HandlerMemory::transfer(&hand_mem, CLOSURE_ARG_MEM_START, &mut hm, 0);
        vals.push(hm);
      }
      let subhandler = HandlerFragment::new(args[1], 0);
      hand_mem.set_addr(CLOSURE_ARG_MEM_START, c, d as usize);
      let mut cumulative = HandlerMemory::new(None, 1);
      HandlerMemory::transfer(&hand_mem, CLOSURE_ARG_MEM_START, &mut cumulative, 0);
      for i in 0..vals.len() {
        let current = &vals[i];
        HandlerMemory::transfer(&cumulative, 0, &mut hand_mem, CLOSURE_ARG_MEM_START + 1);
        HandlerMemory::transfer(current, 0, &mut hand_mem, CLOSURE_ARG_MEM_START + 2);
        hand_mem = subhandler.clone().run(hand_mem).await;
        HandlerMemory::transfer(&hand_mem, CLOSURE_ARG_MEM_START, &mut cumulative, 0);
      }
      let (a, b) = hand_mem.addr_to_idxs(CLOSURE_ARG_MEM_START);
      hand_mem.set_addr(args[2], a, b as usize);
      hand_mem
    })
  });
  io!("filter", |args, mut hand_mem| {
    Box::pin(async move {
      let arr = hand_mem.read_fractal(args[0]).to_vec();
      let len = arr.len();
      let subhandler = HandlerFragment::new(args[1], 0);
      let mut filters = Vec::new();
      for i in 0..len {
        let mut hm = hand_mem.fork();
        hm.set_addr(CLOSURE_ARG_MEM_START + 1, arr[i].0, arr[i].1 as usize);
        filters.push(subhandler.clone().run(hm));
      }
      let hms = join_all(filters).await;
      hand_mem.write_fractal(args[2], &Vec::new());
      for i in 0..len {
        let hm = &hms[i];
        let val = hm.read_fixed(CLOSURE_ARG_MEM_START);
        if val == 1 {
          let (a, b) = arr[i];
          hand_mem.push_idxs(args[2], a, b as usize);
        }
      }
      hand_mem
    })
  });
  io!("filterl", |args, mut hand_mem| {
    Box::pin(async move {
      let arr = hand_mem.read_fractal(args[0]).to_vec();
      let len = arr.len();
      let subhandler = HandlerFragment::new(args[1], 0);
      hand_mem.write_fractal(args[2], &Vec::new());
      for i in 0..len {
        hand_mem.set_addr(CLOSURE_ARG_MEM_START + 1, arr[i].0, arr[i].1 as usize);
        hand_mem = subhandler.clone().run(hand_mem).await;
        let val = hand_mem.read_fixed(CLOSURE_ARG_MEM_START);
        if val == 1 {
          let (a, b) = arr[i];
          hand_mem.push_idxs(args[2], a, b as usize);
        }
      }
      hand_mem
    })
  });
  // Conditional opcode
  io!("condfn", |args, mut hand_mem| {
    Box::pin(async move {
      let cond = hand_mem.read_fixed(args[0]);
      let subhandler = HandlerFragment::new(args[1], 0);
      if cond == 1 {
        let mut hm = hand_mem.clone();
        hm = subhandler.run(hm).await;
        hm.replace(&mut hand_mem);
      }
      hand_mem
    })
  });
  // Std opcodes
  io!("execop", |args, mut hand_mem| {
    Box::pin(async move {
      let full_cmd = HandlerMemory::fractal_to_string(hand_mem.read_fractal(args[0]));
      let split_cmd: Vec<&str> = full_cmd.split(" ").collect();
      let output = Command::new(split_cmd[0]).args(&split_cmd[1..]).output();
      hand_mem.write_fractal(args[2], &Vec::new());
      match output {
        Err(e) => {
          hand_mem.push_fixed(args[2], 127);
          hand_mem.push_fractal(args[2], &vec![(0, 0)]);
          let error_string = e.to_string();
          hand_mem.push_fractal(args[2], &HandlerMemory::str_to_fractal(&error_string));
        },
        Ok(output_res) => {
          let status_code = output_res.status.code().unwrap_or(127) as i64;
          hand_mem.push_fixed(args[2], status_code);
          let stdout_str = String::from_utf8(output_res.stdout).unwrap_or("".to_string());
          hand_mem.push_fractal(args[2], &HandlerMemory::str_to_fractal(&stdout_str));
          let stderr_str = String::from_utf8(output_res.stderr).unwrap_or("".to_string());
          hand_mem.push_fractal(args[2], &HandlerMemory::str_to_fractal(&stderr_str));
        },
      };
      hand_mem
    })
  });
  // IO opcodes
  io!("waitop", |args, mut hand_mem| {
    Box::pin(async move {
      let ms = hand_mem.read_fixed(args[0]) as u64;
      delay_for(Duration::from_millis(ms)).await;
      hand_mem
    })
  });
  io!("httpget", |args, mut hand_mem| {
    Box::pin(async move {
      let url = HandlerMemory::fractal_to_string(hand_mem.read_fractal(args[0]));
      let http_res = reqwest::get(&url).await;
      let mut is_ok = true;
      let result_str = if http_res.is_err() {
        is_ok = false;
        format!("{}", http_res.err().unwrap())
      } else {
        let body = http_res.ok().unwrap().text().await;
        if body.is_err() {
          is_ok = false;
          format!("{}", body.err().unwrap())
        } else {
          body.unwrap()
        }
      };
      let result = if is_ok { 1i64 } else { 0i64 };
      hand_mem.write_fractal(args[2], &Vec::new());
      hand_mem.push_fixed(args[2], result);
      hand_mem.push_fractal(args[2], &HandlerMemory::str_to_fractal(&result_str));
      hand_mem
    })
  });
  io!("httppost", |args, mut hand_mem| {
    Box::pin(async move {
      let url = HandlerMemory::fractal_to_string(hand_mem.read_fractal(args[0]));
      let payload = HandlerMemory::fractal_to_string(hand_mem.read_fractal(args[1]));
      let client = reqwest::Client::new();
      let http_res = client.post(&url).body(payload.clone()).send().await;
      let mut is_ok = true;
      let result_str = if http_res.is_err() {
        is_ok = false;
        format!("{}", http_res.err().unwrap())
      } else {
        let body = http_res.ok().unwrap().text().await;
        if body.is_err() {
          is_ok = false;
          format!("{}", body.err().unwrap())
        } else {
          body.unwrap()
        }
      };
      let result = if is_ok { 1i64 } else { 0i64 };
      hand_mem.write_fractal(args[2], &Vec::new());
      hand_mem.push_fixed(args[2], result);
      hand_mem.push_fractal(args[2], &HandlerMemory::str_to_fractal(&result_str));
      hand_mem
    })
  });

  async fn http_listener(req: Request<Body>) -> Result<Response<Body>, Infallible> {
    // Create a new event handler memory to add to the event queue
    let mut event = HandlerMemory::new(None, 1);
    // Grab the URL
    let url_str = req.uri().to_string();
    let url = HandlerMemory::str_to_fractal(&url_str);
    // Grab the headers
    let headers = req.headers();
    let mut headers_hm = HandlerMemory::new(None, headers.len() as i64);
    let mut i = 0;
    for (key, val) in headers.iter() {
      let key_str = key.as_str();
      let val_str = val.to_str().unwrap();
      headers_hm.write_fractal(i, &Vec::new());
      headers_hm.push_fractal(i, &HandlerMemory::str_to_fractal(key_str));
      headers_hm.push_fractal(i, &HandlerMemory::str_to_fractal(val_str));
      i = i + 1;
    }
    // Grab the body, if any
    let body_req = hyper::body::to_bytes(req.into_body()).await;
    // If we error out while getting the body, just close this listener out immediately
    if body_req.is_err() {
      return Ok(Response::new("Conection terminated".into()));
    }
    let body_str = str::from_utf8(&body_req.unwrap()).unwrap().to_string();
    let body = HandlerMemory::str_to_fractal(&body_str);
    // Generate a connection ID
    let conn_id = OsRng.next_u64() as i64;
    // Populate the event and emit it
    event.write_fractal(0, &Vec::new());
    event.push_fractal(0, &url);
    HandlerMemory::transfer(&headers_hm, 0, &mut event, CLOSURE_ARG_MEM_START);
    let (a, b) = event.addr_to_idxs(CLOSURE_ARG_MEM_START);
    event.push_idxs(0, a, b);
    event.push_fractal(0, &body);
    event.push_fixed(0, conn_id);
    let event_emit = EventEmit {
      id: i64::from(BuiltInEvents::HTTPCONN),
      payload: Some(event),
    };
    let event_tx = EVENT_TX.get().unwrap();
    if event_tx.send(event_emit).is_err() {
      eprintln!("Event transmission error");
      std::process::exit(3);
    }
    // Get the HTTP responses lock and periodically poll it for responses from the user code
    // TODO: Swap from a timing-based poll to getting the waker to the user code so it can be
    // woken only once and reduce pressure on this lock.
    let responses = Arc::clone(&HTTP_RESPONSES);
    let response_hm = poll_fn(|cx: &mut Context<'_>| -> Poll<HandlerMemory> {
      let responses_hm = responses.lock().unwrap();
      let hm = responses_hm.get(&conn_id);
      if hm.is_some() {
        Poll::Ready(hm.unwrap().clone())
      } else {
        drop(hm);
        drop(responses_hm);
        let waker = cx.waker().clone();
        thread::spawn(|| {
          thread::sleep(Duration::from_millis(10));
          waker.wake();
        });
        Poll::Pending
      }
    }).await;
    // Get the status from the user response and begin building the response object
    let status = response_hm.read_fixed(0) as u16;
    let mut res = Response::builder().status(StatusCode::from_u16(status).unwrap());
    // Get the headers and populate the response object
    let headers = res.headers_mut().unwrap();
    let header_hms = response_hm.read_fractal(1);
    for header_hm in header_hms {
      let (h, _) = response_hm.read_either_idxs(header_hm.0, header_hm.1 as usize);
      let (key_hm, _) = response_hm.read_either_idxs(h[0].0, h[0].1 as usize);
      let key = HandlerMemory::fractal_to_string(&key_hm);
      let (val_hm, _) = response_hm.read_either_idxs(h[1].0, h[1].1 as usize);
      let val = HandlerMemory::fractal_to_string(&val_hm);
      let name = HeaderName::from_bytes(key.as_bytes()).unwrap();
      let value = HeaderValue::from_str(&val).unwrap();
      headers.insert(name, value);
    }
    // Get the body, populate the response object, and fire it out
    let body = HandlerMemory::fractal_to_string(response_hm.read_fractal(2));
    Ok(res.body(body.into()).unwrap())
  }

  io!("httplsn", |args, mut hand_mem| {
    Box::pin(async move {
      let port_num = hand_mem.read_fixed(args[0]) as u16;
      let addr = SocketAddr::from(([127, 0, 0, 1], port_num));
      let make_svc = make_service_fn(|_conn| async {
        Ok::<_, Infallible>(service_fn(http_listener))
      });

      let bind = Server::try_bind(&addr);
      hand_mem.write_fractal(args[2], &Vec::new());
      if bind.is_err() {
        hand_mem.push_fixed(args[2], 0i64);
        let result_str = format!("{}", bind.err().unwrap());
        hand_mem.push_fractal(args[2], &HandlerMemory::str_to_fractal(&result_str));
        return hand_mem;
      } else {
        hand_mem.push_fixed(args[2], 1i64);
        hand_mem.push_fractal(args[2], &HandlerMemory::str_to_fractal("ok"));
      }
      let server = bind.unwrap().serve(make_svc);
      tokio::spawn(async move {
        server.await
      });
      hand_mem
    })
  });
  io!("httpsend", |args, mut hand_mem| {
    Box::pin(async move {
      hand_mem.dupe(args[0], args[0]); // Make sure there's no pointers involved
      let response = hand_mem.read_fractal(args[0]);
      let conn_id = response[3].1;
      let responses = Arc::clone(&HTTP_RESPONSES);
      let mut responses_hm = responses.lock().unwrap();
      let mut hm = HandlerMemory::new(None, 1);
      HandlerMemory::transfer(&hand_mem, args[0], &mut hm, CLOSURE_ARG_MEM_START);
      let res_out = hm.read_fractal(CLOSURE_ARG_MEM_START).to_vec();
      for i in 0..res_out.len() {
        hm.set_addr(i as i64, res_out[i].0, res_out[i].1 as usize);
      }
      responses_hm.insert(conn_id, hm);
      drop(responses_hm);
      // TODO: Add a second synchronization tool to return a valid Result status, for now, just
      // return success
      hand_mem.write_fractal(args[2], &Vec::new());
      hand_mem.push_fixed(args[2], 0i64);
      hand_mem.push_fractal(args[2], &HandlerMemory::str_to_fractal("ok"));
      hand_mem
    })
  });
  io!("dssetf", |args, mut hand_mem| {
    Box::pin(async move {
      let val = hand_mem.read_fixed(args[2]);
      let mut hm = HandlerMemory::new(None, 1); 
      hm.write_fixed(0, val);
      let ns = HandlerMemory::fractal_to_string(hand_mem.read_fractal(args[0]));
      let key = HandlerMemory::fractal_to_string(hand_mem.read_fractal(args[1]));
      let nskey = format!("{}:{}", ns, key);
      let ds = Arc::clone(&DS);
      ds.insert(nskey, hm);
      hand_mem
    })
  });
  io!("dssetv", |args, mut hand_mem| {
    Box::pin(async move {
      let mut hm = HandlerMemory::new(None, 1);
      HandlerMemory::transfer(&hand_mem, args[2], &mut hm, 0);
      let ns = HandlerMemory::fractal_to_string(hand_mem.read_fractal(args[0]));
      let key = HandlerMemory::fractal_to_string(hand_mem.read_fractal(args[1]));
      let nskey = format!("{}:{}", ns, key);
      let ds = Arc::clone(&DS);
      ds.insert(nskey, hm);
      hand_mem
    })
  });
  io!("dshas", |args, mut hand_mem| {
    Box::pin(async move {
      let ns = HandlerMemory::fractal_to_string(hand_mem.read_fractal(args[0]));
      let key = HandlerMemory::fractal_to_string(hand_mem.read_fractal(args[1]));
      let nskey = format!("{}:{}", ns, key);
      let ds = Arc::clone(&DS);
      let has = ds.contains_key(&nskey);
      hand_mem.write_fixed(args[2], if has { 1i64 } else { 0i64 });
      hand_mem
    })
  });
  io!("dsdel", |args, mut hand_mem| {
    Box::pin(async move {
      let ns = HandlerMemory::fractal_to_string(hand_mem.read_fractal(args[0]));
      let key = HandlerMemory::fractal_to_string(hand_mem.read_fractal(args[1]));
      let nskey = format!("{}:{}", ns, key);
      let ds = Arc::clone(&DS);
      let removed = ds.remove(&nskey).is_some();
      hand_mem.write_fixed(args[2], if removed { 1i64 } else { 0i64 });
      hand_mem
    })
  });
  io!("dsgetf", |args, mut hand_mem| {
    Box::pin(async move {
      let ns = HandlerMemory::fractal_to_string(hand_mem.read_fractal(args[0]));
      let key = HandlerMemory::fractal_to_string(hand_mem.read_fractal(args[1]));
      let nskey = format!("{}:{}", ns, key);
      let ds = Arc::clone(&DS);
      let maybe_hm = ds.get(&nskey);
      hand_mem.write_fractal(args[2], &Vec::new());
      if maybe_hm.is_some() {
        hand_mem.push_fixed(args[2], 1i64);
        hand_mem.push_fixed(args[2], maybe_hm.unwrap().read_fixed(0));
      } else {
        hand_mem.push_fixed(args[2], 0i64);
        let err_msg = "namespace-key pair not found";
        hand_mem.push_fractal(args[2], &HandlerMemory::str_to_fractal(&err_msg));
      }
      hand_mem
    })
  });
  io!("dsgetv", |args, mut hand_mem| {
    Box::pin(async move {
      let ns = HandlerMemory::fractal_to_string(hand_mem.read_fractal(args[0]));
      let key = HandlerMemory::fractal_to_string(hand_mem.read_fractal(args[1]));
      let nskey = format!("{}:{}", ns, key);
      let ds = Arc::clone(&DS);
      let maybe_hm = ds.get(&nskey);
      hand_mem.write_fractal(args[2], &Vec::new());
      if maybe_hm.is_some() {
        hand_mem.push_fixed(args[2], 1i64);
        let hm = maybe_hm.unwrap();
        HandlerMemory::transfer(&hm, 0, &mut hand_mem, CLOSURE_ARG_MEM_START);
        let (a, b) = hand_mem.addr_to_idxs(CLOSURE_ARG_MEM_START);
        hand_mem.push_idxs(args[2], a, b);
      } else {
        hand_mem.push_fixed(args[2], 0i64);
        let err_msg = "namespace-key pair not found";
        hand_mem.push_fractal(args[2], &HandlerMemory::str_to_fractal(&err_msg));
      }
      hand_mem
    })
  });
  cpu!("newseq", |args, hand_mem| {
    hand_mem.write_fractal(args[2], &Vec::new());
    hand_mem.push_fixed(args[2], 0i64);
    hand_mem.push_fixed(args[2], hand_mem.read_fixed(args[0]));
    None
  });
  cpu!("seqnext", |args, hand_mem| {
    hand_mem.write_fractal(args[2], &Vec::new());
    let seq = hand_mem.read_mut_fractal(args[0]);
    let current = seq[0].1;
    let limit = seq[1].1;
    if current < limit {
      seq[0].1 = current + 1 ;
      hand_mem.push_fixed(args[2], 1i64);
      hand_mem.push_fixed(args[2], current);
    } else {
      hand_mem.push_fixed(args[2], 0i64);
      let err_msg = "error: sequence out-of-bounds";
      hand_mem.push_fractal(args[2], &HandlerMemory::str_to_fractal(&err_msg));
    }
    None
  });
  io!("seqeach", |args, mut hand_mem| {
    Box::pin(async move {
      let seq = hand_mem.read_mut_fractal(args[0]);
      let current = seq[0].1;
      let limit = seq[1].1;
      let subhandler = HandlerFragment::new(args[1], 0);
      if current >= limit {
        return hand_mem;
      }
      seq[0].1 = limit;
      let mut hm = hand_mem.clone();
      // array of potentially many levels of nested fractals
      for i in current..limit {
        // array element is $1 argument of the closure memory space
        hm.write_fixed(CLOSURE_ARG_MEM_START + 1, i);
        hm = subhandler.clone().run(hm).await;
      }
      // side-effects are kinda the point of this opcode
      hm.replace(&mut hand_mem);
      hand_mem
    })
  });
  io!("seqwhile", |args, mut hand_mem| {
    Box::pin(async move {
      let seq = hand_mem.read_mut_fractal(args[0]);
      let mut current = seq[0].1;
      let limit = seq[1].1;
      drop(seq);
      let cond_handler = HandlerFragment::new(args[1], 0);
      let body_handler = HandlerFragment::new(args[2], 0);
      if current >= limit {
        return hand_mem;
      }
      let mut hm = hand_mem.clone();
      hm = cond_handler.clone().run(hm).await;
      while current < limit && hm.read_fixed(CLOSURE_ARG_MEM_START) > 0 {
        hm = body_handler.clone().run(hm).await;
        current = current + 1;
        hm = cond_handler.clone().run(hm).await;
      }
      // side-effects are kinda the point of this opcode
      hm.replace(&mut hand_mem);
      let seq = hand_mem.read_mut_fractal(args[0]);
      seq[0].1 = current;
      hand_mem
    })
  });
  io!("seqdo", |args, mut hand_mem| {
    Box::pin(async move {
      let seq = hand_mem.read_mut_fractal(args[0]);
      let mut current = seq[0].1;
      let limit = seq[1].1;
      drop(seq);
      let subhandler = HandlerFragment::new(args[1], 0);
      let mut hm = hand_mem.clone();
      loop {
        hm = subhandler.clone().run(hm).await;
        current = current + 1;
        if current >= limit || hm.read_fixed(CLOSURE_ARG_MEM_START) == 0 {
          break;
        }
      }
      // side-effects are kinda the point of this opcode
      hm.replace(&mut hand_mem);
      let seq = hand_mem.read_mut_fractal(args[0]);
      seq[0].1 = current;
      hand_mem
    })
  });
  io!("selfrec", |args, mut hand_mem| {
    Box::pin(async move {
      let mut hm = hand_mem.fork();
      // MUST read these first in case the arguments are themselves closure args being overwritten
      // for the recursive function.
      let (a, b) = hm.addr_to_idxs(args[0]);
      let (c, d) = hm.addr_to_idxs(args[1]);
      hm.set_addr(CLOSURE_ARG_MEM_START + 1, a, b);
      hm.set_addr(CLOSURE_ARG_MEM_START + 2, c, d);
      let slf = hm.read_fractal(args[0]);
      let recurse_fn = HandlerFragment::new(slf[1].1, 0);
      let mut seq = hm.read_mut_fractal_idxs(slf[0].0, 0);
      if seq[0].1 < seq[1].1 {
        seq[0].1 = seq[0].1 + 1;
        hm = recurse_fn.run(hm).await;
        let (a, b) = hm.addr_to_idxs(CLOSURE_ARG_MEM_START);
        hand_mem.join(hm);
        hand_mem.set_addr(args[2], a, b);
      } else {
        hand_mem.write_fractal(args[2], &Vec::new());
        hand_mem.push_fixed(args[2], 0);
        hand_mem.push_fractal(args[2], &HandlerMemory::str_to_fractal("error: sequence out-of-bounds"));
      }
      hand_mem
    })
  });
  cpu!("seqrec", |args, hand_mem| {
    hand_mem.write_fractal(args[2], &Vec::new());
    let (a, b) = hand_mem.addr_to_idxs(args[0]);
    hand_mem.push_idxs(args[2], a, b);
    hand_mem.push_fixed(args[2], args[1]);
    None
  });
  // "Special" opcodes
  cpu!("exitop", |args, hand_mem| {
    std::process::exit(hand_mem.read_fixed(args[0]) as i32);
  });
  cpu!("stdoutp", |args, hand_mem| {
    let out_str = HandlerMemory::fractal_to_string(hand_mem.read_fractal(args[0]));
    print!("{}", out_str);
    None
  });
  cpu!("stderrp", |args, hand_mem| {
    let err_str = HandlerMemory::fractal_to_string(hand_mem.read_fractal(args[0]));
    eprint!("{}", err_str);
    None
  });
  // set opcodes use args[0] directly, since the relevant value directly
  // fits in i64, and write it to args[2]
  cpu!("seti64", |args, hand_mem| {
    let data = args[0];
    hand_mem.write_fixed(args[2], data);
    None
  });
  cpu!("seti32", |args, hand_mem| {
    let data = (args[0] as i32) as i64;
    hand_mem.write_fixed(args[2], data);
    None
  });
  cpu!("seti16", |args, hand_mem| {
    let data = (args[0] as i16) as i64;
    hand_mem.write_fixed(args[2], data);
    None
  });
  cpu!("seti8", |args, hand_mem| {
    let data = (args[0] as i8) as i64;
    hand_mem.write_fixed(args[2], data);
    None
  });
  cpu!("setf64", |args, hand_mem| {
    let data = i64::from_ne_bytes((args[0] as f64).to_ne_bytes());
    hand_mem.write_fixed(args[2], data);
    None
  });
  cpu!("setf32", |args, hand_mem| {
    let data = i32::from_ne_bytes((args[0] as f32).to_ne_bytes()) as i64;
    hand_mem.write_fixed(args[2], data);
    None
  });
  cpu!("setbool", |args, hand_mem| {
    let data = if args[0] == 0 { 0i64 } else { 1i64 };
    hand_mem.write_fixed(args[2], data);
    None
  });
  cpu!("setestr", |args, hand_mem| {
    let empty_str = vec![(0, 0)];
    hand_mem.write_fractal(args[2], &empty_str);
    None
  });

  // copy opcodes used for let variable reassignments
  cpu!("copyi8", |args, hand_mem| {
    let val = hand_mem.read_fixed(args[0]);
    hand_mem.write_fixed(args[2], val);
    None
  });
  cpu!("copyi16", |args, hand_mem| {
    let val = hand_mem.read_fixed(args[0]);
    hand_mem.write_fixed(args[2], val);
    None
  });
  cpu!("copyi32", |args, hand_mem| {
    let val = hand_mem.read_fixed(args[0]);
    hand_mem.write_fixed(args[2], val);
    None
  });
  cpu!("copyi64", |args, hand_mem| {
    let val = hand_mem.read_fixed(args[0]);
    hand_mem.write_fixed(args[2], val);
    None
  });
  cpu!("copyvoid", |args, hand_mem| {
    let val = hand_mem.read_fixed(args[0]);
    hand_mem.write_fixed(args[2], val);
    None
  });
  cpu!("copyf32", |args, hand_mem| {
    let val = hand_mem.read_fixed(args[0]);
    hand_mem.write_fixed(args[2], val);
    None
  });
  cpu!("copyf64", |args, hand_mem| {
    let val = hand_mem.read_fixed(args[0]);
    hand_mem.write_fixed(args[2], val);
    None
  });
  cpu!("copybool", |args, hand_mem| {
    let val = hand_mem.read_fixed(args[0]);
    hand_mem.write_fixed(args[2], val);
    None
  });
  cpu!("copystr", |args, hand_mem| {
    let pascal_string = hand_mem.read_fractal(args[0]).to_vec().clone();
    hand_mem.write_fractal(args[2], &pascal_string[..]);
    None
  });
  cpu!("copyarr", |args, hand_mem| {
    // args = [in_addr, unused, out_addr]
    hand_mem.dupe(args[0], args[2]);
    None
  });
  cpu!("zeroed", |args, hand_mem| {
    hand_mem.write_fixed(args[2], 0);
    None
  });

  // Trig opcodes
  cpu!("lnf64", |args, hand_mem| {
    let a = f64::from_ne_bytes(hand_mem.read_fixed(args[0]).to_ne_bytes());
    let out = i64::from_ne_bytes(a.ln().to_ne_bytes());
    hand_mem.write_fixed(args[2], out);
    None
  });

  cpu!("logf64", |args, hand_mem| {
    let a = f64::from_ne_bytes(hand_mem.read_fixed(args[0]).to_ne_bytes());
    let out = i64::from_ne_bytes(a.log10().to_ne_bytes());
    hand_mem.write_fixed(args[2], out);
    None
  });

  cpu!("sinf64", |args, hand_mem| {
    let a = f64::from_ne_bytes(hand_mem.read_fixed(args[0]).to_ne_bytes());
    let out = i64::from_ne_bytes(a.sin().to_ne_bytes());
    hand_mem.write_fixed(args[2], out);
    None
  });

  cpu!("cosf64", |args, hand_mem| {
    let a = f64::from_ne_bytes(hand_mem.read_fixed(args[0]).to_ne_bytes());
    let out = i64::from_ne_bytes(a.cos().to_ne_bytes());
    hand_mem.write_fixed(args[2], out);
    None
  });

  cpu!("tanf64", |args, hand_mem| {
    let a = f64::from_ne_bytes(hand_mem.read_fixed(args[0]).to_ne_bytes());
    let out = i64::from_ne_bytes(a.tan().to_ne_bytes());
    hand_mem.write_fixed(args[2], out);
    None
  });

  cpu!("asinf64", |args, hand_mem| {
    let a = f64::from_ne_bytes(hand_mem.read_fixed(args[0]).to_ne_bytes());
    let out = i64::from_ne_bytes(a.asin().to_ne_bytes());
    hand_mem.write_fixed(args[2], out);
    None
  });

  cpu!("acosf64", |args, hand_mem| {
    let a = f64::from_ne_bytes(hand_mem.read_fixed(args[0]).to_ne_bytes());
    let out = i64::from_ne_bytes(a.acos().to_ne_bytes());
    hand_mem.write_fixed(args[2], out);
    None
  });

  cpu!("atanf64", |args, hand_mem| {
    let a = f64::from_ne_bytes(hand_mem.read_fixed(args[0]).to_ne_bytes());
    let out = i64::from_ne_bytes(a.atan().to_ne_bytes());
    hand_mem.write_fixed(args[2], out);
    None
  });

  cpu!("sinhf64", |args, hand_mem| {
    let a = f64::from_ne_bytes(hand_mem.read_fixed(args[0]).to_ne_bytes());
    let out = i64::from_ne_bytes(a.sinh().to_ne_bytes());
    hand_mem.write_fixed(args[2], out);
    None
  });

  cpu!("coshf64", |args, hand_mem| {
    let a = f64::from_ne_bytes(hand_mem.read_fixed(args[0]).to_ne_bytes());
    let out = i64::from_ne_bytes(a.cosh().to_ne_bytes());
    hand_mem.write_fixed(args[2], out);
    None
  });

  cpu!("tanhf64", |args, hand_mem| {
    let a = f64::from_ne_bytes(hand_mem.read_fixed(args[0]).to_ne_bytes());
    let out = i64::from_ne_bytes(a.tanh().to_ne_bytes());
    hand_mem.write_fixed(args[2], out);
    None
  });
  // Error, Maybe, Result, Either opcodes
  cpu!("error", |args, hand_mem| {
    hand_mem.register(args[2], args[0], true);
    None
  });
  cpu!("refv", |args, hand_mem| {
    hand_mem.register(args[2], args[0], true);
    None
  });
  cpu!("reff", |args, hand_mem| {
    hand_mem.register(args[2], args[0], false);
    None
  });
  cpu!("noerr", |args, hand_mem| {
    let empty_string = vec![(0, 0)];
    hand_mem.write_fractal(args[2], &empty_string);
    None
  });
  cpu!("errorstr", |args, hand_mem| {
    hand_mem.register(args[2], args[0], true);
    None
  });
  cpu!("someM", |args, hand_mem| {
    hand_mem.write_fractal(args[2], &Vec::new());
    hand_mem.push_fixed(args[2], 1i64);
    let val_size = hand_mem.read_fixed(args[1]);
    if val_size == 0 {
      hand_mem.push_register(args[2], args[0]);
    } else {
      let val = hand_mem.read_fixed(args[0]);
      hand_mem.push_fixed(args[2], val);
    }
    None
  });
  cpu!("noneM", |args, hand_mem| {
    hand_mem.write_fractal(args[2], &Vec::new());
    hand_mem.push_fixed(args[2], 0i64);
    None
  });
  cpu!("isSome", |args, hand_mem| {
    hand_mem.register_out(args[0], 0, args[2]);
    None
  });
  cpu!("isNone", |args, hand_mem| {
    let arr = hand_mem.read_fractal(args[0]);
    let val = arr[0].1;
    hand_mem.write_fixed(args[2], if val == 0i64 { 1i64 } else { 0i64 });
    None
  });
  cpu!("getOrM", |args, hand_mem| {
    let arr = hand_mem.read_fractal(args[0]);
    let val = arr[0].1;
    if val == 1i64 {
      hand_mem.register_out(args[0], 1, args[2]);
    } else {
      if args[1] < 0 {
        let val = hand_mem.read_fixed(args[1]);
        hand_mem.write_fixed(args[2], val);
      } else {
        let (data, is_fractal) = hand_mem.read_either(args[1]);
        if is_fractal {
          hand_mem.register(args[2], args[1], true);
        } else {
          hand_mem.write_fixed(args[2], data[0].1);
        }
      }
    }
    None
  });
  cpu!("okR", |args, hand_mem| {
    hand_mem.write_fractal(args[2], &Vec::new());
    hand_mem.push_fixed(args[2], 1i64);
    let val_size = hand_mem.read_fixed(args[1]);
    if val_size == 0 {
      hand_mem.push_register(args[2], args[0]);
    } else {
      let val = hand_mem.read_fixed(args[0]);
      hand_mem.push_fixed(args[2], val);
    }
    None
  });
  cpu!("err", |args, hand_mem| {
    hand_mem.write_fractal(args[2], &Vec::new());
    hand_mem.push_fixed(args[2], 0i64);
    hand_mem.push_register(args[2], args[0]);
    None
  });
  cpu!("isOk", |args, hand_mem| {
    hand_mem.register_out(args[0], 0, args[2]);
    None
  });
  cpu!("isErr", |args, hand_mem| {
    let arr = hand_mem.read_fractal(args[0]);
    let val = arr[0].1;
    hand_mem.write_fixed(args[2], if val == 0i64 { 1i64 } else { 0i64 });
    None
  });
  cpu!("getOrR", |args, hand_mem| {
    let arr = hand_mem.read_fractal(args[0]);
    let val = arr[0].1;
    if val == 1i64 {
      hand_mem.register_out(args[0], 1, args[2]);
    } else {
      let (data, is_fractal) = hand_mem.read_either(args[1]);
      if is_fractal {
        hand_mem.register(args[2], args[1], true);
      } else {
        hand_mem.write_fixed(args[2], data[0].1);
      }
    }
    None
  });
  cpu!("getOrRS", |args, hand_mem| {
    let arr = hand_mem.read_fractal(args[0]);
    let val = arr[0].1;
    if val == 1i64 {
      hand_mem.register_out(args[0], 1, args[2]);
    } else {
      let f = HandlerMemory::str_to_fractal(&HandlerMemory::fractal_to_string(hand_mem.read_fractal(args[1])));
      hand_mem.write_fractal(args[2], &f);
    }
    None
  });
  cpu!("getR", |args, hand_mem| {
    let arr = hand_mem.read_fractal(args[0]);
    let val = arr[0].1;
    if val == 1i64 {
      hand_mem.register_out(args[0], 1, args[2]);
    } else {
      panic!("runtime error: illegal access");
    }
    None
  });
  cpu!("getErr", |args, hand_mem| {
    let arr = hand_mem.read_fractal(args[0]);
    let val = arr[0].1;
    if val == 0i64 {
      hand_mem.register_out(args[0], 1, args[2]);
    } else {
      let (data, is_fractal) = hand_mem.read_either(args[1]);
      if is_fractal {
        hand_mem.register(args[2], args[1], true);
      } else {
        hand_mem.write_fixed(args[2], data[0].1);
      }
    }
    None
  });
  cpu!("resfrom", |args, hand_mem| {
    // args = [arr_addr, arr_idx_addr, outer_addr]
    // a guarded copy of data from an array to a result object
    hand_mem.write_fractal(args[2], &Vec::new());
    let inner_addr = hand_mem.read_fixed(args[1]) as usize;
    let arr = hand_mem.read_fractal(args[0]);
    if arr.len() > inner_addr {
      let (a, b) = arr[inner_addr];
      hand_mem.push_fixed(args[2], 1);
      hand_mem.push_idxs(args[2], a, b as usize);
    } else {
      hand_mem.push_fixed(args[2], 0);
      hand_mem.push_fractal(args[2], &HandlerMemory::str_to_fractal("out-of-bounds access"));
    }
    None
  });
  cpu!("mainE", |args, hand_mem| {
    hand_mem.write_fractal(args[2], &Vec::new());
    hand_mem.push_fixed(args[2], 1i64);
    let val_size = hand_mem.read_fixed(args[1]);
    if val_size == 0 {
      hand_mem.push_register(args[2], args[0]);
    } else {
      let val = hand_mem.read_fixed(args[0]);
      hand_mem.push_fixed(args[2], val);
    }
    None
  });
  cpu!("altE", |args, hand_mem| {
    hand_mem.write_fractal(args[2], &Vec::new());
    hand_mem.push_fixed(args[2], 0i64);
    let val_size = hand_mem.read_fixed(args[1]);
    if val_size == 0 {
      hand_mem.push_register(args[2], args[0]);
    } else {
      let val = hand_mem.read_fixed(args[0]);
      hand_mem.push_fixed(args[2], val);
    }
    None
  });
  cpu!("isMain", |args, hand_mem| {
    hand_mem.register_out(args[0], 0, args[2]);
    None
  });
  cpu!("isAlt", |args, hand_mem| {
    let arr = hand_mem.read_fractal(args[0]);
    let val = arr[0].1;
    hand_mem.write_fixed(args[2], if val == 0i64 { 1i64 } else { 0i64 });
    None
  });
  cpu!("mainOr", |args, hand_mem| {
    let arr = hand_mem.read_fractal(args[0]);
    let val = arr[0].1;
    if val == 1i64 {
      hand_mem.register_out(args[0], 1, args[2]);
    } else {
      let (data, is_fractal) = hand_mem.read_either(args[1]);
      if is_fractal {
        hand_mem.register(args[2], args[1], true);
      } else {
        hand_mem.write_fixed(args[2], data[0].1);
      }
    }
    None
  });
  cpu!("altOr", |args, hand_mem| {
    let arr = hand_mem.read_fractal(args[0]);
    let val = arr[0].1;
    if val == 0i64 {
      hand_mem.register_out(args[0], 1, args[2]);
    } else {
      let (data, is_fractal) = hand_mem.read_either(args[1]);
      if is_fractal {
        hand_mem.register(args[2], args[1], true);
      } else {
        hand_mem.write_fixed(args[2], data[0].1);
      }
    }
    None
  });

  cpu!("hashf", |args, hand_mem| {
    let val = hand_mem.read_fixed(args[0]);
    let mut hasher = XxHash64::with_seed(0xfa57);
    hasher.write_i64(val);
    let out = i64::from_ne_bytes(hasher.finish().to_ne_bytes());
    hand_mem.write_fixed(args[2], out);
    None
  });

  cpu!("hashv", |args, hand_mem| {
    let mut hasher = XxHash64::with_seed(0xfa57);
    let addr = args[0];
    if addr < 0 { // It's a string!
      let pascal_string = hand_mem.read_fractal(args[0]);
      let strlen = pascal_string[0].1 as f64;
      let intlen = 1 + (strlen / 8.0).ceil() as usize;
      for i in 0..intlen {
        hasher.write_i64(pascal_string[i].1);
      }
    } else {
      let mut stack: Vec<Vec<(usize, i64)>> = vec![hand_mem.read_fractal(args[0]).to_vec().clone()];
      while stack.len() > 0 {
        let arr = stack.pop().unwrap();
        let arrlen = arr.len();
        for i in 0..arrlen {
          let (a, b) = arr[i];
          let (data, is_fractal) = hand_mem.read_either_idxs(a, b as usize);
          if is_fractal {
            stack.push(data.clone());
          } else {
            hasher.write_i64(data[0].1);
          }
        }
      }
    }
    let out = i64::from_ne_bytes(hasher.finish().to_ne_bytes());
    hand_mem.write_fixed(args[2], out);
    None
  });


  cpu!("emit", |args, hand_mem| {
    let event = EventEmit {
      id: args[0],
      payload: HandlerMemory::alloc_payload(args[0], args[1], &hand_mem),
    };
    Some(event)
  });
  o
});

impl From<i64> for &ByteOpcode {
  fn from(v: i64) -> Self {
    let opc = OPCODES.get(&v);
    if opc.is_none() {
        panic!(format!("Illegal byte opcode {} ({})", v, str::from_utf8(&v.to_ne_bytes()).unwrap()));
    }
    return &opc.unwrap();
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  #[should_panic]
  fn test_panic_on_invalid_mapping() {
    <&ByteOpcode>::from(100);
  }
}
