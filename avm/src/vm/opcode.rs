use futures::future::{join_all, poll_fn, FutureExt};
use futures::task::{Context, Poll};
use std::collections::HashMap;
use std::convert::Infallible;
use std::fmt::Debug;
use std::future::Future;
use std::hash::Hasher;
use std::io::{self, Write};
use std::net::SocketAddr;
use std::pin::Pin;
use std::str;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use byteorder::{ByteOrder, LittleEndian};
use dashmap::DashMap;
use hyper::header::{HeaderName, HeaderValue};
use hyper::service::{make_service_fn, service_fn};
use hyper::{client::{Client, ResponseFuture}, server::Server, Body, Request, Response, StatusCode};
use hyper_rustls::HttpsConnector;
use once_cell::sync::Lazy;
use rand::RngCore;
use rand::rngs::OsRng;
use regex::Regex;
use tokio::process::Command;
use tokio::time::sleep;
use twox_hash::XxHash64;

use crate::vm::event::{NOP_ID, BuiltInEvents, EventEmit, HandlerFragment};
use crate::vm::memory::{FractalMemory, HandlerMemory, CLOSURE_ARG_MEM_START};
use crate::vm::program::Program;
use crate::vm::run::EVENT_TX;

#[derive(Debug)]
pub struct HttpConfig {
  pub port: u16,
}

#[derive(Debug)]
pub struct HttpsConfig {
  pub port: u16,
  pub priv_key_b64: String,
  pub cert_b64: String,
}

#[derive(Debug)]
pub enum HttpType {
  HTTP(HttpConfig),
  HTTPS(HttpsConfig),
}

static HTTP_RESPONSES: Lazy<Arc<Mutex<HashMap<i64, Arc<HandlerMemory>>>>> =
  Lazy::new(|| Arc::new(Mutex::new(HashMap::<i64, Arc<HandlerMemory>>::new())));

static DS: Lazy<Arc<DashMap<String, Arc<HandlerMemory>>>> =
  Lazy::new(|| Arc::new(DashMap::<String, Arc<HandlerMemory>>::new()));

// type aliases
/// Futures implement an Unpin marker that guarantees to the compiler that the future will not move while it is running
/// so it can be polled. If it is moved, the implementation would be unsafe. We have to manually pin the future because
/// we are creating it dynamically. We must also specify that the `Box`ed Future can be moved across threads with a `+ Send`.
/// For more information see:
/// https://stackoverflow.com/questions/58354633/cannot-use-impl-future-to-store-async-function-in-a-vector
/// https://stackoverflow.com/questions/51485410/unable-to-tokiorun-a-boxed-future-because-the-trait-bound-send-is-not-satisfie
pub type HMFuture = Pin<Box<dyn Future<Output = Arc<HandlerMemory>> + Send>>;

/// A function to be run for an opcode.
pub(crate) enum OpcodeFn {
  Cpu(fn(&[i64], &mut Arc<HandlerMemory>) -> Option<EventEmit>),
  UnpredCpu(fn(Vec<i64>, Arc<HandlerMemory>) -> HMFuture),
  Io(fn(Vec<i64>, Arc<HandlerMemory>) -> HMFuture),
}

impl Debug for OpcodeFn {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      OpcodeFn::Cpu(_) => write!(f, "cpu"),
      OpcodeFn::UnpredCpu(_) => write!(f, "unpred_cpu"),
      OpcodeFn::Io(_) => write!(f, "io"),
    }
  }
}

/// To allow concise definition of opcodes we have a struct that stores all the information
/// about an opcode and how to run it.
/// To define CPU-bound opcodes we use a function pointer type which describes a function whose identity
/// is not necessarily known at compile-time. A closure without context is a function pointer since it can run anywhere.
/// To define IO-bound opcodes it is trickier because `async` fns returns an opaque `impl Future` type so we have to jump through some Rust hoops
/// to be able to define this behaviour
/// For more information see:
/// https://stackoverflow.com/questions/27831944/how-do-i-store-a-closure-in-a-struct-in-rust
/// https://stackoverflow.com/questions/59035366/how-do-i-store-a-variable-of-type-impl-trait-in-a-struct
#[derive(Debug)]
pub struct ByteOpcode {
  /// Opcode value as an i64 number
  pub(crate) _id: i64,
  /// Human readable name for id
  pub(crate) _name: String,
  /// The native code to execute for this opcode
  pub(crate) fun: OpcodeFn,
}

impl ByteOpcode {
  /// There used to be a `pred_exec` field, but that is now dependent on the
  /// kind of `OpcodeFn` that is associated with the opcode, so I made this
  /// inline function to make my life easier when refactoring references :)
  #[inline(always)]
  pub(crate) fn pred_exec(&self) -> bool {
    match self.fun {
      OpcodeFn::Cpu(_) => true,
      OpcodeFn::UnpredCpu(_) => false,
      OpcodeFn::Io(_) => false,
    }
  }
}

pub fn opcode_id(name: &str) -> i64 {
  let mut ascii_name = [b' '; 8];
  // Now insert the new name characters
  for (i, c) in name.chars().take(8).enumerate() {
    ascii_name[i] = c as u8;
  }
  let id = LittleEndian::read_i64(&ascii_name);
  return id;
}

pub static OPCODES: Lazy<HashMap<i64, ByteOpcode>> = Lazy::new(|| {
  let mut o = HashMap::new();

  macro_rules! cpu {
    ($name:ident => fn ($args:ident , $hand_mem:ident) $body:block) => {
      #[allow(non_snake_case)]
      fn $name($args: &[i64], $hand_mem: &mut Arc<HandlerMemory>) -> Option<EventEmit> {
        $body
      }
      let id = opcode_id(stringify!($name));
      let opcode = ByteOpcode {
        _id: id,
        _name: stringify!($name).to_string(),
        fun: OpcodeFn::Cpu($name),
      };
      o.insert(id, opcode);
    };
  }
  macro_rules! unpred_cpu {
    ($name:ident => fn ($args:ident , $hand_mem:ident) $body:block) => {
      #[allow(non_snake_case)]
      fn $name($args: Vec<i64>, $hand_mem: Arc<HandlerMemory>) -> HMFuture {
        $body
      }
      let id = opcode_id(stringify!($name));
      let opcode = ByteOpcode {
        _id: id,
        _name: stringify!($name).to_string(),
        fun: OpcodeFn::UnpredCpu($name),
      };
      o.insert(id, opcode);
    };
    ($name:ident => fn (mut $args:ident , $hand_mem:ident) $body:block) => {
      #[allow(non_snake_case)]
      fn $name(mut $args: Vec<i64>, $hand_mem: Arc<HandlerMemory>) -> HMFuture {
        $body
      }
      let id = opcode_id(stringify!($name));
      let opcode = ByteOpcode {
        _id: id,
        _name: stringify!($name).to_string(),
        fun: OpcodeFn::UnpredCpu($name),
      };
      o.insert(id, opcode);
    };
    ($name:ident => fn ($args:ident , mut $hand_mem:ident) $body:block) => {
      #[allow(non_snake_case)]
      fn $name($args: Vec<i64>, mut $hand_mem: Arc<HandlerMemory>) -> HMFuture {
        $body
      }
      let id = opcode_id(stringify!($name));
      let opcode = ByteOpcode {
        _id: id,
        _name: stringify!($name).to_string(),
        fun: OpcodeFn::UnpredCpu($name),
      };
      o.insert(id, opcode);
    };
    ($name:ident => fn (mut $args:ident , mut $hand_mem:ident) $body:block) => {
      #[allow(non_snake_case)]
      fn $name(mut $args: Vec<i64>, mut $hand_mem: Arc<HandlerMemory>) -> HMFuture {
        $body
      }
      let id = opcode_id(stringify!($name));
      let opcode = ByteOpcode {
        _id: id,
        _name: stringify!($name).to_string(),
        fun: OpcodeFn::UnpredCpu($name),
      };
      o.insert(id, opcode);
    };
  }
  macro_rules! io {
    ($name:ident => fn ($args:ident , $hand_mem:ident) $body:block) => {
      #[allow(non_snake_case)]
      fn $name($args: Vec<i64>, $hand_mem: Arc<HandlerMemory>) -> HMFuture {
        $body
      }
      let id = opcode_id(stringify!($name));
      let opcode = ByteOpcode {
        _id: id,
        _name: stringify!($name).to_string(),
        fun: OpcodeFn::Io($name),
      };
      o.insert(id, opcode);
    };
    ($name:ident => fn (mut $args:ident , $hand_mem:ident) $body:block) => {
      #[allow(non_snake_case)]
      fn $name(mut $args: Vec<i64>, $hand_mem: Arc<HandlerMemory>) -> HMFuture {
        $body
      }
      let id = opcode_id(stringify!($name));
      let opcode = ByteOpcode {
        _id: id,
        _name: stringify!($name).to_string(),
        fun: OpcodeFn::Io($name),
      };
      o.insert(id, opcode);
    };
    ($name:ident => fn ($args:ident , mut $hand_mem:ident) $body:block) => {
      #[allow(non_snake_case)]
      fn $name($args: Vec<i64>, mut $hand_mem: Arc<HandlerMemory>) -> HMFuture {
        $body
      }
      let id = opcode_id(stringify!($name));
      let opcode = ByteOpcode {
        _id: id,
        _name: stringify!($name).to_string(),
        fun: OpcodeFn::Io($name),
      };
      o.insert(id, opcode);
    };
    ($name:ident => fn (mut $args:ident , mut $hand_mem:ident) $body:block) => {
      #[allow(non_snake_case)]
      fn $name(mut $args: Vec<i64>, mut $hand_mem: Arc<HandlerMemory>) -> HMFuture {
        $body
      }
      let id = opcode_id(stringify!($name));
      let opcode = ByteOpcode {
        _id: id,
        _name: stringify!($name).to_string(),
        fun: OpcodeFn::Io($name),
      };
      o.insert(id, opcode);
    };
  }

  // Type conversion opcodes
  cpu!(i8f64 => fn(args, hand_mem) {
    let out = hand_mem.read_fixed(args[0]) as f64;
    hand_mem.write_fixed(args[2], i64::from_ne_bytes(out.to_ne_bytes()));
    None
  });
  cpu!(i16f64 => fn(args, hand_mem) {
    let out = hand_mem.read_fixed(args[0]) as f64;
    hand_mem.write_fixed(args[2], i64::from_ne_bytes(out.to_ne_bytes()));
    None
  });
  cpu!(i32f64 => fn(args, hand_mem) {
    let out = hand_mem.read_fixed(args[0]) as f64;
    hand_mem.write_fixed(args[2], i64::from_ne_bytes(out.to_ne_bytes()));
    None
  });
  cpu!(i64f64 => fn(args, hand_mem) {
    let out = hand_mem.read_fixed(args[0]) as f64;
    hand_mem.write_fixed(args[2], i64::from_ne_bytes(out.to_ne_bytes()));
    None
  });
  cpu!(f32f64 => fn(args, hand_mem) {
    let out = f32::from_ne_bytes((hand_mem.read_fixed(args[0]) as i32).to_ne_bytes());
    hand_mem.write_fixed(args[2], i32::from_ne_bytes(out.to_ne_bytes()) as i64);
    None
  });
  cpu!(strf64 => fn(args, hand_mem) {
    let s = HandlerMemory::fractal_to_string(hand_mem.read_fractal(args[0]));
    let out: f64 = s.parse().unwrap();
    hand_mem.write_fixed(args[2], i64::from_ne_bytes(out.to_ne_bytes()));
    None
  });
  cpu!(boolf64 => fn(args, hand_mem) {
    let out = hand_mem.read_fixed(args[0]) as f64;
    hand_mem.write_fixed(args[2], i64::from_ne_bytes(out.to_ne_bytes()));
    None
  });

  cpu!(i8f32 => fn(args, hand_mem) {
    let num = hand_mem.read_fixed(args[0]) as f32;
    let out = i32::from_ne_bytes(num.to_ne_bytes()) as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!(i16f32 => fn(args, hand_mem) {
    let num = hand_mem.read_fixed(args[0]) as f32;
    let out = i32::from_ne_bytes(num.to_ne_bytes()) as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!(i32f32 => fn(args, hand_mem) {
    let num = hand_mem.read_fixed(args[0]) as f32;
    let out = i32::from_ne_bytes(num.to_ne_bytes()) as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!(i64f32 => fn(args, hand_mem) {
    let num = hand_mem.read_fixed(args[0]) as f32;
    let out = i32::from_ne_bytes(num.to_ne_bytes()) as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!(f64f32 => fn(args, hand_mem) {
    let num = f64::from_ne_bytes(hand_mem.read_fixed(args[0]).to_ne_bytes()) as f32;
    let out = i32::from_ne_bytes(num.to_ne_bytes()) as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!(strf32 => fn(args, hand_mem) {
    let s = HandlerMemory::fractal_to_string(hand_mem.read_fractal(args[0]));
    let num: f32 = s.parse().unwrap();
    let out = i32::from_ne_bytes(num.to_ne_bytes()) as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!(boolf32 => fn(args, hand_mem) {
    let num = hand_mem.read_fixed(args[0]) as f32;
    let out = i32::from_ne_bytes(num.to_ne_bytes()) as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });

  cpu!(i8i64 => fn(args, hand_mem) {
    let out = hand_mem.read_fixed(args[0]);
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!(i16i64 => fn(args, hand_mem) {
    let out = hand_mem.read_fixed(args[0]);
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!(i32i64 => fn(args, hand_mem) {
    let out = hand_mem.read_fixed(args[0]);
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!(f64i64 => fn(args, hand_mem) {
    let out = f64::from_ne_bytes(hand_mem.read_fixed(args[0]).to_ne_bytes()) as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!(f32i64 => fn(args, hand_mem) {
    let out = f32::from_ne_bytes((hand_mem.read_fixed(args[0]) as i32).to_ne_bytes()) as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!(stri64 => fn(args, hand_mem) {
    let s = HandlerMemory::fractal_to_string(hand_mem.read_fractal(args[0]));
    let out: i64 = s.parse().unwrap();
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!(booli64 => fn(args, hand_mem) {
    let out = hand_mem.read_fixed(args[0]);
    hand_mem.write_fixed(args[2], out);
    None
  });

  cpu!(i8i32 => fn(args, hand_mem) {
    let out = hand_mem.read_fixed(args[0]);
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!(i16i32 => fn(args, hand_mem) {
    let out = hand_mem.read_fixed(args[0]);
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!(i64i32 => fn(args, hand_mem) {
    let out = (hand_mem.read_fixed(args[0]) as i32) as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!(f64i32 => fn(args, hand_mem) {
    let out = (f64::from_ne_bytes(hand_mem.read_fixed(args[0]).to_ne_bytes()) as i32) as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!(f32i32 => fn(args, hand_mem) {
    let out = (f32::from_ne_bytes((hand_mem.read_fixed(args[0]) as i32).to_ne_bytes()) as i32) as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!(stri32 => fn(args, hand_mem) {
    let s = HandlerMemory::fractal_to_string(hand_mem.read_fractal(args[0]));
    let num: i32 = s.parse().unwrap();
    let out = num as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!(booli32 => fn(args, hand_mem) {
    let out = hand_mem.read_fixed(args[0]);
    hand_mem.write_fixed(args[2], out);
    None
  });

  cpu!(i8i16 => fn(args, hand_mem) {
    let out = hand_mem.read_fixed(args[0]);
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!(i32i16 => fn(args, hand_mem) {
    let out = (hand_mem.read_fixed(args[0]) as i16) as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!(i64i16 => fn(args, hand_mem) {
    let out = (hand_mem.read_fixed(args[0]) as i16) as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!(f64i16 => fn(args, hand_mem) {
    let out = (f64::from_ne_bytes(hand_mem.read_fixed(args[0]).to_ne_bytes()) as i16) as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!(f32i16 => fn(args, hand_mem) {
    let out = (f32::from_ne_bytes((hand_mem.read_fixed(args[0]) as i32).to_ne_bytes()) as i16) as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!(stri16 => fn(args, hand_mem) {
    let s = HandlerMemory::fractal_to_string(hand_mem.read_fractal(args[0]));
    let num: i16 = s.parse().unwrap();
    let out = num as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!(booli16 => fn(args, hand_mem) {
    let out = hand_mem.read_fixed(args[0]);
    hand_mem.write_fixed(args[2], out);
    None
  });

  cpu!(i16i8 => fn(args, hand_mem) {
    let out = (hand_mem.read_fixed(args[0]) as i8) as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!(i32i8 => fn(args, hand_mem) {
    let out = (hand_mem.read_fixed(args[0]) as i8) as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!(i64i8 => fn(args, hand_mem) {
    let out = (hand_mem.read_fixed(args[0]) as i8) as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!(f64i8 => fn(args, hand_mem) {
    let out = (f64::from_ne_bytes(hand_mem.read_fixed(args[0]).to_ne_bytes()) as i8) as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!(f32i8 => fn(args, hand_mem) {
    let out = (f32::from_ne_bytes((hand_mem.read_fixed(args[0]) as i32).to_ne_bytes()) as i8) as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!(stri8 => fn(args, hand_mem) {
    let s = HandlerMemory::fractal_to_string(hand_mem.read_fractal(args[0]));
    let num: i8 = s.parse().unwrap();
    let out = num as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!(booli8 => fn(args, hand_mem) {
    let out = (hand_mem.read_fixed(args[0]) as i8) as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });

  cpu!(i8bool => fn(args, hand_mem) {
    let a = hand_mem.read_fixed(args[0]) as i8;
    let out = if a != 0 { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!(i16bool => fn(args, hand_mem) {
    let a = hand_mem.read_fixed(args[0]) as i16;
    let out = if a != 0 { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!(i32bool => fn(args, hand_mem) {
    let a = hand_mem.read_fixed(args[0]) as i32;
    let out = if a != 0 { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!(i64bool => fn(args, hand_mem) {
    let a = hand_mem.read_fixed(args[0]);
    let out = if a != 0 { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!(f64bool => fn(args, hand_mem) {
    let a = f64::from_ne_bytes(hand_mem.read_fixed(args[0]).to_ne_bytes());
    let out = if a != 0.0 { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!(f32bool => fn(args, hand_mem) {
    let a = f32::from_ne_bytes((hand_mem.read_fixed(args[0]) as i32).to_ne_bytes());
    let out = if a != 0.0 { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!(strbool => fn(args, hand_mem) {
    let s = HandlerMemory::fractal_to_string(hand_mem.read_fractal(args[0]));
    let out = if s == "true" { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out);
    None
  });

  cpu!(i8str => fn(args, hand_mem) {
    let a = hand_mem.read_fixed(args[0]) as i8;
    let a_str = a.to_string();
    hand_mem.write_fractal(args[2], &HandlerMemory::str_to_fractal(&a_str));
    None
  });
  cpu!(i16str => fn(args, hand_mem) {
    let a = hand_mem.read_fixed(args[0]) as i16;
    let a_str = a.to_string();
    hand_mem.write_fractal(args[2], &HandlerMemory::str_to_fractal(&a_str));
    None
  });
  cpu!(i32str => fn(args, hand_mem) {
    let a = hand_mem.read_fixed(args[0]) as i32;
    let a_str = a.to_string();
    hand_mem.write_fractal(args[2], &HandlerMemory::str_to_fractal(&a_str));
    None
  });
  cpu!(i64str => fn(args, hand_mem) {
    let a = hand_mem.read_fixed(args[0]);
    let a_str = a.to_string();
    hand_mem.write_fractal(args[2], &HandlerMemory::str_to_fractal(&a_str));
    None
  });
  cpu!(f64str => fn(args, hand_mem) {
    let a = f64::from_ne_bytes(hand_mem.read_fixed(args[0]).to_ne_bytes());
    let a_str = a.to_string();
    hand_mem.write_fractal(args[2], &HandlerMemory::str_to_fractal(&a_str));
    None
  });
  cpu!(f32str => fn(args, hand_mem) {
    let a = f32::from_ne_bytes((hand_mem.read_fixed(args[0]) as i32).to_ne_bytes());
    let a_str = a.to_string();
    hand_mem.write_fractal(args[2], &HandlerMemory::str_to_fractal(&a_str));
    None
  });
  cpu!(boolstr => fn(args, hand_mem) {
    let a = hand_mem.read_fixed(args[0]);
    let a_str = if a == 1 { "true" } else { "false" };
    hand_mem.write_fractal(args[2], &HandlerMemory::str_to_fractal(&a_str));
    None
  });

  // Arithmetic opcodes
  cpu!(addi8 => fn(args, hand_mem) {
    let ra = hand_mem.read_fractal(args[0]);
    let rb = hand_mem.read_fractal(args[1]);
    if ra.read_fixed(0) == 0 {
      hand_mem.write_fractal(args[2], &ra);
      return None;
    }
    if rb.read_fixed(0) == 0 {
      hand_mem.write_fractal(args[2], &rb);
      return None;
    }
    let a = ra.read_fixed(1) as i8;
    let b = rb.read_fixed(1) as i8;
    hand_mem.init_fractal(args[2]);
    if a > 0 && b > 0 && a > std::i8::MAX - b {
      hand_mem.push_fixed(args[2], 0);
      hand_mem.push_fractal(args[2], HandlerMemory::str_to_fractal("overflow"));
      return None;
    }
    if a < 0 && b < 0 && a < std::i8::MIN - b {
      hand_mem.push_fixed(args[2], 0);
      hand_mem.push_fractal(args[2], HandlerMemory::str_to_fractal("underflow"));
      return None;
    }
    hand_mem.push_fixed(args[2], 1);
    hand_mem.push_fixed(args[2], (a + b) as i64);
    None
  });
  cpu!(addi16 => fn(args, hand_mem) {
    let ra = hand_mem.read_fractal(args[0]);
    let rb = hand_mem.read_fractal(args[1]);
    if ra.read_fixed(0) == 0 {
      hand_mem.write_fractal(args[2], &ra);
      return None;
    }
    if rb.read_fixed(0) == 0 {
      hand_mem.write_fractal(args[2], &rb);
      return None;
    }
    let a = ra.read_fixed(1) as i16;
    let b = rb.read_fixed(1) as i16;
    hand_mem.init_fractal(args[2]);
    if a > 0 && b > 0 && a > std::i16::MAX - b {
      hand_mem.push_fixed(args[2], 0);
      hand_mem.push_fractal(args[2], HandlerMemory::str_to_fractal("overflow"));
      return None;
    }
    if a < 0 && b < 0 && a < std::i16::MIN - b {
      hand_mem.push_fixed(args[2], 0);
      hand_mem.push_fractal(args[2], HandlerMemory::str_to_fractal("underflow"));
      return None;
    }
    hand_mem.push_fixed(args[2], 1);
    hand_mem.push_fixed(args[2], (a + b) as i64);
    None
  });
  cpu!(addi32 => fn(args, hand_mem) {
    let ra = hand_mem.read_fractal(args[0]);
    let rb = hand_mem.read_fractal(args[1]);
    if ra.read_fixed(0) == 0 {
      hand_mem.write_fractal(args[2], &ra);
      return None;
    }
    if rb.read_fixed(0) == 0 {
      hand_mem.write_fractal(args[2], &rb);
      return None;
    }
    let a = ra.read_fixed(1) as i32;
    let b = rb.read_fixed(1) as i32;
    hand_mem.init_fractal(args[2]);
    if a > 0 && b > 0 && a > std::i32::MAX - b {
      hand_mem.push_fixed(args[2], 0);
      hand_mem.push_fractal(args[2], HandlerMemory::str_to_fractal("overflow"));
      return None;
    }
    if a < 0 && b < 0 && a < std::i32::MIN - b {
      hand_mem.push_fixed(args[2], 0);
      hand_mem.push_fractal(args[2], HandlerMemory::str_to_fractal("underflow"));
      return None;
    }
    hand_mem.push_fixed(args[2], 1);
    hand_mem.push_fixed(args[2], (a + b) as i64);
    None
  });
  cpu!(addi64 => fn(args, hand_mem) {
    let ra = hand_mem.read_fractal(args[0]);
    let rb = hand_mem.read_fractal(args[1]);
    if ra.read_fixed(0) == 0 {
      hand_mem.write_fractal(args[2], &ra);
      return None;
    }
    if rb.read_fixed(0) == 0 {
      hand_mem.write_fractal(args[2], &rb);
      return None;
    }
    let a = ra.read_fixed(1) as i64;
    let b = rb.read_fixed(1) as i64;
    hand_mem.init_fractal(args[2]);
    if a > 0 && b > 0 && a > std::i64::MAX - b {
      hand_mem.push_fixed(args[2], 0);
      hand_mem.push_fractal(args[2], HandlerMemory::str_to_fractal("overflow"));
      return None;
    }
    if a < 0 && b < 0 && a < std::i64::MIN - b {
      hand_mem.push_fixed(args[2], 0);
      hand_mem.push_fractal(args[2], HandlerMemory::str_to_fractal("underflow"));
      return None;
    }
    hand_mem.push_fixed(args[2], 1);
    hand_mem.push_fixed(args[2], (a + b) as i64);
    None
  });
  cpu!(addf32 => fn(args, hand_mem) {
    let ra = hand_mem.read_fractal(args[0]);
    let rb = hand_mem.read_fractal(args[1]);
    if ra.read_fixed(0) == 0 {
      hand_mem.write_fractal(args[2], &ra);
      return None;
    }
    if rb.read_fixed(0) == 0 {
      hand_mem.write_fractal(args[2], &rb);
      return None;
    }
    let a = f32::from_ne_bytes((ra.read_fixed(1) as i32).to_ne_bytes());
    let b = f32::from_ne_bytes((rb.read_fixed(1) as i32).to_ne_bytes());
    let out = a + b;
    hand_mem.init_fractal(args[2]);
    if out == std::f32::INFINITY {
      hand_mem.push_fixed(args[2], 0);
      hand_mem.push_fractal(args[2], HandlerMemory::str_to_fractal("overflow"));
      return None;
    }
    if out == std::f32::NEG_INFINITY {
      hand_mem.push_fixed(args[2], 0);
      hand_mem.push_fractal(args[2], HandlerMemory::str_to_fractal("underflow"));
      return None;
    }
    let num = i32::from_ne_bytes(out.to_ne_bytes()) as i64;
    hand_mem.push_fixed(args[2], 1);
    hand_mem.push_fixed(args[2], num);
    None
  });
  cpu!(addf64 => fn(args, hand_mem) {
    let ra = hand_mem.read_fractal(args[0]);
    let rb = hand_mem.read_fractal(args[1]);
    if ra.read_fixed(0) == 0 {
      hand_mem.write_fractal(args[2], &ra);
      return None;
    }
    if rb.read_fixed(0) == 0 {
      hand_mem.write_fractal(args[2], &rb);
      return None;
    }
    let a = f64::from_ne_bytes(ra.read_fixed(1).to_ne_bytes());
    let b = f64::from_ne_bytes(rb.read_fixed(1).to_ne_bytes());
    let out = a + b;
    hand_mem.init_fractal(args[2]);
    if out == std::f64::INFINITY {
      hand_mem.push_fixed(args[2], 0);
      hand_mem.push_fractal(args[2], HandlerMemory::str_to_fractal("overflow"));
      return None;
    }
    if out == std::f64::NEG_INFINITY {
      hand_mem.push_fixed(args[2], 0);
      hand_mem.push_fractal(args[2], HandlerMemory::str_to_fractal("underflow"));
      return None;
    }
    let num = i64::from_ne_bytes(out.to_ne_bytes());
    hand_mem.push_fixed(args[2], 1);
    hand_mem.push_fixed(args[2], num);
    None
  });

  cpu!(subi8 => fn(args, hand_mem) {
    let ra = hand_mem.read_fractal(args[0]);
    let rb = hand_mem.read_fractal(args[1]);
    if ra.read_fixed(0) == 0 {
      hand_mem.write_fractal(args[2], &ra);
      return None;
    }
    if rb.read_fixed(0) == 0 {
      hand_mem.write_fractal(args[2], &rb);
      return None;
    }
    let a = ra.read_fixed(1) as i8;
    let b = rb.read_fixed(1) as i8;
    hand_mem.init_fractal(args[2]);
    if a > 0 && b < 0 && a > std::i8::MAX + b {
      hand_mem.push_fixed(args[2], 0);
      hand_mem.push_fractal(args[2], HandlerMemory::str_to_fractal("overflow"));
      return None;
    }
    if a < 0 && b > 0 && a < std::i8::MIN + b {
      hand_mem.push_fixed(args[2], 0);
      hand_mem.push_fractal(args[2], HandlerMemory::str_to_fractal("underflow"));
      return None;
    }
    hand_mem.push_fixed(args[2], 1);
    hand_mem.push_fixed(args[2], (a - b) as i64);
    None
  });
  cpu!(subi16 => fn(args, hand_mem) {
    let ra = hand_mem.read_fractal(args[0]);
    let rb = hand_mem.read_fractal(args[1]);
    if ra.read_fixed(0) == 0 {
      hand_mem.write_fractal(args[2], &ra);
      return None;
    }
    if rb.read_fixed(0) == 0 {
      hand_mem.write_fractal(args[2], &rb);
      return None;
    }
    let a = ra.read_fixed(1) as i16;
    let b = rb.read_fixed(1) as i16;
    hand_mem.init_fractal(args[2]);
    if a > 0 && b < 0 && a > std::i16::MAX + b {
      hand_mem.push_fixed(args[2], 0);
      hand_mem.push_fractal(args[2], HandlerMemory::str_to_fractal("overflow"));
      return None;
    }
    if a < 0 && b > 0 && a < std::i16::MIN + b {
      hand_mem.push_fixed(args[2], 0);
      hand_mem.push_fractal(args[2], HandlerMemory::str_to_fractal("underflow"));
      return None;
    }
    hand_mem.push_fixed(args[2], 1);
    hand_mem.push_fixed(args[2], (a - b) as i64);
    None
  });
  cpu!(subi32 => fn(args, hand_mem) {
    let ra = hand_mem.read_fractal(args[0]);
    let rb = hand_mem.read_fractal(args[1]);
    if ra.read_fixed(0) == 0 {
      hand_mem.write_fractal(args[2], &ra);
      return None;
    }
    if rb.read_fixed(0) == 0 {
      hand_mem.write_fractal(args[2], &rb);
      return None;
    }
    let a = ra.read_fixed(1) as i32;
    let b = rb.read_fixed(1) as i32;
    hand_mem.init_fractal(args[2]);
    if a > 0 && b < 0 && a > std::i32::MAX + b {
      hand_mem.push_fixed(args[2], 0);
      hand_mem.push_fractal(args[2], HandlerMemory::str_to_fractal("overflow"));
      return None;
    }
    if a < 0 && b > 0 && a < std::i32::MIN + b {
      hand_mem.push_fixed(args[2], 0);
      hand_mem.push_fractal(args[2], HandlerMemory::str_to_fractal("underflow"));
      return None;
    }
    hand_mem.push_fixed(args[2], 1);
    hand_mem.push_fixed(args[2], (a - b) as i64);
    None
  });
  cpu!(subi64 => fn(args, hand_mem) {
    let ra = hand_mem.read_fractal(args[0]);
    let rb = hand_mem.read_fractal(args[1]);
    if ra.read_fixed(0) == 0 {
      hand_mem.write_fractal(args[2], &ra);
      return None;
    }
    if rb.read_fixed(0) == 0 {
      hand_mem.write_fractal(args[2], &rb);
      return None;
    }
    let a = ra.read_fixed(1) as i64;
    let b = rb.read_fixed(1) as i64;
    hand_mem.init_fractal(args[2]);
    if a > 0 && b < 0 && a > std::i64::MAX + b {
      hand_mem.push_fixed(args[2], 0);
      hand_mem.push_fractal(args[2], HandlerMemory::str_to_fractal("overflow"));
      return None;
    }
    if a < 0 && b > 0 && a < std::i64::MIN + b {
      hand_mem.push_fixed(args[2], 0);
      hand_mem.push_fractal(args[2], HandlerMemory::str_to_fractal("underflow"));
      return None;
    }
    hand_mem.push_fixed(args[2], 1);
    hand_mem.push_fixed(args[2], a - b);
    None
  });
  cpu!(subf32 => fn(args, hand_mem) {
    let ra = hand_mem.read_fractal(args[0]);
    let rb = hand_mem.read_fractal(args[1]);
    if ra.read_fixed(0) == 0 {
      hand_mem.write_fractal(args[2], &ra);
      return None;
    }
    if rb.read_fixed(0) == 0 {
      hand_mem.write_fractal(args[2], &rb);
      return None;
    }
    let a = f32::from_ne_bytes((ra.read_fixed(1) as i32).to_ne_bytes());
    let b = f32::from_ne_bytes((rb.read_fixed(1) as i32).to_ne_bytes());
    let out = a - b;
    hand_mem.init_fractal(args[2]);
    if out == std::f32::INFINITY {
      hand_mem.push_fixed(args[2], 0);
      hand_mem.push_fractal(args[2], HandlerMemory::str_to_fractal("overflow"));
      return None;
    }
    if out == std::f32::NEG_INFINITY {
      hand_mem.push_fixed(args[2], 0);
      hand_mem.push_fractal(args[2], HandlerMemory::str_to_fractal("underflow"));
      return None;
    }
    let num = i32::from_ne_bytes(out.to_ne_bytes()) as i64;
    hand_mem.push_fixed(args[2], 1);
    hand_mem.push_fixed(args[2], num);
    None
  });
  cpu!(subf64 => fn(args, hand_mem) {
    let ra = hand_mem.read_fractal(args[0]);
    let rb = hand_mem.read_fractal(args[1]);
    if ra.read_fixed(0) == 0 {
      hand_mem.write_fractal(args[2], &ra);
      return None;
    }
    if rb.read_fixed(0) == 0 {
      hand_mem.write_fractal(args[2], &rb);
      return None;
    }
    let a = f64::from_ne_bytes(ra.read_fixed(1).to_ne_bytes());
    let b = f64::from_ne_bytes(rb.read_fixed(1).to_ne_bytes());
    let out = a - b;
    hand_mem.init_fractal(args[2]);
    if out == std::f64::INFINITY {
      hand_mem.push_fixed(args[2], 0);
      hand_mem.push_fractal(args[2], HandlerMemory::str_to_fractal("overflow"));
      return None;
    }
    if out == std::f64::NEG_INFINITY {
      hand_mem.push_fixed(args[2], 0);
      hand_mem.push_fractal(args[2], HandlerMemory::str_to_fractal("underflow"));
      return None;
    }
    let num = i64::from_ne_bytes(out.to_ne_bytes());
    hand_mem.push_fixed(args[2], 1);
    hand_mem.push_fixed(args[2], num);
    None
  });

  cpu!(negi8 => fn(args, hand_mem) {
    let a = hand_mem.read_fixed(args[0]) as i8;
    let out = (0 - a) as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!(negi16 => fn(args, hand_mem) {
    let a = hand_mem.read_fixed(args[0]) as i16;
    let out = (0 - a) as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!(negi32 => fn(args, hand_mem) {
    let a = hand_mem.read_fixed(args[0]) as i32;
    let out = (0 - a) as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!(negi64 => fn(args, hand_mem) {
    let a = hand_mem.read_fixed(args[0]);
    let out = 0 - a;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!(negf32 => fn(args, hand_mem) {
    let a = f32::from_ne_bytes((hand_mem.read_fixed(args[0]) as i32).to_ne_bytes());
    let out = i32::from_ne_bytes((0.0 - a).to_ne_bytes()) as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!(negf64 => fn(args, hand_mem) {
    let a = f64::from_ne_bytes(hand_mem.read_fixed(args[0]).to_ne_bytes());
    let out = i64::from_ne_bytes((0.0 - a).to_ne_bytes());
    hand_mem.write_fixed(args[2], out);
    None
  });

  cpu!(absi8 => fn(args, hand_mem) {
    let a = hand_mem.read_fixed(args[0]) as i8;
    let out = a.abs() as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!(absi16 => fn(args, hand_mem) {
    let a = hand_mem.read_fixed(args[0]) as i16;
    let out = a.abs() as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!(absi32 => fn(args, hand_mem) {
    let a = hand_mem.read_fixed(args[0]) as i32;
    let out = a.abs() as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!(absi64 => fn(args, hand_mem) {
    let a = hand_mem.read_fixed(args[0]);
    let out = a.abs();
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!(absf32 => fn(args, hand_mem) {
    let a = f32::from_ne_bytes((hand_mem.read_fixed(args[0]) as i32).to_ne_bytes());
    let out = i32::from_ne_bytes(a.abs().to_ne_bytes()) as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!(absf64 => fn(args, hand_mem) {
    let a = f64::from_ne_bytes(hand_mem.read_fixed(args[0]).to_ne_bytes());
    let out = i64::from_ne_bytes(a.abs().to_ne_bytes());
    hand_mem.write_fixed(args[2], out);
    None
  });

  cpu!(muli8 => fn(args, hand_mem) {
    let ra = hand_mem.read_fractal(args[0]);
    let rb = hand_mem.read_fractal(args[1]);
    if ra.read_fixed(0) == 0 {
      hand_mem.write_fractal(args[2], &ra);
      return None;
    }
    if rb.read_fixed(0) == 0 {
      hand_mem.write_fractal(args[2], &rb);
      return None;
    }
    let a = ra.read_fixed(1) as i8;
    let b = rb.read_fixed(1) as i8;
    hand_mem.init_fractal(args[2]);
    if a > 0 && b > 0 && (a as f64) > (std::i8::MAX as f64) / (b as f64) {
      hand_mem.push_fixed(args[2], 0);
      hand_mem.push_fractal(args[2], HandlerMemory::str_to_fractal("overflow"));
      return None;
    }
    if a < 0 && b < 0 && (a as f64) < (std::i8::MIN as f64) / (b as f64) {
      hand_mem.push_fixed(args[2], 0);
      hand_mem.push_fractal(args[2], HandlerMemory::str_to_fractal("underflow"));
      return None;
    }
    hand_mem.push_fixed(args[2], 1);
    hand_mem.push_fixed(args[2], (a * b) as i64);
    None
  });
  cpu!(muli16 => fn(args, hand_mem) {
    let ra = hand_mem.read_fractal(args[0]);
    let rb = hand_mem.read_fractal(args[1]);
    if ra.read_fixed(0) == 0 {
      hand_mem.write_fractal(args[2], &ra);
      return None;
    }
    if rb.read_fixed(0) == 0 {
      hand_mem.write_fractal(args[2], &rb);
      return None;
    }
    let a = ra.read_fixed(1) as i16;
    let b = rb.read_fixed(1) as i16;
    hand_mem.init_fractal(args[2]);
    if a > 0 && b > 0 && (a as f64) > (std::i16::MAX as f64) / (b as f64) {
      hand_mem.push_fixed(args[2], 0);
      hand_mem.push_fractal(args[2], HandlerMemory::str_to_fractal("overflow"));
      return None;
    }
    if a < 0 && b < 0 && (a as f64) < (std::i16::MIN as f64) / (b as f64) {
      hand_mem.push_fixed(args[2], 0);
      hand_mem.push_fractal(args[2], HandlerMemory::str_to_fractal("underflow"));
      return None;
    }
    hand_mem.push_fixed(args[2], 1);
    hand_mem.push_fixed(args[2], (a * b) as i64);
    None
  });
  cpu!(muli32 => fn(args, hand_mem) {
    let ra = hand_mem.read_fractal(args[0]);
    let rb = hand_mem.read_fractal(args[1]);
    if ra.read_fixed(0) == 0 {
      hand_mem.write_fractal(args[2], &ra);
      return None;
    }
    if rb.read_fixed(0) == 0 {
      hand_mem.write_fractal(args[2], &rb);
      return None;
    }
    let a = ra.read_fixed(1) as i32;
    let b = rb.read_fixed(1) as i32;
    hand_mem.init_fractal(args[2]);
    if a > 0 && b > 0 && (a as f64) > (std::i32::MAX as f64) / (b as f64) {
      hand_mem.push_fixed(args[2], 0);
      hand_mem.push_fractal(args[2], HandlerMemory::str_to_fractal("overflow"));
      return None;
    }
    if a < 0 && b < 0 && (a as f64) < (std::i32::MIN as f64) / (b as f64) {
      hand_mem.push_fixed(args[2], 0);
      hand_mem.push_fractal(args[2], HandlerMemory::str_to_fractal("underflow"));
      return None;
    }
    hand_mem.push_fixed(args[2], 1);
    hand_mem.push_fixed(args[2], (a * b) as i64);
    None
  });
  cpu!(muli64 => fn(args, hand_mem) {
    let ra = hand_mem.read_fractal(args[0]);
    let rb = hand_mem.read_fractal(args[1]);
    if ra.read_fixed(0) == 0 {
      hand_mem.write_fractal(args[2], &ra);
      return None;
    }
    if rb.read_fixed(0) == 0 {
      hand_mem.write_fractal(args[2], &rb);
      return None;
    }
    let a = ra.read_fixed(1) as i64;
    let b = rb.read_fixed(1) as i64;
    hand_mem.init_fractal(args[2]);
    if a > 0 && b > 0 && (a as f64) > (std::i64::MAX as f64) / (b as f64) {
      hand_mem.push_fixed(args[2], 0);
      hand_mem.push_fractal(args[2], HandlerMemory::str_to_fractal("overflow"));
      return None;
    }
    if a < 0 && b < 0 && (a as f64) < (std::i64::MIN as f64) / (b as f64) {
      hand_mem.push_fixed(args[2], 0);
      hand_mem.push_fractal(args[2], HandlerMemory::str_to_fractal("underflow"));
      return None;
    }
    hand_mem.push_fixed(args[2], 1);
    hand_mem.push_fixed(args[2], a * b);
    None
  });
  cpu!(mulf32 => fn(args, hand_mem) {
    let ra = hand_mem.read_fractal(args[0]);
    let rb = hand_mem.read_fractal(args[1]);
    if ra.read_fixed(0) == 0 {
      hand_mem.write_fractal(args[2], &ra);
      return None;
    }
    if rb.read_fixed(0) == 0 {
      hand_mem.write_fractal(args[2], &rb);
      return None;
    }
    let a = f32::from_ne_bytes((ra.read_fixed(1) as i32).to_ne_bytes());
    let b = f32::from_ne_bytes((rb.read_fixed(1) as i32).to_ne_bytes());
    let out = a * b;
    hand_mem.init_fractal(args[2]);
    if out == std::f32::INFINITY {
      hand_mem.push_fixed(args[2], 0);
      hand_mem.push_fractal(args[2], HandlerMemory::str_to_fractal("overflow"));
      return None;
    }
    if out == std::f32::NEG_INFINITY {
      hand_mem.push_fixed(args[2], 0);
      hand_mem.push_fractal(args[2], HandlerMemory::str_to_fractal("underflow"));
      return None;
    }
    let num = i32::from_ne_bytes(out.to_ne_bytes()) as i64;
    hand_mem.push_fixed(args[2], 1);
    hand_mem.push_fixed(args[2], num);
    None
  });
  cpu!(mulf64 => fn(args, hand_mem) {
    let ra = hand_mem.read_fractal(args[0]);
    let rb = hand_mem.read_fractal(args[1]);
    if ra.read_fixed(0) == 0 {
      hand_mem.write_fractal(args[2], &ra);
      return None;
    }
    if rb.read_fixed(0) == 0 {
      hand_mem.write_fractal(args[2], &rb);
      return None;
    }
    let a = f64::from_ne_bytes(ra.read_fixed(1).to_ne_bytes());
    let b = f64::from_ne_bytes(rb.read_fixed(1).to_ne_bytes());
    let out = a * b;
    hand_mem.init_fractal(args[2]);
    if out == std::f64::INFINITY {
      hand_mem.push_fixed(args[2], 0);
      hand_mem.push_fractal(args[2], HandlerMemory::str_to_fractal("overflow"));
      return None;
    }
    if out == std::f64::NEG_INFINITY {
      hand_mem.push_fixed(args[2], 0);
      hand_mem.push_fractal(args[2], HandlerMemory::str_to_fractal("underflow"));
      return None;
    }
    let num = i64::from_ne_bytes(out.to_ne_bytes());
    hand_mem.push_fixed(args[2], 1);
    hand_mem.push_fixed(args[2], num);
    None
  });

  cpu!(divi8 => fn(args, hand_mem) {
    let ra = hand_mem.read_fractal(args[0]);
    let rb = hand_mem.read_fractal(args[1]);
    if ra.read_fixed(0) == 0 {
      hand_mem.write_fractal(args[2], &ra);
      return None;
    }
    if rb.read_fixed(0) == 0 {
      hand_mem.write_fractal(args[2], &rb);
      return None;
    }
    let a = ra.read_fixed(1) as i8;
    let b = rb.read_fixed(1) as i8;
    hand_mem.init_fractal(args[2]);
    if b == 0 {
      hand_mem.push_fixed(args[2], 0);
      hand_mem.push_fractal(args[2], HandlerMemory::str_to_fractal("divide-by-zero"));
      return None;
    }
    hand_mem.push_fixed(args[2], 1);
    hand_mem.push_fixed(args[2], (a / b) as i64);
    None
  });
  cpu!(divi16 => fn(args, hand_mem) {
    let ra = hand_mem.read_fractal(args[0]);
    let rb = hand_mem.read_fractal(args[1]);
    if ra.read_fixed(0) == 0 {
      hand_mem.write_fractal(args[2], &ra);
      return None;
    }
    if rb.read_fixed(0) == 0 {
      hand_mem.write_fractal(args[2], &rb);
      return None;
    }
    let a = ra.read_fixed(1) as i16;
    let b = rb.read_fixed(1) as i16;
    hand_mem.init_fractal(args[2]);
    if b == 0 {
      hand_mem.push_fixed(args[2], 0);
      hand_mem.push_fractal(args[2], HandlerMemory::str_to_fractal("divide-by-zero"));
      return None;
    }
    hand_mem.push_fixed(args[2], 1);
    hand_mem.push_fixed(args[2], (a / b) as i64);
    None
  });
  cpu!(divi32 => fn(args, hand_mem) {
    let ra = hand_mem.read_fractal(args[0]);
    let rb = hand_mem.read_fractal(args[1]);
    if ra.read_fixed(0) == 0 {
      hand_mem.write_fractal(args[2], &ra);
      return None;
    }
    if rb.read_fixed(0) == 0 {
      hand_mem.write_fractal(args[2], &rb);
      return None;
    }
    let a = ra.read_fixed(1) as i32;
    let b = rb.read_fixed(1) as i32;
    hand_mem.init_fractal(args[2]);
    if b == 0 {
      hand_mem.push_fixed(args[2], 0);
      hand_mem.push_fractal(args[2], HandlerMemory::str_to_fractal("divide-by-zero"));
      return None;
    }
    hand_mem.push_fixed(args[2], 1);
    hand_mem.push_fixed(args[2], (a / b) as i64);
    None
  });
  cpu!(divi64 => fn(args, hand_mem) {
    let ra = hand_mem.read_fractal(args[0]);
    let rb = hand_mem.read_fractal(args[1]);
    if ra.read_fixed(0) == 0 {
      hand_mem.write_fractal(args[2], &ra);
      return None;
    }
    if rb.read_fixed(0) == 0 {
      hand_mem.write_fractal(args[2], &rb);
      return None;
    }
    let a = ra.read_fixed(1) as i64;
    let b = rb.read_fixed(1) as i64;
    hand_mem.init_fractal(args[2]);
    if b == 0 {
      hand_mem.push_fixed(args[2], 0);
      hand_mem.push_fractal(args[2], HandlerMemory::str_to_fractal("divide-by-zero"));
      return None;
    }
    hand_mem.push_fixed(args[2], 1);
    hand_mem.push_fixed(args[2], a / b);
    None
  });
  cpu!(divf32 => fn(args, hand_mem) {
    let ra = hand_mem.read_fractal(args[0]);
    let rb = hand_mem.read_fractal(args[1]);
    if ra.read_fixed(0) == 0 {
      hand_mem.write_fractal(args[2], &ra);
      return None;
    }
    if rb.read_fixed(0) == 0 {
      hand_mem.write_fractal(args[2], &rb);
      return None;
    }
    let a = f32::from_ne_bytes((ra.read_fixed(1) as i32).to_ne_bytes());
    let b = f32::from_ne_bytes((rb.read_fixed(1) as i32).to_ne_bytes());
    hand_mem.init_fractal(args[2]);
    if b == 0.0 {
      hand_mem.push_fixed(args[2], 0);
      hand_mem.push_fractal(args[2], HandlerMemory::str_to_fractal("divide-by-zero"));
      return None;
    }
    let out = a / b;
    if out == std::f32::INFINITY {
      hand_mem.push_fixed(args[2], 0);
      hand_mem.push_fractal(args[2], HandlerMemory::str_to_fractal("overflow"));
      return None;
    }
    if out == std::f32::NEG_INFINITY {
      hand_mem.push_fixed(args[2], 0);
      hand_mem.push_fractal(args[2], HandlerMemory::str_to_fractal("underflow"));
      return None;
    }
    let num = i32::from_ne_bytes(out.to_ne_bytes()) as i64;
    hand_mem.push_fixed(args[2], 1);
    hand_mem.push_fixed(args[2], num);
    None
  });
  cpu!(divf64 => fn(args, hand_mem) {
    let ra = hand_mem.read_fractal(args[0]);
    let rb = hand_mem.read_fractal(args[1]);
    if ra.read_fixed(0) == 0 {
      hand_mem.write_fractal(args[2], &ra);
      return None;
    }
    if rb.read_fixed(0) == 0 {
      hand_mem.write_fractal(args[2], &rb);
      return None;
    }
    let a = f64::from_ne_bytes(ra.read_fixed(1).to_ne_bytes());
    let b = f64::from_ne_bytes(rb.read_fixed(1).to_ne_bytes());
    hand_mem.init_fractal(args[2]);
    if b == 0.0 {
      hand_mem.push_fixed(args[2], 0);
      hand_mem.push_fractal(args[2], HandlerMemory::str_to_fractal("divide-by-zero"));
      return None;
    }
    let out = a / b;
    if out == std::f64::INFINITY {
      hand_mem.push_fixed(args[2], 0);
      hand_mem.push_fractal(args[2], HandlerMemory::str_to_fractal("overflow"));
      return None;
    }
    if out == std::f64::NEG_INFINITY {
      hand_mem.push_fixed(args[2], 0);
      hand_mem.push_fractal(args[2], HandlerMemory::str_to_fractal("underflow"));
      return None;
    }
    let num = i64::from_ne_bytes(out.to_ne_bytes());
    hand_mem.push_fixed(args[2], 1);
    hand_mem.push_fixed(args[2], num);
    None
  });

  cpu!(modi8 => fn(args, hand_mem) {
    let a = hand_mem.read_fixed(args[0]) as i8;
    let b = hand_mem.read_fixed(args[1]) as i8;
    let out = (a % b) as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!(modi16 => fn(args, hand_mem) {
    let a = hand_mem.read_fixed(args[0]) as i16;
    let b = hand_mem.read_fixed(args[1]) as i16;
    let out = (a % b) as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!(modi32 => fn(args, hand_mem) {
    let a = hand_mem.read_fixed(args[0]) as i32;
    let b = hand_mem.read_fixed(args[1]) as i32;
    let out = (a % b) as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!(modi64 => fn(args, hand_mem) {
    let a = hand_mem.read_fixed(args[0]);
    let b = hand_mem.read_fixed(args[1]);
    let out = a % b;
    hand_mem.write_fixed(args[2], out);
    None
  });

  cpu!(powi8 => fn(args, hand_mem) {
    let ra = hand_mem.read_fractal(args[0]);
    let rb = hand_mem.read_fractal(args[1]);
    if ra.read_fixed(0) == 0 {
      hand_mem.write_fractal(args[2], &ra);
      return None;
    }
    if rb.read_fixed(0) == 0 {
      hand_mem.write_fractal(args[2], &rb);
      return None;
    }
    let a = ra.read_fixed(1) as i8;
    let b = rb.read_fixed(1) as i8;
    hand_mem.init_fractal(args[2]);
    if a > 0 && b > 1 && (a as f64) > f64::powf(std::i8::MAX as f64, 1.0 / (b as f64)) {
      hand_mem.push_fixed(args[2], 0);
      hand_mem.push_fractal(args[2], HandlerMemory::str_to_fractal("overflow"));
      return None;
    }
    if a < 0 && b > 1 && (a as f64) < f64::powf(std::i8::MIN as f64, 1.0 / (b as f64)) {
      hand_mem.push_fixed(args[2], 0);
      hand_mem.push_fractal(args[2], HandlerMemory::str_to_fractal("underflow"));
      return None;
    }
    hand_mem.push_fixed(args[2], 1);
    let out = if b < 0 { 0i64 } else { i8::pow(a, b as u32) as i64 };
    hand_mem.push_fixed(args[2], out);
    None
  });
  cpu!(powi16 => fn(args, hand_mem) {
    let ra = hand_mem.read_fractal(args[0]);
    let rb = hand_mem.read_fractal(args[1]);
    if ra.read_fixed(0) == 0 {
      hand_mem.write_fractal(args[2], &ra);
      return None;
    }
    if rb.read_fixed(0) == 0 {
      hand_mem.write_fractal(args[2], &rb);
      return None;
    }
    let a = ra.read_fixed(1) as i16;
    let b = rb.read_fixed(1) as i16;
    hand_mem.init_fractal(args[2]);
    if a > 0 && b > 1 && (a as f64) > f64::powf(std::i16::MAX as f64, 1.0 / (b as f64)) {
      hand_mem.push_fixed(args[2], 0);
      hand_mem.push_fractal(args[2], HandlerMemory::str_to_fractal("overflow"));
      return None;
    }
    if a < 0 && b > 1 && (a as f64) < f64::powf(std::i16::MIN as f64, 1.0 / (b as f64)) {
      hand_mem.push_fixed(args[2], 0);
      hand_mem.push_fractal(args[2], HandlerMemory::str_to_fractal("underflow"));
      return None;
    }
    hand_mem.push_fixed(args[2], 1);
    let out = if b < 0 { 0i64 } else { i16::pow(a, b as u32) as i64 };
    hand_mem.push_fixed(args[2], out);
    None
  });
  cpu!(powi32 => fn(args, hand_mem) {
    let ra = hand_mem.read_fractal(args[0]);
    let rb = hand_mem.read_fractal(args[1]);
    if ra.read_fixed(0) == 0 {
      hand_mem.write_fractal(args[2], &ra);
      return None;
    }
    if rb.read_fixed(0) == 0 {
      hand_mem.write_fractal(args[2], &rb);
      return None;
    }
    let a = ra.read_fixed(1) as i32;
    let b = rb.read_fixed(1) as i32;
    hand_mem.init_fractal(args[2]);
    if a > 0 && b > 1 && (a as f64) > f64::powf(std::i32::MAX as f64, 1.0 / (b as f64)) {
      hand_mem.push_fixed(args[2], 0);
      hand_mem.push_fractal(args[2], HandlerMemory::str_to_fractal("overflow"));
      return None;
    }
    if a < 0 && b > 1 && (a as f64) < f64::powf(std::i32::MIN as f64, 1.0 / (b as f64)) {
      hand_mem.push_fixed(args[2], 0);
      hand_mem.push_fractal(args[2], HandlerMemory::str_to_fractal("underflow"));
      return None;
    }
    hand_mem.push_fixed(args[2], 1);
    let out = if b < 0 { 0i64 } else { i32::pow(a, b as u32) as i64 };
    hand_mem.push_fixed(args[2], out);
    None
  });
  cpu!(powi64 => fn(args, hand_mem) {
    let ra = hand_mem.read_fractal(args[0]);
    let rb = hand_mem.read_fractal(args[1]);
    if ra.read_fixed(0) == 0 {
      hand_mem.write_fractal(args[2], &ra);
      return None;
    }
    if rb.read_fixed(0) == 0 {
      hand_mem.write_fractal(args[2], &rb);
      return None;
    }
    let a = ra.read_fixed(1) as i64;
    let b = rb.read_fixed(1) as i64;
    hand_mem.init_fractal(args[2]);
    if a > 0 && b > 1 && (a as f64) > f64::powf(std::i64::MAX as f64, 1.0 / (b as f64)) {
      hand_mem.push_fixed(args[2], 0);
      hand_mem.push_fractal(args[2], HandlerMemory::str_to_fractal("overflow"));
      return None;
    }
    if a < 0 && b > 1 && (a as f64) < f64::powf(std::i64::MIN as f64, 1.0 / (b as f64)) {
      hand_mem.push_fixed(args[2], 0);
      hand_mem.push_fractal(args[2], HandlerMemory::str_to_fractal("underflow"));
      return None;
    }
    hand_mem.push_fixed(args[2], 1);
    let out = if b < 0 { 0i64 } else { i64::pow(a, b as u32) as i64 };
    hand_mem.push_fixed(args[2], out);
    None
  });
  cpu!(powf32 => fn(args, hand_mem) {
    let ra = hand_mem.read_fractal(args[0]);
    let rb = hand_mem.read_fractal(args[1]);
    if ra.read_fixed(0) == 0 {
      hand_mem.write_fractal(args[2], &ra);
      return None;
    }
    if rb.read_fixed(0) == 0 {
      hand_mem.write_fractal(args[2], &rb);
      return None;
    }
    let a = f32::from_ne_bytes((ra.read_fixed(1) as i32).to_ne_bytes());
    let b = f32::from_ne_bytes((rb.read_fixed(1) as i32).to_ne_bytes());
    let out = f32::powf(a, b);
    hand_mem.init_fractal(args[2]);
    if out == std::f32::INFINITY {
      hand_mem.push_fixed(args[2], 0);
      hand_mem.push_fractal(args[2], HandlerMemory::str_to_fractal("overflow"));
      return None;
    }
    if out == std::f32::NEG_INFINITY {
      hand_mem.push_fixed(args[2], 0);
      hand_mem.push_fractal(args[2], HandlerMemory::str_to_fractal("underflow"));
      return None;
    }
    let num = i32::from_ne_bytes(out.to_ne_bytes()) as i64;
    hand_mem.push_fixed(args[2], 1);
    hand_mem.push_fixed(args[2], num);
    None
  });
  cpu!(powf64 => fn(args, hand_mem) {
    let ra = hand_mem.read_fractal(args[0]);
    let rb = hand_mem.read_fractal(args[1]);
    if ra.read_fixed(0) == 0 {
      hand_mem.write_fractal(args[2], &ra);
      return None;
    }
    if rb.read_fixed(0) == 0 {
      hand_mem.write_fractal(args[2], &rb);
      return None;
    }
    let a = f64::from_ne_bytes(ra.read_fixed(1).to_ne_bytes());
    let b = f64::from_ne_bytes(rb.read_fixed(1).to_ne_bytes());
    let out = f64::powf(a, b);
    hand_mem.init_fractal(args[2]);
    if out == std::f64::INFINITY {
      hand_mem.push_fixed(args[2], 0);
      hand_mem.push_fractal(args[2], HandlerMemory::str_to_fractal("overflow"));
      return None;
    }
    if out == std::f64::NEG_INFINITY {
      hand_mem.push_fixed(args[2], 0);
      hand_mem.push_fractal(args[2], HandlerMemory::str_to_fractal("underflow"));
      return None;
    }
    let num = i64::from_ne_bytes(out.to_ne_bytes());
    hand_mem.push_fixed(args[2], 1);
    hand_mem.push_fixed(args[2], num);
    None
  });

  cpu!(sqrtf32 => fn(args, hand_mem) {
    let a = f32::from_ne_bytes((hand_mem.read_fixed(args[0]) as i32).to_ne_bytes());
    let out = i32::from_ne_bytes(f32::sqrt(a).to_ne_bytes()) as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!(sqrtf64 => fn(args, hand_mem) {
    let a = f64::from_ne_bytes(hand_mem.read_fixed(args[0]).to_ne_bytes());
    let out = i64::from_ne_bytes(f64::sqrt(a).to_ne_bytes());
    hand_mem.write_fixed(args[2], out);
    None
  });

  // Boolean and bitwise opcodes
  cpu!(andi8 => fn(args, hand_mem) {
    let a = hand_mem.read_fixed(args[0]) as i8;
    let b = hand_mem.read_fixed(args[1]) as i8;
    let out = (a & b) as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!(andi16 => fn(args, hand_mem) {
    let a = hand_mem.read_fixed(args[0]) as i16;
    let b = hand_mem.read_fixed(args[1]) as i16;
    let out = (a & b) as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!(andi32 => fn(args, hand_mem) {
    let a = hand_mem.read_fixed(args[0]) as i32;
    let b = hand_mem.read_fixed(args[1]) as i32;
    let out = (a & b) as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!(andi64 => fn(args, hand_mem) {
    let a = hand_mem.read_fixed(args[0]);
    let b = hand_mem.read_fixed(args[1]);
    let out = a & b;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!(andbool => fn(args, hand_mem) {
    let a = hand_mem.read_fixed(args[0]);
    let b = hand_mem.read_fixed(args[1]);
    let a_bool = if a == 1 { true } else { false };
    let b_bool = if b == 1 { true } else { false };
    let out = if a_bool & b_bool { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out);
    None
  });

  cpu!(ori8 => fn(args, hand_mem) {
    let a = hand_mem.read_fixed(args[0]) as i8;
    let b = hand_mem.read_fixed(args[1]) as i8;
    let out = (a | b) as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!(ori16 => fn(args, hand_mem) {
    let a = hand_mem.read_fixed(args[0]) as i16;
    let b = hand_mem.read_fixed(args[1]) as i16;
    let out = (a | b) as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!(ori32 => fn(args, hand_mem) {
    let a = hand_mem.read_fixed(args[0]) as i32;
    let b = hand_mem.read_fixed(args[1]) as i32;
    let out = (a | b) as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!(ori64 => fn(args, hand_mem) {
    let a = hand_mem.read_fixed(args[0]);
    let b = hand_mem.read_fixed(args[1]);
    let out = a | b;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!(orbool => fn(args, hand_mem) {
    let a = hand_mem.read_fixed(args[0]);
    let b = hand_mem.read_fixed(args[1]);
    let a_bool = if a == 1 { true } else { false };
    let b_bool = if b == 1 { true } else { false };
    let out = if a_bool | b_bool { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out);
    None
  });

  cpu!(xori8 => fn(args, hand_mem) {
    let a = hand_mem.read_fixed(args[0]) as i8;
    let b = hand_mem.read_fixed(args[1]) as i8;
    let out = (a ^ b) as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!(xori16 => fn(args, hand_mem) {
    let a = hand_mem.read_fixed(args[0]) as i16;
    let b = hand_mem.read_fixed(args[1]) as i16;
    let out = (a ^ b) as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!(xori32 => fn(args, hand_mem) {
    let a = hand_mem.read_fixed(args[0]) as i32;
    let b = hand_mem.read_fixed(args[1]) as i32;
    let out = (a ^ b) as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!(xori64 => fn(args, hand_mem) {
    let a = hand_mem.read_fixed(args[0]);
    let b = hand_mem.read_fixed(args[1]);
    let out = a ^ b;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!(xorbool => fn(args, hand_mem) {
    let a = hand_mem.read_fixed(args[0]);
    let b = hand_mem.read_fixed(args[1]);
    let a_bool = if a == 1 { true } else { false };
    let b_bool = if b == 1 { true } else { false };
    let out = if a_bool ^ b_bool { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out);
    None
  });

  cpu!(noti8 => fn(args, hand_mem) {
    let a = hand_mem.read_fixed(args[0]) as i8;
    let out = !a as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!(noti16 => fn(args, hand_mem) {
    let a = hand_mem.read_fixed(args[0]) as i16;
    let out = !a as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!(noti32 => fn(args, hand_mem) {
    let a = hand_mem.read_fixed(args[0]) as i32;
    let out = !a as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!(noti64 => fn(args, hand_mem) {
    let a = hand_mem.read_fixed(args[0]);
    let out = !a;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!(notbool => fn(args, hand_mem) {
    let a = hand_mem.read_fixed(args[0]);
    let a_bool = if a == 1 { true } else { false };
    let out = if !a_bool { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out);
    None
  });

  cpu!(nandi8 => fn(args, hand_mem) {
    let a = hand_mem.read_fixed(args[0]) as i8;
    let b = hand_mem.read_fixed(args[1]) as i8;
    let out = !(a & b) as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!(nandi16 => fn(args, hand_mem) {
    let a = hand_mem.read_fixed(args[0]) as i16;
    let b = hand_mem.read_fixed(args[1]) as i16;
    let out = !(a & b) as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!(nandi32 => fn(args, hand_mem) {
    let a = hand_mem.read_fixed(args[0]) as i32;
    let b = hand_mem.read_fixed(args[1]) as i32;
    let out = !(a & b) as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!(nandi64 => fn(args, hand_mem) {
    let a = hand_mem.read_fixed(args[0]);
    let b = hand_mem.read_fixed(args[1]);
    let out = !(a & b);
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!(nandboo => fn(args, hand_mem) {
    let a = hand_mem.read_fixed(args[0]);
    let b = hand_mem.read_fixed(args[1]);
    let a_bool = if a == 1 { true } else { false };
    let b_bool = if b == 1 { true } else { false };
    let out = if !(a_bool & b_bool) { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out);
    None
  });

  cpu!(nori8 => fn(args, hand_mem) {
    let a = hand_mem.read_fixed(args[0]) as i8;
    let b = hand_mem.read_fixed(args[1]) as i8;
    let out = !(a | b) as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!(nori16 => fn(args, hand_mem) {
    let a = hand_mem.read_fixed(args[0]) as i16;
    let b = hand_mem.read_fixed(args[1]) as i16;
    let out = !(a | b) as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!(nori32 => fn(args, hand_mem) {
    let a = hand_mem.read_fixed(args[0]) as i32;
    let b = hand_mem.read_fixed(args[1]) as i32;
    let out = !(a | b) as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!(nori64 => fn(args, hand_mem) {
    let a = hand_mem.read_fixed(args[0]);
    let b = hand_mem.read_fixed(args[1]);
    let out = !(a | b);
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!(norbool => fn(args, hand_mem) {
    let a = hand_mem.read_fixed(args[0]);
    let b = hand_mem.read_fixed(args[1]);
    let a_bool = if a == 1 { true } else { false };
    let b_bool = if b == 1 { true } else { false };
    let out = if !(a_bool | b_bool) { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out);
    None
  });

  cpu!(xnori8 => fn(args, hand_mem) {
    let a = hand_mem.read_fixed(args[0]) as i8;
    let b = hand_mem.read_fixed(args[1]) as i8;
    let out = !(a ^ b) as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!(xnori16 => fn(args, hand_mem) {
    let a = hand_mem.read_fixed(args[0]) as i16;
    let b = hand_mem.read_fixed(args[1]) as i16;
    let out = !(a ^ b) as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!(xnori32 => fn(args, hand_mem) {
    let a = hand_mem.read_fixed(args[0]) as i32;
    let b = hand_mem.read_fixed(args[1]) as i32;
    let out = !(a ^ b) as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!(xnori64 => fn(args, hand_mem) {
    let a = hand_mem.read_fixed(args[0]);
    let b = hand_mem.read_fixed(args[1]);
    let out = !(a ^ b);
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!(xnorboo => fn(args, hand_mem) {
    let a = hand_mem.read_fixed(args[0]);
    let b = hand_mem.read_fixed(args[1]);
    let a_bool = if a == 1 { true } else { false };
    let b_bool = if b == 1 { true } else { false };
    let out = if !(a_bool ^ b_bool) { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out);
    None
  });

  // Equality and order opcodes
  cpu!(eqi8 => fn(args, hand_mem) {
    let a = hand_mem.read_fixed(args[0]) as i8;
    let b = hand_mem.read_fixed(args[1]) as i8;
    let out = if a == b { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!(eqi16 => fn(args, hand_mem) {
    let a = hand_mem.read_fixed(args[0]) as i16;
    let b = hand_mem.read_fixed(args[1]) as i16;
    let out = if a == b { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!(eqi32 => fn(args, hand_mem) {
    let a = hand_mem.read_fixed(args[0]) as i32;
    let b = hand_mem.read_fixed(args[1]) as i32;
    let out = if a == b { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!(eqi64 => fn(args, hand_mem) {
    let a = hand_mem.read_fixed(args[0]);
    let b = hand_mem.read_fixed(args[1]);
    let out = if a == b { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!(eqf32 => fn(args, hand_mem) {
    let a = f32::from_ne_bytes((hand_mem.read_fixed(args[0]) as i32).to_ne_bytes());
    let b = f32::from_ne_bytes((hand_mem.read_fixed(args[1]) as i32).to_ne_bytes());
    let out = if a == b { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!(eqf64 => fn(args, hand_mem) {
    let a = f64::from_ne_bytes(hand_mem.read_fixed(args[0]).to_ne_bytes());
    let b = f64::from_ne_bytes(hand_mem.read_fixed(args[1]).to_ne_bytes());
    let out = if a == b { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!(eqstr => fn(args, hand_mem) {
    let a_pascal_string = hand_mem.read_fractal(args[0]);
    let b_pascal_string = hand_mem.read_fractal(args[1]);
    let out = if args[0] < 0 || args[1] < 0 {
      // Special path for global memory stored strings, they aren't represented the same way
      let a_str = HandlerMemory::fractal_to_string(a_pascal_string);
      let b_str = HandlerMemory::fractal_to_string(b_pascal_string);
      if a_str == b_str { 1i64 } else { 0i64 }
    } else if a_pascal_string == b_pascal_string {
      1i64
    } else {
      0i64
    };
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!(eqbool => fn(args, hand_mem) {
    let a = hand_mem.read_fixed(args[0]) as i8;
    let b = hand_mem.read_fixed(args[1]) as i8;
    let out = if a == b { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out);
    None
  });

  cpu!(neqi8 => fn(args, hand_mem) {
    let a = hand_mem.read_fixed(args[0]) as i8;
    let b = hand_mem.read_fixed(args[1]) as i8;
    let out = if a != b { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!(neqi16 => fn(args, hand_mem) {
    let a = hand_mem.read_fixed(args[0]) as i16;
    let b = hand_mem.read_fixed(args[1]) as i16;
    let out = if a != b { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!(neqi32 => fn(args, hand_mem) {
    let a = hand_mem.read_fixed(args[0]) as i32;
    let b = hand_mem.read_fixed(args[1]) as i32;
    let out = if a != b { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!(neqi64 => fn(args, hand_mem) {
    let a = hand_mem.read_fixed(args[0]);
    let b = hand_mem.read_fixed(args[1]);
    let out = if a != b { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!(neqf32 => fn(args, hand_mem) {
    let a = f32::from_ne_bytes((hand_mem.read_fixed(args[0]) as i32).to_ne_bytes());
    let b = f32::from_ne_bytes((hand_mem.read_fixed(args[1]) as i32).to_ne_bytes());
    let out = if a != b { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!(neqf64 => fn(args, hand_mem) {
    let a = f64::from_ne_bytes(hand_mem.read_fixed(args[0]).to_ne_bytes());
    let b = f64::from_ne_bytes(hand_mem.read_fixed(args[1]).to_ne_bytes());
    let out = if a != b { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!(neqstr => fn(args, hand_mem) {
    let a_pascal_string = hand_mem.read_fractal(args[0]);
    let b_pascal_string = hand_mem.read_fractal(args[1]);
    let out = if a_pascal_string != b_pascal_string {
      1i64
    } else {
      0i64
    };
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!(neqbool => fn(args, hand_mem) {
    let a = hand_mem.read_fixed(args[0]) as i8;
    let b = hand_mem.read_fixed(args[1]) as i8;
    let out = if a != b { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out);
    None
  });

  cpu!(lti8 => fn(args, hand_mem) {
    let a = hand_mem.read_fixed(args[0]) as i8;
    let b = hand_mem.read_fixed(args[1]) as i8;
    let out = if a < b { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!(lti16 => fn(args, hand_mem) {
    let a = hand_mem.read_fixed(args[0]) as i16;
    let b = hand_mem.read_fixed(args[1]) as i16;
    let out = if a < b { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!(lti32 => fn(args, hand_mem) {
    let a = hand_mem.read_fixed(args[0]) as i32;
    let b = hand_mem.read_fixed(args[1]) as i32;
    let out = if a < b { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!(lti64 => fn(args, hand_mem) {
    let a = hand_mem.read_fixed(args[0]);
    let b = hand_mem.read_fixed(args[1]);
    let out = if a < b { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!(ltf32 => fn(args, hand_mem) {
    let a = f32::from_ne_bytes((hand_mem.read_fixed(args[0]) as i32).to_ne_bytes());
    let b = f32::from_ne_bytes((hand_mem.read_fixed(args[1]) as i32).to_ne_bytes());
    let out = if a < b { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!(ltf64 => fn(args, hand_mem) {
    let a = f64::from_ne_bytes(hand_mem.read_fixed(args[0]).to_ne_bytes());
    let b = f64::from_ne_bytes(hand_mem.read_fixed(args[1]).to_ne_bytes());
    let out = if a < b { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!(ltstr => fn(args, hand_mem) {
    let a_str = HandlerMemory::fractal_to_string(hand_mem.read_fractal(args[0]));
    let b_str = HandlerMemory::fractal_to_string(hand_mem.read_fractal(args[1]));
    let out = if a_str < b_str { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out);
    None
  });

  cpu!(ltei8 => fn(args, hand_mem) {
    let a = hand_mem.read_fixed(args[0]) as i8;
    let b = hand_mem.read_fixed(args[1]) as i8;
    let out = if a <= b { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!(ltei16 => fn(args, hand_mem) {
    let a = hand_mem.read_fixed(args[0]) as i16;
    let b = hand_mem.read_fixed(args[1]) as i16;
    let out = if a <= b { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!(ltei32 => fn(args, hand_mem) {
    let a = hand_mem.read_fixed(args[0]) as i32;
    let b = hand_mem.read_fixed(args[1]) as i32;
    let out = if a <= b { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!(ltei64 => fn(args, hand_mem) {
    let a = hand_mem.read_fixed(args[0]);
    let b = hand_mem.read_fixed(args[1]);
    let out = if a <= b { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!(ltef32 => fn(args, hand_mem) {
    let a = f32::from_ne_bytes((hand_mem.read_fixed(args[0]) as i32).to_ne_bytes());
    let b = f32::from_ne_bytes((hand_mem.read_fixed(args[1]) as i32).to_ne_bytes());
    let out = if a <= b { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!(ltef64 => fn(args, hand_mem) {
    let a = f64::from_ne_bytes(hand_mem.read_fixed(args[0]).to_ne_bytes());
    let b = f64::from_ne_bytes(hand_mem.read_fixed(args[1]).to_ne_bytes());
    let out = if a <= b { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!(ltestr => fn(args, hand_mem) {
    let a_str = HandlerMemory::fractal_to_string(hand_mem.read_fractal(args[0]));
    let b_str = HandlerMemory::fractal_to_string(hand_mem.read_fractal(args[1]));
    let out = if a_str <= b_str { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out);
    None
  });

  cpu!(gti8 => fn(args, hand_mem) {
    let a = hand_mem.read_fixed(args[0]) as i8;
    let b = hand_mem.read_fixed(args[1]) as i8;
    let out = if a > b { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!(gti16 => fn(args, hand_mem) {
    let a = hand_mem.read_fixed(args[0]) as i16;
    let b = hand_mem.read_fixed(args[1]) as i16;
    let out = if a > b { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!(gti32 => fn(args, hand_mem) {
    let a = hand_mem.read_fixed(args[0]) as i32;
    let b = hand_mem.read_fixed(args[1]) as i32;
    let out = if a > b { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!(gti64 => fn(args, hand_mem) {
    let a = hand_mem.read_fixed(args[0]);
    let b = hand_mem.read_fixed(args[1]);
    let out = if a > b { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!(gtf32 => fn(args, hand_mem) {
    let a = f32::from_ne_bytes((hand_mem.read_fixed(args[0]) as i32).to_ne_bytes());
    let b = f32::from_ne_bytes((hand_mem.read_fixed(args[1]) as i32).to_ne_bytes());
    let out = if a > b { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!(gtf64 => fn(args, hand_mem) {
    let a = f64::from_ne_bytes(hand_mem.read_fixed(args[0]).to_ne_bytes());
    let b = f64::from_ne_bytes(hand_mem.read_fixed(args[1]).to_ne_bytes());
    let out = if a > b { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!(gtstr => fn(args, hand_mem) {
    let a_str = HandlerMemory::fractal_to_string(hand_mem.read_fractal(args[0]));
    let b_str = HandlerMemory::fractal_to_string(hand_mem.read_fractal(args[1]));
    let out = if a_str > b_str { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out);
    None
  });

  cpu!(gtei8 => fn(args, hand_mem) {
    let a = hand_mem.read_fixed(args[0]) as i8;
    let b = hand_mem.read_fixed(args[1]) as i8;
    let out = if a >= b { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!(gtei16 => fn(args, hand_mem) {
    let a = hand_mem.read_fixed(args[0]) as i16;
    let b = hand_mem.read_fixed(args[1]) as i16;
    let out = if a >= b { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!(gtei32 => fn(args, hand_mem) {
    let a = hand_mem.read_fixed(args[0]) as i32;
    let b = hand_mem.read_fixed(args[1]) as i32;
    let out = if a >= b { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!(gtei64 => fn(args, hand_mem) {
    let a = hand_mem.read_fixed(args[0]);
    let b = hand_mem.read_fixed(args[1]);
    let out = if a >= b { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!(gtef32 => fn(args, hand_mem) {
    let a = f32::from_ne_bytes((hand_mem.read_fixed(args[0]) as i32).to_ne_bytes());
    let b = f32::from_ne_bytes((hand_mem.read_fixed(args[1]) as i32).to_ne_bytes());
    let out = if a >= b { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!(gtef64 => fn(args, hand_mem) {
    let a = f64::from_ne_bytes(hand_mem.read_fixed(args[0]).to_ne_bytes());
    let b = f64::from_ne_bytes(hand_mem.read_fixed(args[1]).to_ne_bytes());
    let out = if a >= b { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!(gtestr => fn(args, hand_mem) {
    let a_str = HandlerMemory::fractal_to_string(hand_mem.read_fractal(args[0]));
    let b_str = HandlerMemory::fractal_to_string(hand_mem.read_fractal(args[1]));
    let out = if a_str >= b_str { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out);
    None
  });

  // String opcodes
  cpu!(catstr => fn(args, hand_mem) {
    let a_str = HandlerMemory::fractal_to_string(hand_mem.read_fractal(args[0]));
    let b_str = HandlerMemory::fractal_to_string(hand_mem.read_fractal(args[1]));
    let out_str = format!("{}{}", a_str, b_str);
    hand_mem.write_fractal(args[2], &HandlerMemory::str_to_fractal(&out_str));
    None
  });
  cpu!(split => fn(args, hand_mem) {
    let a_str = HandlerMemory::fractal_to_string(hand_mem.read_fractal(args[0]));
    let b_str = HandlerMemory::fractal_to_string(hand_mem.read_fractal(args[1]));
    let out_hms = a_str.split(&b_str).map(|out_str| HandlerMemory::str_to_fractal(&out_str));
    hand_mem.init_fractal(args[2]);
    for out in out_hms {
      hand_mem.push_fractal(args[2], out);
    }
    None
  });
  cpu!(repstr => fn(args, hand_mem) {
    let a_str = HandlerMemory::fractal_to_string(hand_mem.read_fractal(args[0]));
    let n = hand_mem.read_fixed(args[1]);
    let out_str = a_str.repeat(n as usize);
    hand_mem.write_fractal(args[2], &HandlerMemory::str_to_fractal(&out_str));
    None
  });
  cpu!(matches => fn(args, hand_mem) {
    let a_str = HandlerMemory::fractal_to_string(hand_mem.read_fractal(args[0]));
    let b_str = HandlerMemory::fractal_to_string(hand_mem.read_fractal(args[1]));
    let b_regex = Regex::new(&b_str).unwrap();
    let out = if b_regex.is_match(&a_str) { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!(indstr => fn(args, hand_mem) {
    let a_str = HandlerMemory::fractal_to_string(hand_mem.read_fractal(args[0]));
    let b_str = HandlerMemory::fractal_to_string(hand_mem.read_fractal(args[1]));
    let out_option = a_str.find(&b_str);
    hand_mem.init_fractal(args[2]);
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
  cpu!(lenstr => fn(args, hand_mem) {
    let pascal_string = hand_mem.read_fractal(args[0]);
    let val = pascal_string.read_fixed(0);
    hand_mem.write_fixed(args[2], val);
    None
  });
  cpu!(trim => fn(args, hand_mem) {
    let in_str = HandlerMemory::fractal_to_string(hand_mem.read_fractal(args[0]));
    let out_str = in_str.trim();
    hand_mem.write_fractal(args[2], &HandlerMemory::str_to_fractal(&out_str));
    None
  });

  // Array opcodes
  cpu!(register => fn(args, hand_mem) {
    // args[2] is the register address
    // args[0] point to an array in memory
    // args[1] is the address within the array to register
    let inner_addr = hand_mem.read_fixed(args[1]);
    hand_mem.register_out(args[0], inner_addr as usize, args[2]);
    None
  });
  cpu!(copyfrom => fn(args, hand_mem) {
    // args = [arr_addr, arr_idx_addr, outer_addr]
    // copy data from outer_addr to inner_addr of the array in reg_addr
    // The array index instead of inner address is provided to keep interaction with the js-runtime
    // sane.
    let inner_addr = hand_mem.read_fixed(args[1]);
    hand_mem.register_out(args[0], inner_addr as usize, args[2]);
    None
  });
  cpu!(copytof => fn(args, hand_mem) {
    // args = [arr_addr, inner_addr, outer_addr]
    // copy data from outer_addr to inner_addr in arr_addr
    let inner = hand_mem.read_fixed(args[1]);
    hand_mem.register_in(args[2], args[0], inner);
    None
  });
  cpu!(copytov => fn(args, hand_mem) {
    // args = [arr_addr, inner_addr, outer_addr]
    // copy data from outer_addr to inner_addr in arr_addr
    let inner = hand_mem.read_fixed(args[1]);
    hand_mem.register_in(args[2], args[0], inner);
    None
  });
  cpu!(lenarr => fn(args, hand_mem) {
    let arr = hand_mem.read_fractal(args[0]);
    let len = arr.len() as i64;
    hand_mem.write_fixed(args[2], len);
    None
  });
  cpu!(indarrf => fn(args, hand_mem) {
    let val = hand_mem.read_fixed(args[1]);
    let mem = hand_mem.read_fractal(args[0]);
    let len = mem.len();
    let mut idx = -1i64;
    for i in 0..len {
      let check = mem.read_fixed(i);
      if val == check {
        idx = i as i64;
        break;
      }
    }
    hand_mem.init_fractal(args[2]);
    if idx == -1i64 {
      hand_mem.push_fixed(args[2], 0i64);
      hand_mem.write_fractal(args[2], &HandlerMemory::str_to_fractal("element not found"));
    } else {
      hand_mem.push_fixed(args[2], 1i64);
      hand_mem.push_fixed(args[2], idx);
    }
    None
  });
  cpu!(indarrv => fn(args, hand_mem) {
    let val = hand_mem.read_fractal(args[1]);
    let fractal = hand_mem.read_fractal(args[0]);
    let mut idx: Option<i64> = None;
    for i in 0..fractal.len() {
      if let (check, true) = hand_mem.read_from_fractal(&fractal, i) {
        // TODO: equality comparisons for nested arrays, for now, assume it's string-like
        if val.len() != check.len() {
          continue;
        }
        let mut matches = true;
        for j in 0..val.len() {
          if !val.compare_at(j, &check) {
            matches = false;
            break;
          }
        }
        if matches {
          idx = Some(i as i64);
          break;
        }
      }
      // the else branch originally just had `continue`
    }
    hand_mem.init_fractal(args[2]);
    if let Some(idx) = idx {
      hand_mem.push_fixed(args[2], 1i64);
      hand_mem.push_fixed(args[2], idx);
    } else {
      hand_mem.push_fixed(args[2], 0i64);
      hand_mem.write_fractal(args[2], &HandlerMemory::str_to_fractal("element not found"));
    }
    None
  });
  cpu!(join => fn(args, hand_mem) {
    let sep_str = HandlerMemory::fractal_to_string(hand_mem.read_fractal(args[1]));
    let fractal = hand_mem.read_fractal(args[0]);
    let mut strs = Vec::with_capacity(fractal.len());
    for i in 0..fractal.len() {
      match hand_mem.read_from_fractal(&fractal, i) {
        (data, true) => {
          let v_str = HandlerMemory::fractal_to_string(data);
          strs.push(v_str);
        },
        (_, false) => todo!("handle joining non-fractal strings I think?"),
      }
    }
    let out_str = strs.join(&sep_str);
    hand_mem.write_fractal(args[2], &HandlerMemory::str_to_fractal(&out_str));
    None
  });
  cpu!(pusharr => fn(args, hand_mem) {
    let val_size = hand_mem.read_fixed(args[2]);
    if val_size == 0 {
      hand_mem.push_register(args[0], args[1]);
    } else {
      let val = hand_mem.read_fixed(args[1]);
      hand_mem.push_fixed(args[0], val);
    }
    None
  });
  cpu!(poparr => fn(args, hand_mem) {
    let last = hand_mem.pop(args[0]);
    hand_mem.init_fractal(args[2]);
    match last {
      Ok(record) => {
        hand_mem.push_fixed(args[2], 1i64);
        hand_mem.push_register_out(args[2], &record, 0);
      },
      Err(error_string) => {
        hand_mem.push_fixed(args[2], 0i64);
        hand_mem.push_fractal(args[2], HandlerMemory::str_to_fractal(&error_string));
      },
    }
    None
  });
  cpu!(delindx => fn(args, hand_mem) {
    let idx = hand_mem.read_fixed(args[1]) as usize;
    let el = hand_mem.delete(args[0], idx);
    hand_mem.init_fractal(args[2]);
    match el {
      Ok(record) => {
        hand_mem.push_fixed(args[2], 1i64);
        hand_mem.push_register_out(args[2], &record, 0);
      },
      Err(error_string) => {
        hand_mem.push_fixed(args[2], 0i64);
        hand_mem.push_fractal(args[2], HandlerMemory::str_to_fractal(&error_string));
      },
    }
    None
  });
  cpu!(newarr => fn(args, hand_mem) {
    hand_mem.init_fractal(args[2]);
    None
  });
  io!(map => fn(args, mut hand_mem) {
    Box::pin(async move {
      let fractal = hand_mem.read_fractal(args[0]);
      let subhandler = HandlerFragment::new(args[1], 0);
      let mut mappers = Vec::with_capacity(fractal.len());
      for i in 0..fractal.len() {
        let mut hm = HandlerMemory::fork(hand_mem.clone());
        hm.register_out(args[0], i, CLOSURE_ARG_MEM_START + 1);
        hm.write_fixed(CLOSURE_ARG_MEM_START + 2, i as i64);
        mappers.push(subhandler.clone().run(hm).then(HandlerMemory::drop_parent_async));
      }
      let hms = join_all(mappers).await;
      hand_mem.init_fractal(args[2]);
      for hm in hms {
        hand_mem.join(hm);
        hand_mem.push_register(args[2], CLOSURE_ARG_MEM_START);
      }
      hand_mem
    })
  });
  unpred_cpu!(mapl => fn(args, mut hand_mem) {
    Box::pin(async move {
      let fractal = hand_mem.read_fractal(args[0]);
      let subhandler = HandlerFragment::new(args[1], 0);
      hand_mem.init_fractal(args[2]);
      for i in 0..fractal.len() {
        hand_mem.register_out(args[0], i, CLOSURE_ARG_MEM_START + 1);
        hand_mem.write_fixed(CLOSURE_ARG_MEM_START + 2, i as i64);
        hand_mem = subhandler.clone().run(hand_mem).await;
        hand_mem.push_register(args[2], CLOSURE_ARG_MEM_START);
      }
      hand_mem
    })
  });
  cpu!(reparr => fn(args, hand_mem) {
    hand_mem.init_fractal(args[2]);
    let n = hand_mem.read_fixed(args[1]);
    if n == 0 {
      return None;
    }
    let fractal = hand_mem.read_fractal(args[0]);
    let mut is_fixed = true;
    let mut arr = Vec::with_capacity(fractal.len());
    for i in 0..fractal.len() {
      let (val, is_fractal) = hand_mem.read_from_fractal(&fractal, i);
      arr.push(val);
      if is_fractal {
        is_fixed = false;
      }
    }
    for _ in 0..n {
      for val in arr.iter() {
        if is_fixed {
          hand_mem.push_fixed(args[2], val.read_fixed(0));
        } else {
          hand_mem.push_fractal(args[2], val.clone());
        }
      }
    }
    None
  });
  io!(each => fn(args, hand_mem) {
    Box::pin(async move {
      if args[1] == NOP_ID {
        // each is expected to result in purely side effects
        return hand_mem;
      }
      let fractal = hand_mem.read_fractal(args[0]);
      let subhandler = HandlerFragment::new(args[1], 0);
      let mut runners = Vec::with_capacity(fractal.len());
      for i in 0..fractal.len() {
        let mut hm = HandlerMemory::fork(hand_mem.clone());
        hm.register_out(args[0], i, CLOSURE_ARG_MEM_START + 1);
        hm.write_fixed(CLOSURE_ARG_MEM_START + 2, i as i64);
        runners.push(subhandler.clone().run(hm).then(HandlerMemory::drop_parent_async));
      }
      join_all(runners).await;
      hand_mem
    })
  });
  unpred_cpu!(eachl => fn(args, mut hand_mem) {
    Box::pin(async move {
      if args[1] == NOP_ID {
        // eachl is expected to result in purely side effects
        return hand_mem;
      }
      let n = hand_mem.read_fractal(args[0]).len();
      let subhandler = HandlerFragment::new(args[1], 0);
      for i in 0..n {
        hand_mem.register_out(args[0], i, CLOSURE_ARG_MEM_START + 1);
        hand_mem.write_fixed(CLOSURE_ARG_MEM_START + 2, i as i64);
        hand_mem = subhandler.clone().run(hand_mem).await;
      }
      hand_mem
    })
  });
  io!(find => fn(args, mut hand_mem) {
    Box::pin(async move {
      if args[1] == NOP_ID {
        panic!("NOP closure provided when finding");
      }
      let fractal = hand_mem.read_fractal(args[0]);
      let len = fractal.len();
      let subhandler = HandlerFragment::new(args[1], 0);
      let mut finders = Vec::with_capacity(fractal.len());
      for i in 0..len {
        let mut hm = HandlerMemory::fork(hand_mem.clone());
        hm.register_out(args[0], i, CLOSURE_ARG_MEM_START + 1);
        finders.push(subhandler.clone().run(hm));
      }
      let hms = join_all(finders).await;
      let mut idx = None;
      for (i, hm) in hms.into_iter().enumerate() {
        let val = hm.read_fixed(CLOSURE_ARG_MEM_START);
        hm.drop_parent();
        if idx.is_none() && val == 1 {
          idx = Some(i);
        }
      }
      hand_mem.init_fractal(args[2]);
      if let Some(idx) = idx {
        hand_mem.push_fixed(args[2], 1);
        hand_mem.push_register_out(args[2], &fractal, idx);
      } else {
        hand_mem.push_fixed(args[2], 0);
        hand_mem.push_fractal(args[2], HandlerMemory::str_to_fractal("no element matches"));
      }
      hand_mem
    })
  });
  unpred_cpu!(findl => fn(args, mut hand_mem) {
    Box::pin(async move {
      if args[1] == NOP_ID {
        panic!("NOP closure provided when finding");
      }
      let fractal = hand_mem.read_fractal(args[0]);
      let subhandler = HandlerFragment::new(args[1], 0);
      for i in 0..fractal.len() {
        hand_mem.register_out(args[0], i, CLOSURE_ARG_MEM_START + 1);
        hand_mem = subhandler.clone().run(hand_mem).await;
        let val = hand_mem.read_fixed(CLOSURE_ARG_MEM_START);
        if val == 1 {
          hand_mem.init_fractal(args[2]);
          hand_mem.push_fixed(args[2], 1);
          hand_mem.push_register_out(args[2], &fractal, i);
          return hand_mem;
        }
      }
      hand_mem.init_fractal(args[2]);
      hand_mem.push_fixed(args[2], 0);
      hand_mem.push_fractal(args[2], HandlerMemory::str_to_fractal("no element matches"));
      hand_mem
    })
  });
  io!(some => fn(args, mut hand_mem) {
    Box::pin(async move {
      if args[1] == NOP_ID {
        panic!("NOP closure provided when checking if some");
      }
      let fractal = hand_mem.read_fractal(args[0]);
      let subhandler = HandlerFragment::new(args[1], 0);
      let mut somers = Vec::with_capacity(fractal.len());
      for i in 0..fractal.len() {
        let mut hm = HandlerMemory::fork(hand_mem.clone());
        hm.register_out(args[0], i, CLOSURE_ARG_MEM_START + 1);
        somers.push(subhandler.clone().run(hm));
      }
      let hms = join_all(somers).await;
      let mut ret_val = 0;
      for hm in hms {
        let val = hm.read_fixed(CLOSURE_ARG_MEM_START);
        hm.drop_parent();
        if val == 1 {
          ret_val = 1;
        }
      }
      hand_mem.write_fixed(args[2], ret_val);
      hand_mem
    })
  });
  unpred_cpu!(somel => fn(args, mut hand_mem) {
    Box::pin(async move {
      if args[1] == NOP_ID {
        panic!("NOP closure provided when checking if some");
      }
      let fractal = hand_mem.read_fractal(args[0]);
      let subhandler = HandlerFragment::new(args[1], 0);
      for i in 0..fractal.len() {
        hand_mem.register_out(args[0], i, CLOSURE_ARG_MEM_START + 1);
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
  io!(every => fn(args, mut hand_mem) {
    Box::pin(async move {
      if args[1] == NOP_ID {
        panic!("NOP closure provided when checking if every");
      }
      let fractal = hand_mem.read_fractal(args[0]);
      let subhandler = HandlerFragment::new(args[1], 0);
      let mut somers = Vec::with_capacity(fractal.len());
      for i in 0..fractal.len() {
        let mut hm = HandlerMemory::fork(hand_mem.clone());
        hm.register_out(args[0], i, CLOSURE_ARG_MEM_START + 1);
        somers.push(subhandler.clone().run(hm));
      }
      let hms = join_all(somers).await;
      let mut ret_val = 1;
      for hm in hms {
        let val = hm.read_fixed(CLOSURE_ARG_MEM_START);
        hm.drop_parent();
        if val == 0 {
          ret_val = 0;
        }
      }
      hand_mem.write_fixed(args[2], ret_val);
      hand_mem
    })
  });
  unpred_cpu!(everyl => fn(args, mut hand_mem) {
    Box::pin(async move {
      if args[1] == NOP_ID {
        panic!("NOP closure provided when checking if every");
      }
      let fractal = hand_mem.read_fractal(args[0]);
      let subhandler = HandlerFragment::new(args[1], 0);
      for i in 0..fractal.len() {
        hand_mem.register_out(args[0], i, CLOSURE_ARG_MEM_START + 1);
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
  cpu!(catarr => fn(args, hand_mem) {
    let fractal1 = hand_mem.read_fractal(args[0]);
    let fractal2 = hand_mem.read_fractal(args[1]);
    hand_mem.init_fractal(args[2]);
    for i in 0..fractal1.len() {
      hand_mem.push_register_out(args[2], &fractal1, i);
    }
    for i in 0..fractal2.len() {
      hand_mem.push_register_out(args[2], &fractal2, i);
    }
    None
  });
  io!(reducep => fn(args, mut hand_mem) {
    Box::pin(async move {
      if args[1] == NOP_ID {
        panic!("NOP closure provided when reducing");
      }
      let fractal = hand_mem.read_fractal(args[0]);
      let mut vals = Vec::with_capacity(fractal.len());
      for i in 0..fractal.len() {
        let mut hm = HandlerMemory::new(None, 1);
        hand_mem.register_out(args[0], i, CLOSURE_ARG_MEM_START);
        HandlerMemory::transfer(&hand_mem, CLOSURE_ARG_MEM_START, &mut hm, 0);
        vals.push(hm);
      }
      let subhandler = HandlerFragment::new(args[1], 0);
      // Log-n parallelism. First n/2 in parallel, then n/4, then n/8, etc
      while vals.len() > 1 {
        let mut reducers = Vec::with_capacity((vals.len() / 2) + 1);
        while vals.len() > 1 {
          let mut hm = hand_mem.clone();
          let a = vals.remove(0);
          let b = vals.remove(0);
          HandlerMemory::transfer(&a, 0, &mut hm, CLOSURE_ARG_MEM_START + 1);
          HandlerMemory::transfer(&b, 0, &mut hm, CLOSURE_ARG_MEM_START + 2);
          reducers.push(subhandler.clone().run(hm));
        }
        // Check if one of the records was skipped over this round, and if so, pop it into a
        // special field
        let maybe_hm = if vals.len() == 1 { Some(vals.remove(0)) } else { None };
        let hms = join_all(reducers).await;
        for mut hm in hms {
          hm.register(0, CLOSURE_ARG_MEM_START, false);
          vals.push(hm);
        }
        if let Some(hm) = maybe_hm {
          vals.push(hm);
        }
      }
      // There can be only one
      HandlerMemory::transfer(&vals[0], 0, &mut hand_mem, args[2]);
      hand_mem
    })
  });
  unpred_cpu!(reducel => fn(args, mut hand_mem) {
    Box::pin(async move {
      if args[1] == NOP_ID {
        panic!("NOP closure provided when reducing");
      }
      let fractal = hand_mem.read_fractal(args[0]);
      if fractal.len() == 0 {
        return hand_mem;
      }
      let mut vals = Vec::with_capacity(fractal.len());
      for i in 0..fractal.len() {
        let mut hm = HandlerMemory::new(None, 1);
        hand_mem.register_out(args[0], i, CLOSURE_ARG_MEM_START);
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
  // io!(foldp => fn(args, mut hand_mem) {
  //   todo!("foldp with the new only-`Arc<HandlerMemory>`");
  //   Box::pin(async move {
  //     let obj = hand_mem.read_fractal(args[0]);
  //     let (arr, _) = hand_mem.read_from_fractal(&obj, 0);
  //     let mut vals = Vec::with_capacity(arr.len());
  //     for i in 0..arr.len() {
  //       let mut hm = HandlerMemory::new(None, 1);
  //       hand_mem.register_from_fractal(CLOSURE_ARG_MEM_START, &arr, i);
  //       HandlerMemory::transfer(&hand_mem, CLOSURE_ARG_MEM_START, &mut hm, 0);
  //       vals.push(hm);
  //     }
  //     let subhandler = HandlerFragment::new(args[1], 0);
  //     hand_mem.register_out(args[0], 1, CLOSURE_ARG_MEM_START);
  //     let mut init = HandlerMemory::new(None, 1);
  //     HandlerMemory::transfer(&hand_mem, CLOSURE_ARG_MEM_START, &mut init, 0);
  //     // We can only go up to 'n' parallel sequential computations here
  //     let n = num_cpus::get();
  //     let l = vals.len();
  //     let s = l / n;
  //     let mut reducers = Vec::with_capacity(n);
  //     for i in 0..n {
  //       let subvals = if i == n - 1 {
  //         vals[i * s..].to_vec()
  //       } else {
  //         vals[i * s..(i + 1) * s].to_vec()
  //       };
  //       eprintln!("subvals: {:?}", subvals);
  //       let mem = hand_mem.clone();
  //       let init2 = init.clone();
  //       let subhandler2 = subhandler.clone();
  //       reducers.push(task::spawn(async move {
  //         let mut cumulative = init2.clone();
  //         for i in 0..subvals.len() {
  //           let current = &subvals[i];
  //           let mut hm = mem.clone();
  //           HandlerMemory::transfer(&cumulative, 0, &mut hm, CLOSURE_ARG_MEM_START + 1);
  //           HandlerMemory::transfer(current, 0, &mut hm, CLOSURE_ARG_MEM_START + 2);
  //           hm = subhandler2.clone().run(hm).await;
  //           HandlerMemory::transfer(&hm, CLOSURE_ARG_MEM_START, &mut cumulative, 0);
  //         }
  //         cumulative
  //       }));
  //     }
  //     let hms = join_all(reducers).await;
  //     hand_mem.init_fractal(args[2]);
  //     for i in 0..n {
  //       let hm = hms[i].as_ref().unwrap();
  //       HandlerMemory::transfer(&hm, 0, &mut hand_mem, CLOSURE_ARG_MEM_START);
  //       hand_mem.push_register(args[2], CLOSURE_ARG_MEM_START);
  //     }
  //     hand_mem
  //   })
  // });
  unpred_cpu!(foldl => fn(args, mut hand_mem) {
    Box::pin(async move {
      if args[1] == NOP_ID {
        panic!("NOP closure provided when folding");
      }
      let obj = hand_mem.read_fractal(args[0]);
      let (arr, _) = hand_mem.read_from_fractal(&obj, 0);
      let mut vals = Vec::with_capacity(arr.len());
      for i in 0..arr.len() {
        let mut hm = HandlerMemory::new(None, 1);
        hand_mem.register_from_fractal(CLOSURE_ARG_MEM_START, &arr, i);
        HandlerMemory::transfer(&hand_mem, CLOSURE_ARG_MEM_START, &mut hm, 0);
        vals.push(hm);
      }
      let subhandler = HandlerFragment::new(args[1], 0);
      hand_mem.register_out(args[0], 1, CLOSURE_ARG_MEM_START);
      let mut cumulative = HandlerMemory::new(None, 1);
      HandlerMemory::transfer(&hand_mem, CLOSURE_ARG_MEM_START, &mut cumulative, 0);
      for i in 0..vals.len() {
        let current = &vals[i];
        HandlerMemory::transfer(&cumulative, 0, &mut hand_mem, CLOSURE_ARG_MEM_START + 1);
        HandlerMemory::transfer(current, 0, &mut hand_mem, CLOSURE_ARG_MEM_START + 2);
        hand_mem = subhandler.clone().run(hand_mem).await;
        HandlerMemory::transfer(&hand_mem, CLOSURE_ARG_MEM_START, &mut cumulative, 0);
      }
      hand_mem.register(args[2], CLOSURE_ARG_MEM_START, false);
      hand_mem
    })
  });
  io!(filter => fn(args, mut hand_mem) {
    Box::pin(async move {
      if args[1] == NOP_ID {
        panic!("NOP closure provided when filtering");
      }
      let fractal = hand_mem.read_fractal(args[0]);
      let len = fractal.len();
      let subhandler = HandlerFragment::new(args[1], 0);
      let mut filters = Vec::with_capacity(len);
      for i in 0..len {
        let mut hm = HandlerMemory::fork(hand_mem.clone());
        hm.register_out(args[0], i, CLOSURE_ARG_MEM_START + 1);
        filters.push(subhandler.clone().run(hm));
      }
      let hms = join_all(filters).await;
      let mut idxs = vec![];
      for (i, hm) in hms.into_iter().enumerate() {
        let val = hm.read_fixed(CLOSURE_ARG_MEM_START);
        hm.drop_parent(); // this drops `hm`
        if val == 1 {
          idxs.push(i);
        }
      }
      hand_mem.init_fractal(args[2]);
      for i in idxs {
        hand_mem.push_register_out(args[2], &fractal, i);
      }
      hand_mem
    })
  });
  unpred_cpu!(filterl => fn(args, mut hand_mem) {
    Box::pin(async move {
      if args[1] == NOP_ID {
        panic!("NOP closure provided when filtering");
      }
      let fractal = hand_mem.read_fractal(args[0]);
      let len = fractal.len();
      let subhandler = HandlerFragment::new(args[1], 0);
      hand_mem.init_fractal(args[2]);
      for i in 0..len {
        hand_mem.register_out(args[0], i, CLOSURE_ARG_MEM_START + 1);
        hand_mem = subhandler.clone().run(hand_mem).await;
        let val = hand_mem.read_fixed(CLOSURE_ARG_MEM_START);
        if val == 1 {
          hand_mem.push_register_out(args[2], &fractal, i);
        }
      }
      hand_mem
    })
  });

  // Conditional opcode
  unpred_cpu!(condfn => fn(args, mut hand_mem) {
    Box::pin(async move {
      let cond = hand_mem.read_fixed(args[0]);
      let subhandler = HandlerFragment::new(args[1], 0);
      if cond == 1 {
        hand_mem = subhandler.run(hand_mem).await;
      }
      hand_mem
    })
  });

  // Std opcodes
  io!(execop => fn(args, mut hand_mem) {
    Box::pin(async move {
      let cmd = HandlerMemory::fractal_to_string(hand_mem.read_fractal(args[0]));
      let output = if cfg!(target_os = "windows") {
        Command::new("cmd").arg("/C").arg(cmd).output().await
      } else {
        Command::new("sh").arg("-c").arg(cmd).output().await
      };
      hand_mem.init_fractal(args[2]);
      match output {
        Err(e) => {
          hand_mem.push_fixed(args[2], 127);
          hand_mem.push_fractal(args[2], FractalMemory::new(vec![(0, 0)]));
          let error_string = e.to_string();
          hand_mem.push_fractal(args[2], HandlerMemory::str_to_fractal(&error_string));
        },
        Ok(output_res) => {
          let status_code = output_res.status.code().unwrap_or(127) as i64;
          hand_mem.push_fixed(args[2], status_code);
          let stdout_str = String::from_utf8(output_res.stdout).unwrap_or("".to_string());
          hand_mem.push_fractal(args[2], HandlerMemory::str_to_fractal(&stdout_str));
          let stderr_str = String::from_utf8(output_res.stderr).unwrap_or("".to_string());
          hand_mem.push_fractal(args[2], HandlerMemory::str_to_fractal(&stderr_str));
        },
      };
      hand_mem
    })
  });

  unpred_cpu!(waitop => fn(args, hand_mem) {
    Box::pin(async move {
      let ms = hand_mem.read_fixed(args[0]) as u64;
      sleep(Duration::from_millis(ms)).await;
      hand_mem
    })
  });

  unpred_cpu!(syncop => fn(args, mut hand_mem) {
    Box::pin(async move {
      let closure = HandlerFragment::new(args[0], 0);
      hand_mem.register(CLOSURE_ARG_MEM_START + 1, args[1], true);
      hand_mem = closure.clone().run(hand_mem).await;
      hand_mem.register(args[2], CLOSURE_ARG_MEM_START, true);
      hand_mem
    })
  });

  // IO opcodes
  fn __httpreq(
    method: String,
    uri: String,
    headers: Vec<(String, String)>,
    body: Option<String>,
  ) -> Result<ResponseFuture, String> {
    let mut req = Request::builder()
      .method(method.as_str())
      .uri(uri.as_str());
    for header in headers {
      req = req.header(header.0.as_str(), header.1.as_str());
    }
    let req_obj = if let Some(body) = body {
      req.body(Body::from(body))
    } else {
      req.body(Body::empty())
    };
    if req_obj.is_err() {
      return Err("Failed to construct request, invalid body provided".to_string());
    } else {
      return Ok(Client::builder()
        .build::<_, Body>(HttpsConnector::with_native_roots())
        .request(req_obj.unwrap()));
    }
  }
  io!(httpreq => fn(args, mut hand_mem) {
    Box::pin(async move {
      let req = hand_mem.read_fractal(args[0]);
      let method = HandlerMemory::fractal_to_string(hand_mem.read_from_fractal(&req, 0).0);
      let url = HandlerMemory::fractal_to_string(hand_mem.read_from_fractal(&req, 1).0);
      let headers = hand_mem.read_from_fractal(&req, 2).0;
      let mut out_headers = Vec::new();
      for i in 0..headers.len() {
        let header = hand_mem.read_from_fractal(&headers, i).0;
        let key = HandlerMemory::fractal_to_string(hand_mem.read_from_fractal(&header, 0).0);
        let val = HandlerMemory::fractal_to_string(hand_mem.read_from_fractal(&header, 1).0);
        out_headers.push((key, val));
      }
      let body = HandlerMemory::fractal_to_string(hand_mem.read_from_fractal(&req, 3).0);
      let out_body = if body.len() > 0 { Some(body) /* once told me... */ } else { None };
      hand_mem.init_fractal(args[2]);
      let res = match __httpreq(method, url, out_headers, out_body) {
        Ok(res) => res,
        Err(estring) => {
          hand_mem.push_fixed(args[2], 0i64);
          hand_mem.push_fractal(args[2], HandlerMemory::str_to_fractal(&estring));
          return hand_mem;
        },
      };
      let mut res = match res.await {
        Ok(res) => res,
        Err(ee) => {
          hand_mem.push_fixed(args[2], 0i64);
          hand_mem.push_fractal(
            args[2],
            HandlerMemory::str_to_fractal(format!("{}", ee).as_str())
          );
          return hand_mem;
        },
      };
      // The headers and body can fail, so check those first
      let headers = res.headers();
      let mut headers_hm = HandlerMemory::new(None, headers.len() as i64);
      headers_hm.init_fractal(CLOSURE_ARG_MEM_START);
      for (i, (key, val)) in headers.iter().enumerate() {
        let key_str = key.as_str();
        let val_str = val.to_str();
        match val_str {
          Ok(val_str) => {
            headers_hm.init_fractal(i as i64);
            headers_hm.push_fractal(i as i64, HandlerMemory::str_to_fractal(key_str));
            headers_hm.push_fractal(i as i64, HandlerMemory::str_to_fractal(val_str));
            headers_hm.push_register(CLOSURE_ARG_MEM_START, i as i64);
          },
          Err(_) => {
            hand_mem.push_fixed(args[2], 0i64);
            hand_mem.push_fractal(
              args[2],
              HandlerMemory::str_to_fractal("Malformed headers encountered")
            );
            return hand_mem;
          },
        }
      }
      let body = match hyper::body::to_bytes(res.body_mut()).await {
        Ok(body) => body,
        Err(ee) => {
          hand_mem.push_fixed(args[2], 0i64);
          hand_mem.push_fractal(
            args[2],
            HandlerMemory::str_to_fractal(format!("{}", ee).as_str())
          );
          return hand_mem;
        },
      };
      let body_str = match String::from_utf8(body.to_vec()) {
        Ok(body_str) => body_str,
        Err(ee) => {
          hand_mem.push_fixed(args[2], 0i64);
          hand_mem.push_fractal(
            args[2],
            HandlerMemory::str_to_fractal(format!("{}", ee).as_str())
          );
          return hand_mem;
        },
      };
      hand_mem.push_fixed(args[2], 1i64);
      let mut res_hm = HandlerMemory::new(None, 3);
      res_hm.init_fractal(0);
      res_hm.push_fixed(0, res.status().as_u16() as i64);
      HandlerMemory::transfer(&headers_hm, CLOSURE_ARG_MEM_START, &mut res_hm, CLOSURE_ARG_MEM_START);
      res_hm.push_register(0, CLOSURE_ARG_MEM_START);
      res_hm.push_fractal(0, HandlerMemory::str_to_fractal(&body_str));
      res_hm.push_fixed(0, 0i64);
      HandlerMemory::transfer(&res_hm, 0, &mut hand_mem, CLOSURE_ARG_MEM_START);
      hand_mem.push_register(args[2], CLOSURE_ARG_MEM_START);
      hand_mem
    })
  });

  async fn http_listener(req: Request<Body>) -> Result<Response<Body>, Infallible> {
    // Create a new event handler memory to add to the event queue
    let mut event = HandlerMemory::new(None, 1);
    // Grab the method
    let method_str = req.method().to_string();
    let method = HandlerMemory::str_to_fractal(&method_str);
    // Grab the URL
    let url_str = req.uri().to_string();
    let url = HandlerMemory::str_to_fractal(&url_str);
    // Grab the headers
    let headers = req.headers();
    let mut headers_hm = HandlerMemory::new(None, headers.len() as i64);
    headers_hm.init_fractal(CLOSURE_ARG_MEM_START);
    for (i, (key, val)) in headers.iter().enumerate() {
      let key_str = key.as_str();
      let val_str = val.to_str().unwrap();
      headers_hm.init_fractal(i as i64);
      headers_hm.push_fractal(i as i64, HandlerMemory::str_to_fractal(key_str));
      headers_hm.push_fractal(i as i64, HandlerMemory::str_to_fractal(val_str));
      headers_hm.push_register(CLOSURE_ARG_MEM_START, i as i64);
    }
    // Grab the body, if any
    let body_req = match hyper::body::to_bytes(req.into_body()).await {
      Ok(bytes) => bytes,
      // If we error out while getting the body, just close this listener out immediately
      Err(ee) => {
        return Ok(Response::new(
          format!("Connection terminated: {}", ee).into(),
        ));
      }
    };
    let body_str = str::from_utf8(&body_req).unwrap().to_string();
    let body = HandlerMemory::str_to_fractal(&body_str);
    // Generate a connection ID
    let conn_id = OsRng.next_u64() as i64;
    // Populate the event and emit it
    event.init_fractal(0);
    event.push_fractal(0, method);
    event.push_fractal(0, url);
    HandlerMemory::transfer(&headers_hm, CLOSURE_ARG_MEM_START, &mut event, CLOSURE_ARG_MEM_START);
    event.push_register(0, CLOSURE_ARG_MEM_START);
    event.push_fractal(0, body);
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
    let response_hm = poll_fn(|cx: &mut Context<'_>| -> Poll<Arc<HandlerMemory>> {
      let responses_hm = responses.lock().unwrap();
      let hm = responses_hm.get(&conn_id);
      if let Some(hm) = hm {
        Poll::Ready(hm.clone())
      } else {
        drop(hm);
        drop(responses_hm);
        let waker = cx.waker().clone();
        // TODO: os threads are quite expensive and could cause our runtime to
        // stop executing, this might be a point of optimization:
        thread::spawn(|| {
          thread::sleep(Duration::from_millis(10));
          waker.wake();
        });
        Poll::Pending
      }
    })
    .await;
    // Get the status from the user response and begin building the response object
    let status = response_hm.read_fixed(0) as u16;
    let mut res = Response::builder().status(StatusCode::from_u16(status).unwrap());
    // Get the headers and populate the response object
    let headers = res.headers_mut().unwrap();
    let header_hms = response_hm.read_fractal(1);
    for i in 0..header_hms.len() {
      let (h, _) = response_hm.read_from_fractal(&header_hms.clone(), i);
      let (key_hm, _) = response_hm.read_from_fractal(&h, 0);
      let (val_hm, _) = response_hm.read_from_fractal(&h, 1);
      let key = HandlerMemory::fractal_to_string(key_hm);
      let val = HandlerMemory::fractal_to_string(val_hm);
      let name = HeaderName::from_bytes(key.as_bytes()).unwrap();
      let value = HeaderValue::from_str(&val).unwrap();
      headers.insert(name, value);
    }
    // Get the body, populate the response object, and fire it out
    let body = HandlerMemory::fractal_to_string(response_hm.read_fractal(2));
    Ok(res.body(body.into()).unwrap())
  }
  io!(httplsn => fn(_args, hand_mem) {
    Box::pin(async move {
      let port_num = match &Program::global().http_config {
        HttpType::HTTP(http) => http.port,
        HttpType::HTTPS(https) => https.port,
      };
      let addr = SocketAddr::from(([0, 0, 0, 0], port_num));
      let make_svc = make_service_fn(|_conn| async { Ok::<_, Infallible>(service_fn(http_listener)) });

      let bind = Server::try_bind(&addr);
      match bind {
        Ok(server) => {
          let server = server.serve(make_svc);
          tokio::spawn(async move { server.await });
          println!("HTTP server listening on port {}", port_num);
        },
        Err(ee) => eprintln!("HTTP server failed to listen on port {}: {}", port_num, ee),
      }
      return hand_mem;
    })
  });
  io!(httpsend => fn(args, mut hand_mem) {
    Box::pin(async move {
      hand_mem.dupe(args[0], args[0]); // Make sure there's no pointers involved
      let fractal = hand_mem.read_fractal(args[0]);
      let conn_id = fractal.read_fixed(3);
      let responses = Arc::clone(&HTTP_RESPONSES);
      let mut responses_hm = responses.lock().unwrap();
      let mut hm = HandlerMemory::new(None, 1);
      HandlerMemory::transfer(&hand_mem, args[0], &mut hm, CLOSURE_ARG_MEM_START);
      let res_out = hm.read_fractal(CLOSURE_ARG_MEM_START);
      for i in 0..res_out.len() {
        hm.register_from_fractal(i as i64, &res_out, i);
      }
      responses_hm.insert(conn_id, hm);
      drop(responses_hm);
      // TODO: Add a second synchronization tool to return a valid Result status, for now, just
      // return success
      hand_mem.init_fractal(args[2]);
      hand_mem.push_fixed(args[2], 0i64);
      hand_mem.push_fractal(args[2], HandlerMemory::str_to_fractal("ok"));
      hand_mem
    })
  });

  // Datastore opcodes
  io!(dssetf => fn(args, hand_mem) {
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
  io!(dssetv => fn(args, hand_mem) {
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
  io!(dshas => fn(args, mut hand_mem) {
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
  io!(dsdel => fn(args, mut hand_mem) {
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
  io!(dsgetf => fn(args, mut hand_mem) {
    Box::pin(async move {
      let ns = HandlerMemory::fractal_to_string(hand_mem.read_fractal(args[0]));
      let key = HandlerMemory::fractal_to_string(hand_mem.read_fractal(args[1]));
      let nskey = format!("{}:{}", ns, key);
      let ds = Arc::clone(&DS);
      let maybe_hm = ds.get(&nskey);
      hand_mem.init_fractal(args[2]);
      hand_mem.push_fixed(args[2], if maybe_hm.is_some() { 1i64 } else { 0i64 });
      match maybe_hm {
        Some(hm) => hand_mem.push_fixed(args[2], hm.read_fixed(0)),
        None => hand_mem.push_fractal(args[2], HandlerMemory::str_to_fractal("namespace-key pair not found")),
      }
      hand_mem
    })
  });
  io!(dsgetv => fn(args, mut hand_mem) {
    Box::pin(async move {
      let ns = HandlerMemory::fractal_to_string(hand_mem.read_fractal(args[0]));
      let key = HandlerMemory::fractal_to_string(hand_mem.read_fractal(args[1]));
      let nskey = format!("{}:{}", ns, key);
      let ds = Arc::clone(&DS);
      let maybe_hm = ds.get(&nskey);
      hand_mem.init_fractal(args[2]);
      match maybe_hm {
        Some(hm) => {
          hand_mem.push_fixed(args[2], 1i64);
          HandlerMemory::transfer(&hm, 0, &mut hand_mem, CLOSURE_ARG_MEM_START);
          hand_mem.push_register(args[2], CLOSURE_ARG_MEM_START);
        },
        None => {
          hand_mem.push_fixed(args[2], 0i64);
          hand_mem.push_fractal(args[2], HandlerMemory::str_to_fractal("namespace-key pair not found"))
        },
      }
      hand_mem
    })
  });

  // seq opcodes
  cpu!(newseq => fn(args, hand_mem) {
    hand_mem.init_fractal(args[2]);
    hand_mem.push_fixed(args[2], 0i64);
    hand_mem.push_fixed(args[2], hand_mem.read_fixed(args[0]));
    None
  });
  cpu!(seqnext => fn(args, hand_mem) {
    hand_mem.init_fractal(args[2]);
    let mut seq = hand_mem.read_fractal(args[0]);
    let current = seq.read_fixed(0);
    let limit = seq.read_fixed(1);
    if current < limit {
      hand_mem.write_fixed_in_fractal(&mut seq, 0, current + 1);
      hand_mem.push_fixed(args[2], 1i64);
      hand_mem.push_fixed(args[2], current);
    } else {
      hand_mem.push_fixed(args[2], 0i64);
      let err_msg = "error: sequence out-of-bounds";
      hand_mem.push_fractal(args[2], HandlerMemory::str_to_fractal(&err_msg));
    }
    None
  });
  unpred_cpu!(seqeach => fn(args, mut hand_mem) {
    Box::pin(async move {
      if args[1] == NOP_ID {
        // same as `each`
        return hand_mem;
      }
      let mut seq = hand_mem.read_fractal(args[0]);
      let current = seq.read_fixed(0);
      let limit = seq.read_fixed(1);
      let subhandler = HandlerFragment::new(args[1], 0);
      if current >= limit {
        return hand_mem;
      }
      hand_mem.write_fixed_in_fractal(&mut seq, 0, limit);
      // array of potentially many levels of nested fractals
      for i in current..limit {
        // array element is $1 argument of the closure memory space
        hand_mem.write_fixed(CLOSURE_ARG_MEM_START + 1, i);
        hand_mem = subhandler.clone().run(hand_mem).await;
      }
      hand_mem
    })
  });
  unpred_cpu!(seqwhile => fn(args, mut hand_mem) {
    Box::pin(async move {
      if args[1] == NOP_ID {
        panic!("NOP closure provided instead of a while condition");
      }
      let seq = hand_mem.read_fractal(args[0]);
      let mut current = seq.read_fixed(0);
      let limit = seq.read_fixed(1);
      drop(seq);
      let cond_handler = HandlerFragment::new(args[1], 0);
      let body_handler = HandlerFragment::new(args[2], 0);
      if current >= limit {
        return hand_mem;
      }
      hand_mem = cond_handler.clone().run(hand_mem).await;
      while current < limit && hand_mem.read_fixed(CLOSURE_ARG_MEM_START) > 0 {
        if args[2] != NOP_ID {
          hand_mem = body_handler.clone().run(hand_mem).await;
        }
        current = current + 1;
        hand_mem = cond_handler.clone().run(hand_mem).await;
      }
      let mut seq = hand_mem.read_fractal(args[0]);
      hand_mem.write_fixed_in_fractal(&mut seq, 0, current);
      hand_mem
    })
  });
  unpred_cpu!(seqdo => fn(args, mut hand_mem) {
    Box::pin(async move {
      let seq = hand_mem.read_fractal(args[0]);
      let mut current = seq.read_fixed(0);
      let limit = seq.read_fixed(1);
      drop(seq);
      let subhandler = HandlerFragment::new(args[1], 0);
      loop {
        if args[1] != NOP_ID {
          hand_mem = subhandler.clone().run(hand_mem).await;
        }
        current = current + 1;
        if current >= limit || hand_mem.read_fixed(CLOSURE_ARG_MEM_START) == 0 {
          break;
        }
      }
      let mut seq = hand_mem.read_fractal(args[0]);
      hand_mem.write_fixed_in_fractal(&mut seq, 0, current);
      hand_mem
    })
  });
  unpred_cpu!(selfrec => fn(args, mut hand_mem) {
    Box::pin(async move {
      let mut hm = HandlerMemory::fork(hand_mem.clone());
      // MUST read these first in case the arguments are themselves closure args being overwritten
      // for the recursive function.
      // Since we mutate the `Self` object in this, it *must* be read as mutable *first* to make
      // sure that the later registration of the `Self` object is pointing at the correct copy
      let slf = hm.read_mut_fractal(args[0]);
      let recurse_fn = HandlerFragment::new(slf[1].1, 0);
      let seq_addr = slf[0].0;
      drop(slf);
      hm.register(CLOSURE_ARG_MEM_START + 1, args[0], false);
      hm.register(CLOSURE_ARG_MEM_START + 2, args[1], false);
      let seq = hm.read_mut_fractal_by_idx(seq_addr);
      let curr = seq[0].1;
      if curr < seq[1].1 {
        seq[0].1 = curr + 1;
        hm = recurse_fn.run(hm).await;
        hm = hm.drop_parent();
        // CANNOT `join` the memory like usual because the nested `recurse` calls have set "future"
        // values in the handler and will cause weird behavior. Only transfer the Self mutation and
        // the return value between iterations
        HandlerMemory::transfer(&mut hm, CLOSURE_ARG_MEM_START, &mut hand_mem, args[2]);
        HandlerMemory::transfer(&mut hm, CLOSURE_ARG_MEM_START + 1, &mut hand_mem, args[0]);
      } else {
        hand_mem.init_fractal(args[2]);
        hand_mem.push_fixed(args[2], 0);
        hand_mem.push_fractal(args[2], HandlerMemory::str_to_fractal("error: sequence out-of-bounds"));
      }
      hand_mem
    })
  });
  cpu!(seqrec => fn(args, hand_mem) {
    if args[1] == NOP_ID {
      panic!("NOP can't be recursive");
    }
    hand_mem.init_fractal(args[2]);
    hand_mem.push_register(args[2], args[0]);
    hand_mem.push_fixed(args[2], args[1]);
    None
  });

  // "Special" opcodes
  cpu!(exitop => fn(args, hand_mem) {
    io::stdout().flush().unwrap();
    io::stderr().flush().unwrap();
    std::process::exit(hand_mem.read_fixed(args[0]) as i32);
  });
  cpu!(stdoutp => fn(args, hand_mem) {
    let out_str = HandlerMemory::fractal_to_string(hand_mem.read_fractal(args[0]));
    print!("{}", out_str);
    None
  });
  cpu!(stderrp => fn(args, hand_mem) {
    let err_str = HandlerMemory::fractal_to_string(hand_mem.read_fractal(args[0]));
    eprint!("{}", err_str);
    None
  });

  // set opcodes use args[0] directly, since the relevant value directly
  // fits in i64, and write it to args[2]
  cpu!(seti64 => fn(args, hand_mem) {
    let data = args[0];
    hand_mem.write_fixed(args[2], data);
    None
  });
  cpu!(seti32 => fn(args, hand_mem) {
    let data = (args[0] as i32) as i64;
    hand_mem.write_fixed(args[2], data);
    None
  });
  cpu!(seti16 => fn(args, hand_mem) {
    let data = (args[0] as i16) as i64;
    hand_mem.write_fixed(args[2], data);
    None
  });
  cpu!(seti8 => fn(args, hand_mem) {
    let data = (args[0] as i8) as i64;
    hand_mem.write_fixed(args[2], data);
    None
  });
  cpu!(setf64 => fn(args, hand_mem) {
    let data = i64::from_ne_bytes((args[0] as f64).to_ne_bytes());
    hand_mem.write_fixed(args[2], data);
    None
  });
  cpu!(setf32 => fn(args, hand_mem) {
    let data = i32::from_ne_bytes((args[0] as f32).to_ne_bytes()) as i64;
    hand_mem.write_fixed(args[2], data);
    None
  });
  cpu!(setbool => fn(args, hand_mem) {
    let data = if args[0] == 0 { 0i64 } else { 1i64 };
    hand_mem.write_fixed(args[2], data);
    None
  });
  cpu!(setestr => fn(args, hand_mem) {
    let empty_str = FractalMemory::new(vec![(0, 0)]);
    hand_mem.write_fractal(args[2], &empty_str);
    None
  });

  // copy opcodes used for let variable reassignments
  cpu!(copyi8 => fn(args, hand_mem) {
    let val = hand_mem.read_fixed(args[0]);
    hand_mem.write_fixed(args[2], val);
    None
  });
  cpu!(copyi16 => fn(args, hand_mem) {
    let val = hand_mem.read_fixed(args[0]);
    hand_mem.write_fixed(args[2], val);
    None
  });
  cpu!(copyi32 => fn(args, hand_mem) {
    let val = hand_mem.read_fixed(args[0]);
    hand_mem.write_fixed(args[2], val);
    None
  });
  cpu!(copyi64 => fn(args, hand_mem) {
    let val = hand_mem.read_fixed(args[0]);
    hand_mem.write_fixed(args[2], val);
    None
  });
  cpu!(copyvoid => fn(args, hand_mem) {
    let val = hand_mem.read_fixed(args[0]);
    hand_mem.write_fixed(args[2], val);
    None
  });
  cpu!(copyf32 => fn(args, hand_mem) {
    let val = hand_mem.read_fixed(args[0]);
    hand_mem.write_fixed(args[2], val);
    None
  });
  cpu!(copyf64 => fn(args, hand_mem) {
    let val = hand_mem.read_fixed(args[0]);
    hand_mem.write_fixed(args[2], val);
    None
  });
  cpu!(copybool => fn(args, hand_mem) {
    let val = hand_mem.read_fixed(args[0]);
    hand_mem.write_fixed(args[2], val);
    None
  });
  cpu!(copystr => fn(args, hand_mem) {
    let pascal_string = hand_mem.read_fractal(args[0]);
    hand_mem.write_fractal(args[2], &pascal_string);
    None
  });
  cpu!(copyarr => fn(args, hand_mem) {
    // args = [in_addr, unused, out_addr]
    hand_mem.dupe(args[0], args[2]);
    None
  });
  cpu!(zeroed => fn(args, hand_mem) {
    hand_mem.write_fixed(args[2], 0);
    None
  });

  // Trig opcodes
  cpu!(lnf64 => fn(args, hand_mem) {
    let a = f64::from_ne_bytes(hand_mem.read_fixed(args[0]).to_ne_bytes());
    let out = i64::from_ne_bytes(a.ln().to_ne_bytes());
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!(logf64 => fn(args, hand_mem) {
    let a = f64::from_ne_bytes(hand_mem.read_fixed(args[0]).to_ne_bytes());
    let out = i64::from_ne_bytes(a.log10().to_ne_bytes());
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!(sinf64 => fn(args, hand_mem) {
    let a = f64::from_ne_bytes(hand_mem.read_fixed(args[0]).to_ne_bytes());
    let out = i64::from_ne_bytes(a.sin().to_ne_bytes());
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!(cosf64 => fn(args, hand_mem) {
    let a = f64::from_ne_bytes(hand_mem.read_fixed(args[0]).to_ne_bytes());
    let out = i64::from_ne_bytes(a.cos().to_ne_bytes());
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!(tanf64 => fn(args, hand_mem) {
    let a = f64::from_ne_bytes(hand_mem.read_fixed(args[0]).to_ne_bytes());
    let out = i64::from_ne_bytes(a.tan().to_ne_bytes());
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!(asinf64 => fn(args, hand_mem) {
    let a = f64::from_ne_bytes(hand_mem.read_fixed(args[0]).to_ne_bytes());
    let out = i64::from_ne_bytes(a.asin().to_ne_bytes());
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!(acosf64 => fn(args, hand_mem) {
    let a = f64::from_ne_bytes(hand_mem.read_fixed(args[0]).to_ne_bytes());
    let out = i64::from_ne_bytes(a.acos().to_ne_bytes());
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!(atanf64 => fn(args, hand_mem) {
    let a = f64::from_ne_bytes(hand_mem.read_fixed(args[0]).to_ne_bytes());
    let out = i64::from_ne_bytes(a.atan().to_ne_bytes());
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!(sinhf64 => fn(args, hand_mem) {
    let a = f64::from_ne_bytes(hand_mem.read_fixed(args[0]).to_ne_bytes());
    let out = i64::from_ne_bytes(a.sinh().to_ne_bytes());
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!(coshf64 => fn(args, hand_mem) {
    let a = f64::from_ne_bytes(hand_mem.read_fixed(args[0]).to_ne_bytes());
    let out = i64::from_ne_bytes(a.cosh().to_ne_bytes());
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!(tanhf64 => fn(args, hand_mem) {
    let a = f64::from_ne_bytes(hand_mem.read_fixed(args[0]).to_ne_bytes());
    let out = i64::from_ne_bytes(a.tanh().to_ne_bytes());
    hand_mem.write_fixed(args[2], out);
    None
  });

  // Error, Maybe, Result, Either opcodes
  cpu!(error => fn(args, hand_mem) {
    hand_mem.register(args[2], args[0], true);
    None
  });
  cpu!(refv => fn(args, hand_mem) {
    hand_mem.register(args[2], args[0], true);
    None
  });
  cpu!(reff => fn(args, hand_mem) {
    hand_mem.register(args[2], args[0], false);
    None
  });
  cpu!(noerr => fn(args, hand_mem) {
    let empty_string = FractalMemory::new(vec![(0, 0)]);
    hand_mem.write_fractal(args[2], &empty_string);
    None
  });
  cpu!(errorstr => fn(args, hand_mem) {
    hand_mem.register(args[2], args[0], true);
    None
  });
  cpu!(someM => fn(args, hand_mem) {
    hand_mem.init_fractal(args[2]);
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
  cpu!(noneM => fn(args, hand_mem) {
    hand_mem.init_fractal(args[2]);
    hand_mem.push_fixed(args[2], 0i64);
    None
  });
  cpu!(isSome => fn(args, hand_mem) {
    hand_mem.register_out(args[0], 0, args[2]);
    None
  });
  cpu!(isNone => fn(args, hand_mem) {
    let fractal = hand_mem.read_fractal(args[0]);
    let val = fractal.read_fixed(0);
    hand_mem.write_fixed(args[2], if val == 0i64 { 1i64 } else { 0i64 });
    None
  });
  cpu!(getOrM => fn(args, hand_mem) {
    let fractal = hand_mem.read_fractal(args[0]);
    let val = fractal.read_fixed(0);
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
          hand_mem.write_fixed(args[2], data.read_fixed(0));
        }
      }
    }
    None
  });
  cpu!(okR => fn(args, hand_mem) {
    hand_mem.init_fractal(args[2]);
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
  cpu!(err => fn(args, hand_mem) {
    hand_mem.init_fractal(args[2]);
    hand_mem.push_fixed(args[2], 0i64);
    hand_mem.push_register(args[2], args[0]);
    None
  });
  cpu!(isOk => fn(args, hand_mem) {
    hand_mem.register_out(args[0], 0, args[2]);
    None
  });
  cpu!(isErr => fn(args, hand_mem) {
    let fractal = hand_mem.read_fractal(args[0]);
    let val = fractal.read_fixed(0);
    hand_mem.write_fixed(args[2], if val == 0i64 { 1i64 } else { 0i64 });
    None
  });
  cpu!(getOrR => fn(args, hand_mem) {
    let fractal = hand_mem.read_fractal(args[0]);
    let val = fractal.read_fixed(0);
    if val == 1i64 {
      hand_mem.register_out(args[0], 1, args[2]);
    } else {
      let (data, is_fractal) = hand_mem.read_either(args[1]);
      if is_fractal {
        hand_mem.register(args[2], args[1], true);
      } else {
        hand_mem.write_fixed(args[2], data.read_fixed(0));
      }
    }
    None
  });
  cpu!(getOrRS => fn(args, hand_mem) {
    let fractal = hand_mem.read_fractal(args[0]);
    let val = fractal.read_fixed(0);
    if val == 1i64 {
      hand_mem.register_out(args[0], 1, args[2]);
    } else {
      let f = HandlerMemory::str_to_fractal(&HandlerMemory::fractal_to_string(hand_mem.read_fractal(args[1])));
      hand_mem.write_fractal(args[2], &f);
    }
    None
  });
  cpu!(getR => fn(args, hand_mem) {
    let fractal = hand_mem.read_fractal(args[0]);
    let val = fractal.read_fixed(0);
    if val == 1i64 {
      hand_mem.register_out(args[0], 1, args[2]);
    } else {
      panic!("runtime error: illegal access");
    }
    None
  });
  cpu!(getErr => fn(args, hand_mem) {
    let fractal = hand_mem.read_fractal(args[0]);
    let val = fractal.read_fixed(0);
    if val == 0i64 {
      hand_mem.register_out(args[0], 1, args[2]);
    } else {
      let (data, is_fractal) = hand_mem.read_either(args[1]);
      if is_fractal {
        hand_mem.register(args[2], args[1], true);
      } else {
        hand_mem.write_fixed(args[2], data.read_fixed(0));
      }
    }
    None
  });
  cpu!(resfrom => fn(args, hand_mem) {
    // args = [arr_addr, arr_idx_addr, outer_addr]
    // a guarded copy of data from an array to a result object
    hand_mem.init_fractal(args[2]);
    let fractal = hand_mem.read_fractal(args[1]);
    let val = fractal.read_fixed(0);
    if val == 0 {
      hand_mem.write_fractal(args[2], &fractal);
      return None;
    }
    let inner_addr = fractal.read_fixed(1) as usize;
    let arr = hand_mem.read_fractal(args[0]);
    if arr.len() > inner_addr {
      hand_mem.push_fixed(args[2], 1);
      hand_mem.push_register_out(args[2], &arr, inner_addr);
    } else {
      hand_mem.push_fixed(args[2], 0);
      hand_mem.push_fractal(args[2], HandlerMemory::str_to_fractal("out-of-bounds access"));
    }
    None
  });
  cpu!(mainE => fn(args, hand_mem) {
    hand_mem.init_fractal(args[2]);
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
  cpu!(altE => fn(args, hand_mem) {
    hand_mem.init_fractal(args[2]);
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
  cpu!(isMain => fn(args, hand_mem) {
    hand_mem.register_out(args[0], 0, args[2]);
    None
  });
  cpu!(isAlt => fn(args, hand_mem) {
    let fractal = hand_mem.read_fractal(args[0]);
    let val = fractal.read_fixed(0);
    hand_mem.write_fixed(args[2], if val == 0i64 { 1i64 } else { 0i64 });
    None
  });
  cpu!(mainOr => fn(args, hand_mem) {
    let fractal = hand_mem.read_fractal(args[0]);
    let val = fractal.read_fixed(0);
    if val == 1i64 {
      hand_mem.register_out(args[0], 1, args[2]);
    } else {
      let (data, is_fractal) = hand_mem.read_either(args[1]);
      if is_fractal {
        hand_mem.register(args[2], args[1], true);
      } else {
        hand_mem.write_fixed(args[2], data.read_fixed(0));
      }
    }
    None
  });
  cpu!(altOr => fn(args, hand_mem) {
    let fractal = hand_mem.read_fractal(args[0]);
    let val = fractal.read_fixed(0);
    if val == 0i64 {
      hand_mem.register_out(args[0], 1, args[2]);
    } else {
      let (data, is_fractal) = hand_mem.read_either(args[1]);
      if is_fractal {
        hand_mem.register(args[2], args[1], true);
      } else {
        hand_mem.write_fixed(args[2], data.read_fixed(0));
      }
    }
    None
  });

  cpu!(hashf => fn(args, hand_mem) {
    let val = hand_mem.read_fixed(args[0]);
    let mut hasher = XxHash64::with_seed(0xfa57);
    hasher.write_i64(val);
    let out = i64::from_ne_bytes(hasher.finish().to_ne_bytes());
    hand_mem.write_fixed(args[2], out);
    None
  });

  cpu!(hashv => fn(args, hand_mem) {
    let mut hasher = XxHash64::with_seed(0xfa57);
    let addr = args[0];
    if addr < 0 {
      // It's a string!
      let pascal_string = hand_mem.read_fractal(args[0]);
      let strlen = pascal_string.read_fixed(0) as f64;
      let intlen = 1 + (strlen / 8.0).ceil() as usize;
      for i in 0..intlen {
        hasher.write_i64(pascal_string.read_fixed(i));
      }
    } else {
      let mut stack: Vec<FractalMemory> = vec![hand_mem.read_fractal(args[0])];
      while stack.len() > 0 {
        let fractal = stack.pop().unwrap();
        for i in 0..fractal.len() {
          let (data, is_fractal) = hand_mem.read_from_fractal(&fractal, i);
          if is_fractal {
            stack.push(data);
          } else {
            hasher.write_i64(data.read_fixed(0));
          }
        }
      }
    }
    let out = i64::from_ne_bytes(hasher.finish().to_ne_bytes());
    hand_mem.write_fixed(args[2], out);
    None
  });

  // king opcode
  cpu!(emit => fn(args, hand_mem) {
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
      panic!(format!(
        "Illegal byte opcode {} ({})",
        v,
        str::from_utf8(&v.to_ne_bytes()).unwrap()
      ));
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
