use futures::future::join_all;
use std::collections::HashMap;
use std::convert::Infallible;
use std::fmt::Debug;
use std::future::Future;
use std::hash::Hasher;
use std::io::{self, Write};
use std::pin::Pin;
use std::ptr::NonNull;
use std::str;
use std::sync::{Arc, RwLock};
use std::time::Duration;

use byteorder::{ByteOrder, LittleEndian};
use dashmap::DashMap;
use hyper::header::{HeaderName, HeaderValue};
use hyper::{client::ResponseFuture, Body, Request, Response, StatusCode};
use once_cell::sync::Lazy;
use rand::{thread_rng, Rng};
use regex::Regex;
use tokio::process::Command;
use tokio::sync::oneshot::{self, Receiver, Sender};
use tokio::time::sleep;
use twox_hash::XxHash64;

use crate::daemon::ctrl::NAIVE_CLIENT;
use crate::daemon::daemon::{CLUSTER_SECRET, CONTROL_PORT_CHANNEL};
use crate::vm::event::{BuiltInEvents, EventEmit, HandlerFragment, NOP_ID};
use crate::vm::http::HTTP_CLIENT;
use crate::vm::memory::{FractalMemory, HandlerMemory, CLOSURE_ARG_MEM_START};
use crate::vm::program::Program;
use crate::vm::run::EVENT_TX;
use crate::vm::{VMError, VMResult};

pub static DS: Lazy<Arc<DashMap<String, Arc<HandlerMemory>>>> =
  Lazy::new(|| Arc::new(DashMap::<String, Arc<HandlerMemory>>::new()));

// used for load balancing in the cluster
pub static REGION_VMS: Lazy<Arc<RwLock<Vec<String>>>> =
  Lazy::new(|| Arc::new(RwLock::new(Vec::new())));

// type aliases
/// Futures implement an Unpin marker that guarantees to the compiler that the future will not move while it is running
/// so it can be polled. If it is moved, the implementation would be unsafe. We have to manually pin the future because
/// we are creating it dynamically. We must also specify that the `Box`ed Future can be moved across threads with a `+ Send`.
/// For more information see:
/// https://stackoverflow.com/questions/58354633/cannot-use-impl-future-to-store-async-function-in-a-vector
/// https://stackoverflow.com/questions/51485410/unable-to-tokiorun-a-boxed-future-because-the-trait-bound-send-is-not-satisfie
pub type HMFuture = Pin<Box<dyn Future<Output = VMResult<Arc<HandlerMemory>>> + Send>>;

/// A function to be run for an opcode.
pub(crate) enum OpcodeFn {
  Cpu(fn(&[i64], &mut Arc<HandlerMemory>) -> VMResult<Option<EventEmit>>),
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
      fn $name($args: &[i64], $hand_mem: &mut Arc<HandlerMemory>) -> VMResult<Option<EventEmit>> {
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
    let out = hand_mem.read_fixed(args[0])? as f64;
    hand_mem.write_fixed(args[2], i64::from_ne_bytes(out.to_ne_bytes()))?;
    Ok(None)
  });
  cpu!(i16f64 => fn(args, hand_mem) {
    let out = hand_mem.read_fixed(args[0])? as f64;
    hand_mem.write_fixed(args[2], i64::from_ne_bytes(out.to_ne_bytes()))?;
    Ok(None)
  });
  cpu!(i32f64 => fn(args, hand_mem) {
    let out = hand_mem.read_fixed(args[0])? as f64;
    hand_mem.write_fixed(args[2], i64::from_ne_bytes(out.to_ne_bytes()))?;
    Ok(None)
  });
  cpu!(i64f64 => fn(args, hand_mem) {
    let out = hand_mem.read_fixed(args[0])? as f64;
    hand_mem.write_fixed(args[2], i64::from_ne_bytes(out.to_ne_bytes()))?;
    Ok(None)
  });
  cpu!(f32f64 => fn(args, hand_mem) {
    let out = f32::from_ne_bytes((hand_mem.read_fixed(args[0])? as i32).to_ne_bytes());
    hand_mem.write_fixed(args[2], i32::from_ne_bytes(out.to_ne_bytes()) as i64)?;
    Ok(None)
  });
  cpu!(strf64 => fn(args, hand_mem) {
    let s = HandlerMemory::fractal_to_string(hand_mem.read_fractal(args[0])?)?;
    let out: f64 = s.parse().unwrap();
    hand_mem.write_fixed(args[2], i64::from_ne_bytes(out.to_ne_bytes()))?;
    Ok(None)
  });
  cpu!(boolf64 => fn(args, hand_mem) {
    let out = hand_mem.read_fixed(args[0])? as f64;
    hand_mem.write_fixed(args[2], i64::from_ne_bytes(out.to_ne_bytes()))?;
    Ok(None)
  });

  cpu!(i8f32 => fn(args, hand_mem) {
    let num = hand_mem.read_fixed(args[0])? as f32;
    let out = i32::from_ne_bytes(num.to_ne_bytes()) as i64;
    hand_mem.write_fixed(args[2], out)?;
    Ok(None)
  });
  cpu!(i16f32 => fn(args, hand_mem) {
    let num = hand_mem.read_fixed(args[0])? as f32;
    let out = i32::from_ne_bytes(num.to_ne_bytes()) as i64;
    hand_mem.write_fixed(args[2], out)?;
    Ok(None)
  });
  cpu!(i32f32 => fn(args, hand_mem) {
    let num = hand_mem.read_fixed(args[0])? as f32;
    let out = i32::from_ne_bytes(num.to_ne_bytes()) as i64;
    hand_mem.write_fixed(args[2], out)?;
    Ok(None)
  });
  cpu!(i64f32 => fn(args, hand_mem) {
    let num = hand_mem.read_fixed(args[0])? as f32;
    let out = i32::from_ne_bytes(num.to_ne_bytes()) as i64;
    hand_mem.write_fixed(args[2], out)?;
    Ok(None)
  });
  cpu!(f64f32 => fn(args, hand_mem) {
    let num = f64::from_ne_bytes(hand_mem.read_fixed(args[0])?.to_ne_bytes()) as f32;
    let out = i32::from_ne_bytes(num.to_ne_bytes()) as i64;
    hand_mem.write_fixed(args[2], out)?;
    Ok(None)
  });
  cpu!(strf32 => fn(args, hand_mem) {
    let s = HandlerMemory::fractal_to_string(hand_mem.read_fractal(args[0])?)?;
    let num: f32 = s.parse().unwrap();
    let out = i32::from_ne_bytes(num.to_ne_bytes()) as i64;
    hand_mem.write_fixed(args[2], out)?;
    Ok(None)
  });
  cpu!(boolf32 => fn(args, hand_mem) {
    let num = hand_mem.read_fixed(args[0])? as f32;
    let out = i32::from_ne_bytes(num.to_ne_bytes()) as i64;
    hand_mem.write_fixed(args[2], out)?;
    Ok(None)
  });

  cpu!(i8i64 => fn(args, hand_mem) {
    let out = hand_mem.read_fixed(args[0])?;
    hand_mem.write_fixed(args[2], out)?;
    Ok(None)
  });
  cpu!(i16i64 => fn(args, hand_mem) {
    let out = hand_mem.read_fixed(args[0])?;
    hand_mem.write_fixed(args[2], out)?;
    Ok(None)
  });
  cpu!(i32i64 => fn(args, hand_mem) {
    let out = hand_mem.read_fixed(args[0])?;
    hand_mem.write_fixed(args[2], out)?;
    Ok(None)
  });
  cpu!(f64i64 => fn(args, hand_mem) {
    let out = f64::from_ne_bytes(hand_mem.read_fixed(args[0])?.to_ne_bytes()) as i64;
    hand_mem.write_fixed(args[2], out)?;
    Ok(None)
  });
  cpu!(f32i64 => fn(args, hand_mem) {
    let out = f32::from_ne_bytes((hand_mem.read_fixed(args[0])? as i32).to_ne_bytes()) as i64;
    hand_mem.write_fixed(args[2], out)?;
    Ok(None)
  });
  cpu!(stri64 => fn(args, hand_mem) {
    let s = HandlerMemory::fractal_to_string(hand_mem.read_fractal(args[0])?)?;
    let out: i64 = s.parse().unwrap();
    hand_mem.write_fixed(args[2], out)?;
    Ok(None)
  });
  cpu!(booli64 => fn(args, hand_mem) {
    let out = hand_mem.read_fixed(args[0])?;
    hand_mem.write_fixed(args[2], out)?;
    Ok(None)
  });

  cpu!(i8i32 => fn(args, hand_mem) {
    let out = hand_mem.read_fixed(args[0])?;
    hand_mem.write_fixed(args[2], out)?;
    Ok(None)
  });
  cpu!(i16i32 => fn(args, hand_mem) {
    let out = hand_mem.read_fixed(args[0])?;
    hand_mem.write_fixed(args[2], out)?;
    Ok(None)
  });
  cpu!(i64i32 => fn(args, hand_mem) {
    let out = (hand_mem.read_fixed(args[0])? as i32) as i64;
    hand_mem.write_fixed(args[2], out)?;
    Ok(None)
  });
  cpu!(f64i32 => fn(args, hand_mem) {
    let out = (f64::from_ne_bytes(hand_mem.read_fixed(args[0])?.to_ne_bytes()) as i32) as i64;
    hand_mem.write_fixed(args[2], out)?;
    Ok(None)
  });
  cpu!(f32i32 => fn(args, hand_mem) {
    let out = (f32::from_ne_bytes((hand_mem.read_fixed(args[0])? as i32).to_ne_bytes()) as i32) as i64;
    hand_mem.write_fixed(args[2], out)?;
    Ok(None)
  });
  cpu!(stri32 => fn(args, hand_mem) {
    let s = HandlerMemory::fractal_to_string(hand_mem.read_fractal(args[0])?)?;
    let num: i32 = s.parse().unwrap();
    let out = num as i64;
    hand_mem.write_fixed(args[2], out)?;
    Ok(None)
  });
  cpu!(booli32 => fn(args, hand_mem) {
    let out = hand_mem.read_fixed(args[0])?;
    hand_mem.write_fixed(args[2], out)?;
    Ok(None)
  });

  cpu!(i8i16 => fn(args, hand_mem) {
    let out = hand_mem.read_fixed(args[0])?;
    hand_mem.write_fixed(args[2], out)?;
    Ok(None)
  });
  cpu!(i32i16 => fn(args, hand_mem) {
    let out = (hand_mem.read_fixed(args[0])? as i16) as i64;
    hand_mem.write_fixed(args[2], out)?;
    Ok(None)
  });
  cpu!(i64i16 => fn(args, hand_mem) {
    let out = (hand_mem.read_fixed(args[0])? as i16) as i64;
    hand_mem.write_fixed(args[2], out)?;
    Ok(None)
  });
  cpu!(f64i16 => fn(args, hand_mem) {
    let out = (f64::from_ne_bytes(hand_mem.read_fixed(args[0])?.to_ne_bytes()) as i16) as i64;
    hand_mem.write_fixed(args[2], out)?;
    Ok(None)
  });
  cpu!(f32i16 => fn(args, hand_mem) {
    let out = (f32::from_ne_bytes((hand_mem.read_fixed(args[0])? as i32).to_ne_bytes()) as i16) as i64;
    hand_mem.write_fixed(args[2], out)?;
    Ok(None)
  });
  cpu!(stri16 => fn(args, hand_mem) {
    let s = HandlerMemory::fractal_to_string(hand_mem.read_fractal(args[0])?)?;
    let num: i16 = s.parse().unwrap();
    let out = num as i64;
    hand_mem.write_fixed(args[2], out)?;
    Ok(None)
  });
  cpu!(booli16 => fn(args, hand_mem) {
    let out = hand_mem.read_fixed(args[0])?;
    hand_mem.write_fixed(args[2], out)?;
    Ok(None)
  });

  cpu!(i16i8 => fn(args, hand_mem) {
    let out = (hand_mem.read_fixed(args[0])? as i8) as i64;
    hand_mem.write_fixed(args[2], out)?;
    Ok(None)
  });
  cpu!(i32i8 => fn(args, hand_mem) {
    let out = (hand_mem.read_fixed(args[0])? as i8) as i64;
    hand_mem.write_fixed(args[2], out)?;
    Ok(None)
  });
  cpu!(i64i8 => fn(args, hand_mem) {
    let out = (hand_mem.read_fixed(args[0])? as i8) as i64;
    hand_mem.write_fixed(args[2], out)?;
    Ok(None)
  });
  cpu!(f64i8 => fn(args, hand_mem) {
    let out = (f64::from_ne_bytes(hand_mem.read_fixed(args[0])?.to_ne_bytes()) as i8) as i64;
    hand_mem.write_fixed(args[2], out)?;
    Ok(None)
  });
  cpu!(f32i8 => fn(args, hand_mem) {
    let out = (f32::from_ne_bytes((hand_mem.read_fixed(args[0])? as i32).to_ne_bytes()) as i8) as i64;
    hand_mem.write_fixed(args[2], out)?;
    Ok(None)
  });
  cpu!(stri8 => fn(args, hand_mem) {
    let s = HandlerMemory::fractal_to_string(hand_mem.read_fractal(args[0])?)?;
    let num: i8 = s.parse().unwrap();
    let out = num as i64;
    hand_mem.write_fixed(args[2], out)?;
    Ok(None)
  });
  cpu!(booli8 => fn(args, hand_mem) {
    let out = (hand_mem.read_fixed(args[0])? as i8) as i64;
    hand_mem.write_fixed(args[2], out)?;
    Ok(None)
  });

  cpu!(i8bool => fn(args, hand_mem) {
    let a = hand_mem.read_fixed(args[0])? as i8;
    let out = if a != 0 { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out)?;
    Ok(None)
  });
  cpu!(i16bool => fn(args, hand_mem) {
    let a = hand_mem.read_fixed(args[0])? as i16;
    let out = if a != 0 { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out)?;
    Ok(None)
  });
  cpu!(i32bool => fn(args, hand_mem) {
    let a = hand_mem.read_fixed(args[0])? as i32;
    let out = if a != 0 { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out)?;
    Ok(None)
  });
  cpu!(i64bool => fn(args, hand_mem) {
    let a = hand_mem.read_fixed(args[0])?;
    let out = if a != 0 { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out)?;
    Ok(None)
  });
  cpu!(f64bool => fn(args, hand_mem) {
    let a = f64::from_ne_bytes(hand_mem.read_fixed(args[0])?.to_ne_bytes());
    let out = if a != 0.0 { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out)?;
    Ok(None)
  });
  cpu!(f32bool => fn(args, hand_mem) {
    let a = f32::from_ne_bytes((hand_mem.read_fixed(args[0])? as i32).to_ne_bytes());
    let out = if a != 0.0 { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out)?;
    Ok(None)
  });
  cpu!(strbool => fn(args, hand_mem) {
    let s = HandlerMemory::fractal_to_string(hand_mem.read_fractal(args[0])?)?;
    let out = if s == "true" { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out)?;
    Ok(None)
  });

  cpu!(i8str => fn(args, hand_mem) {
    let a = hand_mem.read_fixed(args[0])? as i8;
    let a_str = a.to_string();
    hand_mem.write_fractal(args[2], &HandlerMemory::str_to_fractal(&a_str))?;
    Ok(None)
  });
  cpu!(i16str => fn(args, hand_mem) {
    let a = hand_mem.read_fixed(args[0])? as i16;
    let a_str = a.to_string();
    hand_mem.write_fractal(args[2], &HandlerMemory::str_to_fractal(&a_str))?;
    Ok(None)
  });
  cpu!(i32str => fn(args, hand_mem) {
    let a = hand_mem.read_fixed(args[0])? as i32;
    let a_str = a.to_string();
    hand_mem.write_fractal(args[2], &HandlerMemory::str_to_fractal(&a_str))?;
    Ok(None)
  });
  cpu!(i64str => fn(args, hand_mem) {
    let a = hand_mem.read_fixed(args[0])?;
    let a_str = a.to_string();
    hand_mem.write_fractal(args[2], &HandlerMemory::str_to_fractal(&a_str))?;
    Ok(None)
  });
  cpu!(f64str => fn(args, hand_mem) {
    let a = f64::from_ne_bytes(hand_mem.read_fixed(args[0])?.to_ne_bytes());
    let a_str = a.to_string();
    hand_mem.write_fractal(args[2], &HandlerMemory::str_to_fractal(&a_str))?;
    Ok(None)
  });
  cpu!(f32str => fn(args, hand_mem) {
    let a = f32::from_ne_bytes((hand_mem.read_fixed(args[0])? as i32).to_ne_bytes());
    let a_str = a.to_string();
    hand_mem.write_fractal(args[2], &HandlerMemory::str_to_fractal(&a_str))?;
    Ok(None)
  });
  cpu!(boolstr => fn(args, hand_mem) {
    let a = hand_mem.read_fixed(args[0])?;
    let a_str = if a == 1 { "true" } else { "false" };
    hand_mem.write_fractal(args[2], &HandlerMemory::str_to_fractal(&a_str))?;
    Ok(None)
  });

  // Arithmetic opcodes
  cpu!(addi8 => fn(args, hand_mem) {
    let ra = hand_mem.read_fractal(args[0])?;
    let rb = hand_mem.read_fractal(args[1])?;
    if ra.read_fixed(0)? == 0 {
      hand_mem.write_fractal(args[2], &ra)?;
      return Ok(None);
    }
    if rb.read_fixed(0)? == 0 {
      hand_mem.write_fractal(args[2], &rb)?;
      return Ok(None);
    }
    let a = ra.read_fixed(1)? as i8;
    let b = rb.read_fixed(1)? as i8;
    hand_mem.init_fractal(args[2])?;
    if a > 0 && b > 0 && a > std::i8::MAX - b {
      hand_mem.push_fixed(args[2], 0)?;
      hand_mem.push_fractal(args[2], HandlerMemory::str_to_fractal("overflow"))?;
      return Ok(None);
    }
    if a < 0 && b < 0 && a < std::i8::MIN - b {
      hand_mem.push_fixed(args[2], 0)?;
      hand_mem.push_fractal(args[2], HandlerMemory::str_to_fractal("underflow"))?;
      return Ok(None);
    }
    hand_mem.push_fixed(args[2], 1)?;
    hand_mem.push_fixed(args[2], (a + b) as i64)?;
    Ok(None)
  });
  cpu!(addi16 => fn(args, hand_mem) {
    let ra = hand_mem.read_fractal(args[0])?;
    let rb = hand_mem.read_fractal(args[1])?;
    if ra.read_fixed(0)? == 0 {
      hand_mem.write_fractal(args[2], &ra)?;
      return Ok(None);
    }
    if rb.read_fixed(0)? == 0 {
      hand_mem.write_fractal(args[2], &rb)?;
      return Ok(None);
    }
    let a = ra.read_fixed(1)? as i16;
    let b = rb.read_fixed(1)? as i16;
    hand_mem.init_fractal(args[2])?;
    if a > 0 && b > 0 && a > std::i16::MAX - b {
      hand_mem.push_fixed(args[2], 0)?;
      hand_mem.push_fractal(args[2], HandlerMemory::str_to_fractal("overflow"))?;
      return Ok(None);
    }
    if a < 0 && b < 0 && a < std::i16::MIN - b {
      hand_mem.push_fixed(args[2], 0)?;
      hand_mem.push_fractal(args[2], HandlerMemory::str_to_fractal("underflow"))?;
      return Ok(None);
    }
    hand_mem.push_fixed(args[2], 1)?;
    hand_mem.push_fixed(args[2], (a + b) as i64)?;
    Ok(None)
  });
  cpu!(addi32 => fn(args, hand_mem) {
    let ra = hand_mem.read_fractal(args[0])?;
    let rb = hand_mem.read_fractal(args[1])?;
    if ra.read_fixed(0)? == 0 {
      hand_mem.write_fractal(args[2], &ra)?;
      return Ok(None);
    }
    if rb.read_fixed(0)? == 0 {
      hand_mem.write_fractal(args[2], &rb)?;
      return Ok(None);
    }
    let a = ra.read_fixed(1)? as i32;
    let b = rb.read_fixed(1)? as i32;
    hand_mem.init_fractal(args[2])?;
    if a > 0 && b > 0 && a > std::i32::MAX - b {
      hand_mem.push_fixed(args[2], 0)?;
      hand_mem.push_fractal(args[2], HandlerMemory::str_to_fractal("overflow"))?;
      return Ok(None);
    }
    if a < 0 && b < 0 && a < std::i32::MIN - b {
      hand_mem.push_fixed(args[2], 0)?;
      hand_mem.push_fractal(args[2], HandlerMemory::str_to_fractal("underflow"))?;
      return Ok(None);
    }
    hand_mem.push_fixed(args[2], 1)?;
    hand_mem.push_fixed(args[2], (a + b) as i64)?;
    Ok(None)
  });
  cpu!(addi64 => fn(args, hand_mem) {
    let ra = hand_mem.read_fractal(args[0])?;
    let rb = hand_mem.read_fractal(args[1])?;
    if ra.read_fixed(0)? == 0 {
      hand_mem.write_fractal(args[2], &ra)?;
      return Ok(None);
    }
    if rb.read_fixed(0)? == 0 {
      hand_mem.write_fractal(args[2], &rb)?;
      return Ok(None);
    }
    let a = ra.read_fixed(1)? as i64;
    let b = rb.read_fixed(1)? as i64;
    hand_mem.init_fractal(args[2])?;
    if a > 0 && b > 0 && a > std::i64::MAX - b {
      hand_mem.push_fixed(args[2], 0)?;
      hand_mem.push_fractal(args[2], HandlerMemory::str_to_fractal("overflow"))?;
      return Ok(None);
    }
    if a < 0 && b < 0 && a < std::i64::MIN - b {
      hand_mem.push_fixed(args[2], 0)?;
      hand_mem.push_fractal(args[2], HandlerMemory::str_to_fractal("underflow"))?;
      return Ok(None);
    }
    hand_mem.push_fixed(args[2], 1)?;
    hand_mem.push_fixed(args[2], (a + b) as i64)?;
    Ok(None)
  });
  cpu!(addf32 => fn(args, hand_mem) {
    let ra = hand_mem.read_fractal(args[0])?;
    let rb = hand_mem.read_fractal(args[1])?;
    if ra.read_fixed(0)? == 0 {
      hand_mem.write_fractal(args[2], &ra)?;
      return Ok(None);
    }
    if rb.read_fixed(0)? == 0 {
      hand_mem.write_fractal(args[2], &rb)?;
      return Ok(None);
    }
    let a = f32::from_ne_bytes((ra.read_fixed(1)? as i32).to_ne_bytes());
    let b = f32::from_ne_bytes((rb.read_fixed(1)? as i32).to_ne_bytes());
    let out = a + b;
    hand_mem.init_fractal(args[2])?;
    if out == std::f32::INFINITY {
      hand_mem.push_fixed(args[2], 0)?;
      hand_mem.push_fractal(args[2], HandlerMemory::str_to_fractal("overflow"))?;
      return Ok(None);
    }
    if out == std::f32::NEG_INFINITY {
      hand_mem.push_fixed(args[2], 0)?;
      hand_mem.push_fractal(args[2], HandlerMemory::str_to_fractal("underflow"))?;
      return Ok(None);
    }
    let num = i32::from_ne_bytes(out.to_ne_bytes()) as i64;
    hand_mem.push_fixed(args[2], 1)?;
    hand_mem.push_fixed(args[2], num)?;
    Ok(None)
  });
  cpu!(addf64 => fn(args, hand_mem) {
    let ra = hand_mem.read_fractal(args[0])?;
    let rb = hand_mem.read_fractal(args[1])?;
    if ra.read_fixed(0)? == 0 {
      hand_mem.write_fractal(args[2], &ra)?;
      return Ok(None);
    }
    if rb.read_fixed(0)? == 0 {
      hand_mem.write_fractal(args[2], &rb)?;
      return Ok(None);
    }
    let a = f64::from_ne_bytes(ra.read_fixed(1)?.to_ne_bytes());
    let b = f64::from_ne_bytes(rb.read_fixed(1)?.to_ne_bytes());
    let out = a + b;
    hand_mem.init_fractal(args[2])?;
    if out == std::f64::INFINITY {
      hand_mem.push_fixed(args[2], 0)?;
      hand_mem.push_fractal(args[2], HandlerMemory::str_to_fractal("overflow"))?;
      return Ok(None);
    }
    if out == std::f64::NEG_INFINITY {
      hand_mem.push_fixed(args[2], 0)?;
      hand_mem.push_fractal(args[2], HandlerMemory::str_to_fractal("underflow"))?;
      return Ok(None);
    }
    let num = i64::from_ne_bytes(out.to_ne_bytes());
    hand_mem.push_fixed(args[2], 1)?;
    hand_mem.push_fixed(args[2], num)?;
    Ok(None)
  });

  cpu!(subi8 => fn(args, hand_mem) {
    let ra = hand_mem.read_fractal(args[0])?;
    let rb = hand_mem.read_fractal(args[1])?;
    if ra.read_fixed(0)? == 0 {
      hand_mem.write_fractal(args[2], &ra)?;
      return Ok(None);
    }
    if rb.read_fixed(0)? == 0 {
      hand_mem.write_fractal(args[2], &rb)?;
      return Ok(None);
    }
    let a = ra.read_fixed(1)? as i8;
    let b = rb.read_fixed(1)? as i8;
    hand_mem.init_fractal(args[2])?;
    if a > 0 && b < 0 && a > std::i8::MAX + b {
      hand_mem.push_fixed(args[2], 0)?;
      hand_mem.push_fractal(args[2], HandlerMemory::str_to_fractal("overflow"))?;
      return Ok(None);
    }
    if a < 0 && b > 0 && a < std::i8::MIN + b {
      hand_mem.push_fixed(args[2], 0)?;
      hand_mem.push_fractal(args[2], HandlerMemory::str_to_fractal("underflow"))?;
      return Ok(None);
    }
    hand_mem.push_fixed(args[2], 1)?;
    hand_mem.push_fixed(args[2], (a - b) as i64)?;
    Ok(None)
  });
  cpu!(subi16 => fn(args, hand_mem) {
    let ra = hand_mem.read_fractal(args[0])?;
    let rb = hand_mem.read_fractal(args[1])?;
    if ra.read_fixed(0)? == 0 {
      hand_mem.write_fractal(args[2], &ra)?;
      return Ok(None);
    }
    if rb.read_fixed(0)? == 0 {
      hand_mem.write_fractal(args[2], &rb)?;
      return Ok(None);
    }
    let a = ra.read_fixed(1)? as i16;
    let b = rb.read_fixed(1)? as i16;
    hand_mem.init_fractal(args[2])?;
    if a > 0 && b < 0 && a > std::i16::MAX + b {
      hand_mem.push_fixed(args[2], 0)?;
      hand_mem.push_fractal(args[2], HandlerMemory::str_to_fractal("overflow"))?;
      return Ok(None);
    }
    if a < 0 && b > 0 && a < std::i16::MIN + b {
      hand_mem.push_fixed(args[2], 0)?;
      hand_mem.push_fractal(args[2], HandlerMemory::str_to_fractal("underflow"))?;
      return Ok(None);
    }
    hand_mem.push_fixed(args[2], 1)?;
    hand_mem.push_fixed(args[2], (a - b) as i64)?;
    Ok(None)
  });
  cpu!(subi32 => fn(args, hand_mem) {
    let ra = hand_mem.read_fractal(args[0])?;
    let rb = hand_mem.read_fractal(args[1])?;
    if ra.read_fixed(0)? == 0 {
      hand_mem.write_fractal(args[2], &ra)?;
      return Ok(None);
    }
    if rb.read_fixed(0)? == 0 {
      hand_mem.write_fractal(args[2], &rb)?;
      return Ok(None);
    }
    let a = ra.read_fixed(1)? as i32;
    let b = rb.read_fixed(1)? as i32;
    hand_mem.init_fractal(args[2])?;
    if a > 0 && b < 0 && a > std::i32::MAX + b {
      hand_mem.push_fixed(args[2], 0)?;
      hand_mem.push_fractal(args[2], HandlerMemory::str_to_fractal("overflow"))?;
      return Ok(None);
    }
    if a < 0 && b > 0 && a < std::i32::MIN + b {
      hand_mem.push_fixed(args[2], 0)?;
      hand_mem.push_fractal(args[2], HandlerMemory::str_to_fractal("underflow"))?;
      return Ok(None);
    }
    hand_mem.push_fixed(args[2], 1)?;
    hand_mem.push_fixed(args[2], (a - b) as i64)?;
    Ok(None)
  });
  cpu!(subi64 => fn(args, hand_mem) {
    let ra = hand_mem.read_fractal(args[0])?;
    let rb = hand_mem.read_fractal(args[1])?;
    if ra.read_fixed(0)? == 0 {
      hand_mem.write_fractal(args[2], &ra)?;
      return Ok(None);
    }
    if rb.read_fixed(0)? == 0 {
      hand_mem.write_fractal(args[2], &rb)?;
      return Ok(None);
    }
    let a = ra.read_fixed(1)? as i64;
    let b = rb.read_fixed(1)? as i64;
    hand_mem.init_fractal(args[2])?;
    if a > 0 && b < 0 && a > std::i64::MAX + b {
      hand_mem.push_fixed(args[2], 0)?;
      hand_mem.push_fractal(args[2], HandlerMemory::str_to_fractal("overflow"))?;
      return Ok(None);
    }
    if a < 0 && b > 0 && a < std::i64::MIN + b {
      hand_mem.push_fixed(args[2], 0)?;
      hand_mem.push_fractal(args[2], HandlerMemory::str_to_fractal("underflow"))?;
      return Ok(None);
    }
    hand_mem.push_fixed(args[2], 1)?;
    hand_mem.push_fixed(args[2], a - b)?;
    Ok(None)
  });
  cpu!(subf32 => fn(args, hand_mem) {
    let ra = hand_mem.read_fractal(args[0])?;
    let rb = hand_mem.read_fractal(args[1])?;
    if ra.read_fixed(0)? == 0 {
      hand_mem.write_fractal(args[2], &ra)?;
      return Ok(None);
    }
    if rb.read_fixed(0)? == 0 {
      hand_mem.write_fractal(args[2], &rb)?;
      return Ok(None);
    }
    let a = f32::from_ne_bytes((ra.read_fixed(1)? as i32).to_ne_bytes());
    let b = f32::from_ne_bytes((rb.read_fixed(1)? as i32).to_ne_bytes());
    let out = a - b;
    hand_mem.init_fractal(args[2])?;
    if out == std::f32::INFINITY {
      hand_mem.push_fixed(args[2], 0)?;
      hand_mem.push_fractal(args[2], HandlerMemory::str_to_fractal("overflow"))?;
      return Ok(None);
    }
    if out == std::f32::NEG_INFINITY {
      hand_mem.push_fixed(args[2], 0)?;
      hand_mem.push_fractal(args[2], HandlerMemory::str_to_fractal("underflow"))?;
      return Ok(None);
    }
    let num = i32::from_ne_bytes(out.to_ne_bytes()) as i64;
    hand_mem.push_fixed(args[2], 1)?;
    hand_mem.push_fixed(args[2], num)?;
    Ok(None)
  });
  cpu!(subf64 => fn(args, hand_mem) {
    let ra = hand_mem.read_fractal(args[0])?;
    let rb = hand_mem.read_fractal(args[1])?;
    if ra.read_fixed(0)? == 0 {
      hand_mem.write_fractal(args[2], &ra)?;
      return Ok(None);
    }
    if rb.read_fixed(0)? == 0 {
      hand_mem.write_fractal(args[2], &rb)?;
      return Ok(None);
    }
    let a = f64::from_ne_bytes(ra.read_fixed(1)?.to_ne_bytes());
    let b = f64::from_ne_bytes(rb.read_fixed(1)?.to_ne_bytes());
    let out = a - b;
    hand_mem.init_fractal(args[2])?;
    if out == std::f64::INFINITY {
      hand_mem.push_fixed(args[2], 0)?;
      hand_mem.push_fractal(args[2], HandlerMemory::str_to_fractal("overflow"))?;
      return Ok(None);
    }
    if out == std::f64::NEG_INFINITY {
      hand_mem.push_fixed(args[2], 0)?;
      hand_mem.push_fractal(args[2], HandlerMemory::str_to_fractal("underflow"))?;
      return Ok(None);
    }
    let num = i64::from_ne_bytes(out.to_ne_bytes());
    hand_mem.push_fixed(args[2], 1)?;
    hand_mem.push_fixed(args[2], num)?;
    Ok(None)
  });

  cpu!(negi8 => fn(args, hand_mem) {
    let ra = hand_mem.read_fractal(args[0])?;
    if ra.read_fixed(0)? == 0 {
      hand_mem.write_fractal(args[2], &ra)?;
      return Ok(None);
    }
    let a = ra.read_fixed(1)? as i8;
    hand_mem.init_fractal(args[2])?;
    if a == std::i8::MIN {
      hand_mem.push_fixed(args[2], 0)?;
      hand_mem.push_fractal(args[2], HandlerMemory::str_to_fractal("overflow"))?;
    }
    hand_mem.push_fixed(args[2], 1)?;
    hand_mem.push_fixed(args[2], (0 - a) as i64)?;
    Ok(None)
  });
  cpu!(negi16 => fn(args, hand_mem) {
    let ra = hand_mem.read_fractal(args[0])?;
    if ra.read_fixed(0)? == 0 {
      hand_mem.write_fractal(args[2], &ra)?;
      return Ok(None);
    }
    let a = ra.read_fixed(1)? as i16;
    hand_mem.init_fractal(args[2])?;
    if a == std::i16::MIN {
      hand_mem.push_fixed(args[2], 0)?;
      hand_mem.push_fractal(args[2], HandlerMemory::str_to_fractal("overflow"))?;
    }
    hand_mem.push_fixed(args[2], 1)?;
    hand_mem.push_fixed(args[2], (0 - a) as i64)?;
    Ok(None)
  });
  cpu!(negi32 => fn(args, hand_mem) {
    let ra = hand_mem.read_fractal(args[0])?;
    if ra.read_fixed(0)? == 0 {
      hand_mem.write_fractal(args[2], &ra)?;
      return Ok(None);
    }
    let a = ra.read_fixed(1)? as i32;
    hand_mem.init_fractal(args[2])?;
    if a == std::i32::MIN {
      hand_mem.push_fixed(args[2], 0)?;
      hand_mem.push_fractal(args[2], HandlerMemory::str_to_fractal("overflow"))?;
    }
    hand_mem.push_fixed(args[2], 1)?;
    hand_mem.push_fixed(args[2], (0 - a) as i64)?;
    Ok(None)
  });
  cpu!(negi64 => fn(args, hand_mem) {
    let ra = hand_mem.read_fractal(args[0])?;
    if ra.read_fixed(0)? == 0 {
      hand_mem.write_fractal(args[2], &ra)?;
      return Ok(None);
    }
    let a = ra.read_fixed(1)?;
    hand_mem.init_fractal(args[2])?;
    if a == std::i64::MIN {
      hand_mem.push_fixed(args[2], 0)?;
      hand_mem.push_fractal(args[2], HandlerMemory::str_to_fractal("overflow"))?;
    }
    hand_mem.push_fixed(args[2], 1)?;
    hand_mem.push_fixed(args[2], 0 - a)?;
    Ok(None)
  });
  cpu!(negf32 => fn(args, hand_mem) {
    let ra = hand_mem.read_fractal(args[0])?;
    if ra.read_fixed(0)? == 0 {
      hand_mem.write_fractal(args[2], &ra)?;
      return Ok(None);
    }
    let a = f32::from_ne_bytes((ra.read_fixed(1)? as i32).to_ne_bytes());
    hand_mem.init_fractal(args[2])?;
    hand_mem.push_fixed(args[2], 1)?;
    let out = i32::from_ne_bytes((0.0 - a).to_ne_bytes()) as i64;
    hand_mem.push_fixed(args[2], out)?;
    Ok(None)
  });
  cpu!(negf64 => fn(args, hand_mem) {
    let ra = hand_mem.read_fractal(args[0])?;
    if ra.read_fixed(0)? == 0 {
      hand_mem.write_fractal(args[2], &ra)?;
      return Ok(None);
    }
    let a = f64::from_ne_bytes(ra.read_fixed(1)?.to_ne_bytes());
    hand_mem.init_fractal(args[2])?;
    hand_mem.push_fixed(args[2], 1)?;
    let out = i64::from_ne_bytes((0.0 - a).to_ne_bytes());
    hand_mem.push_fixed(args[2], out)?;
    Ok(None)
  });

  cpu!(absi8 => fn(args, hand_mem) {
    let ra = hand_mem.read_fractal(args[0])?;
    if ra.read_fixed(0)? == 0 {
      hand_mem.write_fractal(args[2], &ra)?;
      return Ok(None);
    }
    let a = ra.read_fixed(1)? as i8;
    hand_mem.init_fractal(args[2])?;
    if a == std::i8::MIN {
      hand_mem.push_fixed(args[2], 0)?;
      hand_mem.push_fractal(args[2], HandlerMemory::str_to_fractal("overflow"))?;
    }
    hand_mem.push_fixed(args[2], 1)?;
    hand_mem.push_fixed(args[2], a.abs() as i64)?;
    Ok(None)
  });
  cpu!(absi16 => fn(args, hand_mem) {
    let ra = hand_mem.read_fractal(args[0])?;
    if ra.read_fixed(0)? == 0 {
      hand_mem.write_fractal(args[2], &ra)?;
      return Ok(None);
    }
    let a = ra.read_fixed(1)? as i16;
    hand_mem.init_fractal(args[2])?;
    if a == std::i16::MIN {
      hand_mem.push_fixed(args[2], 0)?;
      hand_mem.push_fractal(args[2], HandlerMemory::str_to_fractal("overflow"))?;
    }
    hand_mem.push_fixed(args[2], 1)?;
    hand_mem.push_fixed(args[2], a.abs() as i64)?;
    Ok(None)
  });
  cpu!(absi32 => fn(args, hand_mem) {
    let ra = hand_mem.read_fractal(args[0])?;
    if ra.read_fixed(0)? == 0 {
      hand_mem.write_fractal(args[2], &ra)?;
      return Ok(None);
    }
    let a = ra.read_fixed(1)? as i32;
    hand_mem.init_fractal(args[2])?;
    if a == std::i32::MIN {
      hand_mem.push_fixed(args[2], 0)?;
      hand_mem.push_fractal(args[2], HandlerMemory::str_to_fractal("overflow"))?;
    }
    hand_mem.push_fixed(args[2], 1)?;
    hand_mem.push_fixed(args[2], a.abs() as i64)?;
    Ok(None)
  });
  cpu!(absi64 => fn(args, hand_mem) {
    let ra = hand_mem.read_fractal(args[0])?;
    if ra.read_fixed(0)? == 0 {
      hand_mem.write_fractal(args[2], &ra)?;
      return Ok(None);
    }
    let a = ra.read_fixed(1)?;
    hand_mem.init_fractal(args[2])?;
    if a == std::i64::MIN {
      hand_mem.push_fixed(args[2], 0)?;
      hand_mem.push_fractal(args[2], HandlerMemory::str_to_fractal("overflow"))?;
    }
    hand_mem.push_fixed(args[2], 1)?;
    hand_mem.push_fixed(args[2], a.abs())?;
    Ok(None)
  });
  cpu!(absf32 => fn(args, hand_mem) {
    let ra = hand_mem.read_fractal(args[0])?;
    if ra.read_fixed(0)? == 0 {
      hand_mem.write_fractal(args[2], &ra)?;
      return Ok(None);
    }
    let a = f32::from_ne_bytes((ra.read_fixed(1)? as i32).to_ne_bytes());
    hand_mem.init_fractal(args[2])?;
    hand_mem.push_fixed(args[2], 1)?;
    let out = i32::from_ne_bytes(a.abs().to_ne_bytes()) as i64;
    hand_mem.push_fixed(args[2], out)?;
    Ok(None)
  });
  cpu!(absf64 => fn(args, hand_mem) {
    let ra = hand_mem.read_fractal(args[0])?;
    if ra.read_fixed(0)? == 0 {
      hand_mem.write_fractal(args[2], &ra)?;
      return Ok(None);
    }
    let a = f64::from_ne_bytes(ra.read_fixed(1)?.to_ne_bytes());
    hand_mem.init_fractal(args[2])?;
    hand_mem.push_fixed(args[2], 1)?;
    let out = i64::from_ne_bytes(a.abs().to_ne_bytes());
    hand_mem.push_fixed(args[2], out)?;
    Ok(None)
  });

  cpu!(muli8 => fn(args, hand_mem) {
    let ra = hand_mem.read_fractal(args[0])?;
    let rb = hand_mem.read_fractal(args[1])?;
    if ra.read_fixed(0)? == 0 {
      hand_mem.write_fractal(args[2], &ra)?;
      return Ok(None);
    }
    if rb.read_fixed(0)? == 0 {
      hand_mem.write_fractal(args[2], &rb)?;
      return Ok(None);
    }
    let a = ra.read_fixed(1)? as i8;
    let b = rb.read_fixed(1)? as i8;
    hand_mem.init_fractal(args[2])?;
    if a > 0 && b > 0 && (a as f64) > (std::i8::MAX as f64) / (b as f64) {
      hand_mem.push_fixed(args[2], 0)?;
      hand_mem.push_fractal(args[2], HandlerMemory::str_to_fractal("overflow"))?;
      return Ok(None);
    }
    if a < 0 && b < 0 && (a as f64) < (std::i8::MIN as f64) / (b as f64) {
      hand_mem.push_fixed(args[2], 0)?;
      hand_mem.push_fractal(args[2], HandlerMemory::str_to_fractal("underflow"))?;
      return Ok(None);
    }
    hand_mem.push_fixed(args[2], 1)?;
    hand_mem.push_fixed(args[2], (a * b) as i64)?;
    Ok(None)
  });
  cpu!(muli16 => fn(args, hand_mem) {
    let ra = hand_mem.read_fractal(args[0])?;
    let rb = hand_mem.read_fractal(args[1])?;
    if ra.read_fixed(0)? == 0 {
      hand_mem.write_fractal(args[2], &ra)?;
      return Ok(None);
    }
    if rb.read_fixed(0)? == 0 {
      hand_mem.write_fractal(args[2], &rb)?;
      return Ok(None);
    }
    let a = ra.read_fixed(1)? as i16;
    let b = rb.read_fixed(1)? as i16;
    hand_mem.init_fractal(args[2])?;
    if a > 0 && b > 0 && (a as f64) > (std::i16::MAX as f64) / (b as f64) {
      hand_mem.push_fixed(args[2], 0)?;
      hand_mem.push_fractal(args[2], HandlerMemory::str_to_fractal("overflow"))?;
      return Ok(None);
    }
    if a < 0 && b < 0 && (a as f64) < (std::i16::MIN as f64) / (b as f64) {
      hand_mem.push_fixed(args[2], 0)?;
      hand_mem.push_fractal(args[2], HandlerMemory::str_to_fractal("underflow"))?;
      return Ok(None);
    }
    hand_mem.push_fixed(args[2], 1)?;
    hand_mem.push_fixed(args[2], (a * b) as i64)?;
    Ok(None)
  });
  cpu!(muli32 => fn(args, hand_mem) {
    let ra = hand_mem.read_fractal(args[0])?;
    let rb = hand_mem.read_fractal(args[1])?;
    if ra.read_fixed(0)? == 0 {
      hand_mem.write_fractal(args[2], &ra)?;
      return Ok(None);
    }
    if rb.read_fixed(0)? == 0 {
      hand_mem.write_fractal(args[2], &rb)?;
      return Ok(None);
    }
    let a = ra.read_fixed(1)? as i32;
    let b = rb.read_fixed(1)? as i32;
    hand_mem.init_fractal(args[2])?;
    if a > 0 && b > 0 && (a as f64) > (std::i32::MAX as f64) / (b as f64) {
      hand_mem.push_fixed(args[2], 0)?;
      hand_mem.push_fractal(args[2], HandlerMemory::str_to_fractal("overflow"))?;
      return Ok(None);
    }
    if a < 0 && b < 0 && (a as f64) < (std::i32::MIN as f64) / (b as f64) {
      hand_mem.push_fixed(args[2], 0)?;
      hand_mem.push_fractal(args[2], HandlerMemory::str_to_fractal("underflow"))?;
      return Ok(None);
    }
    hand_mem.push_fixed(args[2], 1)?;
    hand_mem.push_fixed(args[2], (a * b) as i64)?;
    Ok(None)
  });
  cpu!(muli64 => fn(args, hand_mem) {
    let ra = hand_mem.read_fractal(args[0])?;
    let rb = hand_mem.read_fractal(args[1])?;
    if ra.read_fixed(0)? == 0 {
      hand_mem.write_fractal(args[2], &ra)?;
      return Ok(None);
    }
    if rb.read_fixed(0)? == 0 {
      hand_mem.write_fractal(args[2], &rb)?;
      return Ok(None);
    }
    let a = ra.read_fixed(1)? as i64;
    let b = rb.read_fixed(1)? as i64;
    hand_mem.init_fractal(args[2])?;
    if a > 0 && b > 0 && (a as f64) > (std::i64::MAX as f64) / (b as f64) {
      hand_mem.push_fixed(args[2], 0)?;
      hand_mem.push_fractal(args[2], HandlerMemory::str_to_fractal("overflow"))?;
      return Ok(None);
    }
    if a < 0 && b < 0 && (a as f64) < (std::i64::MIN as f64) / (b as f64) {
      hand_mem.push_fixed(args[2], 0)?;
      hand_mem.push_fractal(args[2], HandlerMemory::str_to_fractal("underflow"))?;
      return Ok(None);
    }
    hand_mem.push_fixed(args[2], 1)?;
    hand_mem.push_fixed(args[2], a * b)?;
    Ok(None)
  });
  cpu!(mulf32 => fn(args, hand_mem) {
    let ra = hand_mem.read_fractal(args[0])?;
    let rb = hand_mem.read_fractal(args[1])?;
    if ra.read_fixed(0)? == 0 {
      hand_mem.write_fractal(args[2], &ra)?;
      return Ok(None);
    }
    if rb.read_fixed(0)? == 0 {
      hand_mem.write_fractal(args[2], &rb)?;
      return Ok(None);
    }
    let a = f32::from_ne_bytes((ra.read_fixed(1)? as i32).to_ne_bytes());
    let b = f32::from_ne_bytes((rb.read_fixed(1)? as i32).to_ne_bytes());
    let out = a * b;
    hand_mem.init_fractal(args[2])?;
    if out == std::f32::INFINITY {
      hand_mem.push_fixed(args[2], 0)?;
      hand_mem.push_fractal(args[2], HandlerMemory::str_to_fractal("overflow"))?;
      return Ok(None);
    }
    if out == std::f32::NEG_INFINITY {
      hand_mem.push_fixed(args[2], 0)?;
      hand_mem.push_fractal(args[2], HandlerMemory::str_to_fractal("underflow"))?;
      return Ok(None);
    }
    let num = i32::from_ne_bytes(out.to_ne_bytes()) as i64;
    hand_mem.push_fixed(args[2], 1)?;
    hand_mem.push_fixed(args[2], num)?;
    Ok(None)
  });
  cpu!(mulf64 => fn(args, hand_mem) {
    let ra = hand_mem.read_fractal(args[0])?;
    let rb = hand_mem.read_fractal(args[1])?;
    if ra.read_fixed(0)? == 0 {
      hand_mem.write_fractal(args[2], &ra)?;
      return Ok(None);
    }
    if rb.read_fixed(0)? == 0 {
      hand_mem.write_fractal(args[2], &rb)?;
      return Ok(None);
    }
    let a = f64::from_ne_bytes(ra.read_fixed(1)?.to_ne_bytes());
    let b = f64::from_ne_bytes(rb.read_fixed(1)?.to_ne_bytes());
    let out = a * b;
    hand_mem.init_fractal(args[2])?;
    if out == std::f64::INFINITY {
      hand_mem.push_fixed(args[2], 0)?;
      hand_mem.push_fractal(args[2], HandlerMemory::str_to_fractal("overflow"))?;
      return Ok(None);
    }
    if out == std::f64::NEG_INFINITY {
      hand_mem.push_fixed(args[2], 0)?;
      hand_mem.push_fractal(args[2], HandlerMemory::str_to_fractal("underflow"))?;
      return Ok(None);
    }
    let num = i64::from_ne_bytes(out.to_ne_bytes());
    hand_mem.push_fixed(args[2], 1)?;
    hand_mem.push_fixed(args[2], num)?;
    Ok(None)
  });

  cpu!(divi8 => fn(args, hand_mem) {
    let ra = hand_mem.read_fractal(args[0])?;
    let rb = hand_mem.read_fractal(args[1])?;
    if ra.read_fixed(0)? == 0 {
      hand_mem.write_fractal(args[2], &ra)?;
      return Ok(None);
    }
    if rb.read_fixed(0)? == 0 {
      hand_mem.write_fractal(args[2], &rb)?;
      return Ok(None);
    }
    let a = ra.read_fixed(1)? as i8;
    let b = rb.read_fixed(1)? as i8;
    hand_mem.init_fractal(args[2])?;
    if b == 0 {
      hand_mem.push_fixed(args[2], 0)?;
      hand_mem.push_fractal(args[2], HandlerMemory::str_to_fractal("divide-by-zero"))?;
      return Ok(None);
    }
    hand_mem.push_fixed(args[2], 1)?;
    hand_mem.push_fixed(args[2], (a / b) as i64)?;
    Ok(None)
  });
  cpu!(divi16 => fn(args, hand_mem) {
    let ra = hand_mem.read_fractal(args[0])?;
    let rb = hand_mem.read_fractal(args[1])?;
    if ra.read_fixed(0)? == 0 {
      hand_mem.write_fractal(args[2], &ra)?;
      return Ok(None);
    }
    if rb.read_fixed(0)? == 0 {
      hand_mem.write_fractal(args[2], &rb)?;
      return Ok(None);
    }
    let a = ra.read_fixed(1)? as i16;
    let b = rb.read_fixed(1)? as i16;
    hand_mem.init_fractal(args[2])?;
    if b == 0 {
      hand_mem.push_fixed(args[2], 0)?;
      hand_mem.push_fractal(args[2], HandlerMemory::str_to_fractal("divide-by-zero"))?;
      return Ok(None);
    }
    hand_mem.push_fixed(args[2], 1)?;
    hand_mem.push_fixed(args[2], (a / b) as i64)?;
    Ok(None)
  });
  cpu!(divi32 => fn(args, hand_mem) {
    let ra = hand_mem.read_fractal(args[0])?;
    let rb = hand_mem.read_fractal(args[1])?;
    if ra.read_fixed(0)? == 0 {
      hand_mem.write_fractal(args[2], &ra)?;
      return Ok(None);
    }
    if rb.read_fixed(0)? == 0 {
      hand_mem.write_fractal(args[2], &rb)?;
      return Ok(None);
    }
    let a = ra.read_fixed(1)? as i32;
    let b = rb.read_fixed(1)? as i32;
    hand_mem.init_fractal(args[2])?;
    if b == 0 {
      hand_mem.push_fixed(args[2], 0)?;
      hand_mem.push_fractal(args[2], HandlerMemory::str_to_fractal("divide-by-zero"))?;
      return Ok(None);
    }
    hand_mem.push_fixed(args[2], 1)?;
    hand_mem.push_fixed(args[2], (a / b) as i64)?;
    Ok(None)
  });
  cpu!(divi64 => fn(args, hand_mem) {
    let ra = hand_mem.read_fractal(args[0])?;
    let rb = hand_mem.read_fractal(args[1])?;
    if ra.read_fixed(0)? == 0 {
      hand_mem.write_fractal(args[2], &ra)?;
      return Ok(None);
    }
    if rb.read_fixed(0)? == 0 {
      hand_mem.write_fractal(args[2], &rb)?;
      return Ok(None);
    }
    let a = ra.read_fixed(1)? as i64;
    let b = rb.read_fixed(1)? as i64;
    hand_mem.init_fractal(args[2])?;
    if b == 0 {
      hand_mem.push_fixed(args[2], 0)?;
      hand_mem.push_fractal(args[2], HandlerMemory::str_to_fractal("divide-by-zero"))?;
      return Ok(None);
    }
    hand_mem.push_fixed(args[2], 1)?;
    hand_mem.push_fixed(args[2], a / b)?;
    Ok(None)
  });
  cpu!(divf32 => fn(args, hand_mem) {
    let ra = hand_mem.read_fractal(args[0])?;
    let rb = hand_mem.read_fractal(args[1])?;
    if ra.read_fixed(0)? == 0 {
      hand_mem.write_fractal(args[2], &ra)?;
      return Ok(None);
    }
    if rb.read_fixed(0)? == 0 {
      hand_mem.write_fractal(args[2], &rb)?;
      return Ok(None);
    }
    let a = f32::from_ne_bytes((ra.read_fixed(1)? as i32).to_ne_bytes());
    let b = f32::from_ne_bytes((rb.read_fixed(1)? as i32).to_ne_bytes());
    hand_mem.init_fractal(args[2])?;
    if b == 0.0 {
      hand_mem.push_fixed(args[2], 0)?;
      hand_mem.push_fractal(args[2], HandlerMemory::str_to_fractal("divide-by-zero"))?;
      return Ok(None);
    }
    let out = a / b;
    if out == std::f32::INFINITY {
      hand_mem.push_fixed(args[2], 0)?;
      hand_mem.push_fractal(args[2], HandlerMemory::str_to_fractal("overflow"))?;
      return Ok(None);
    }
    if out == std::f32::NEG_INFINITY {
      hand_mem.push_fixed(args[2], 0)?;
      hand_mem.push_fractal(args[2], HandlerMemory::str_to_fractal("underflow"))?;
      return Ok(None);
    }
    let num = i32::from_ne_bytes(out.to_ne_bytes()) as i64;
    hand_mem.push_fixed(args[2], 1)?;
    hand_mem.push_fixed(args[2], num)?;
    Ok(None)
  });
  cpu!(divf64 => fn(args, hand_mem) {
    let ra = hand_mem.read_fractal(args[0])?;
    let rb = hand_mem.read_fractal(args[1])?;
    if ra.read_fixed(0)? == 0 {
      hand_mem.write_fractal(args[2], &ra)?;
      return Ok(None);
    }
    if rb.read_fixed(0)? == 0 {
      hand_mem.write_fractal(args[2], &rb)?;
      return Ok(None);
    }
    let a = f64::from_ne_bytes(ra.read_fixed(1)?.to_ne_bytes());
    let b = f64::from_ne_bytes(rb.read_fixed(1)?.to_ne_bytes());
    hand_mem.init_fractal(args[2])?;
    if b == 0.0 {
      hand_mem.push_fixed(args[2], 0)?;
      hand_mem.push_fractal(args[2], HandlerMemory::str_to_fractal("divide-by-zero"))?;
      return Ok(None);
    }
    let out = a / b;
    if out == std::f64::INFINITY {
      hand_mem.push_fixed(args[2], 0)?;
      hand_mem.push_fractal(args[2], HandlerMemory::str_to_fractal("overflow"))?;
      return Ok(None);
    }
    if out == std::f64::NEG_INFINITY {
      hand_mem.push_fixed(args[2], 0)?;
      hand_mem.push_fractal(args[2], HandlerMemory::str_to_fractal("underflow"))?;
      return Ok(None);
    }
    let num = i64::from_ne_bytes(out.to_ne_bytes());
    hand_mem.push_fixed(args[2], 1)?;
    hand_mem.push_fixed(args[2], num)?;
    Ok(None)
  });

  cpu!(modi8 => fn(args, hand_mem) {
    let a = hand_mem.read_fixed(args[0])? as i8;
    let b = hand_mem.read_fixed(args[1])? as i8;
    let out = (a % b) as i64;
    hand_mem.write_fixed(args[2], out)?;
    Ok(None)
  });
  cpu!(modi16 => fn(args, hand_mem) {
    let a = hand_mem.read_fixed(args[0])? as i16;
    let b = hand_mem.read_fixed(args[1])? as i16;
    let out = (a % b) as i64;
    hand_mem.write_fixed(args[2], out)?;
    Ok(None)
  });
  cpu!(modi32 => fn(args, hand_mem) {
    let a = hand_mem.read_fixed(args[0])? as i32;
    let b = hand_mem.read_fixed(args[1])? as i32;
    let out = (a % b) as i64;
    hand_mem.write_fixed(args[2], out)?;
    Ok(None)
  });
  cpu!(modi64 => fn(args, hand_mem) {
    let a = hand_mem.read_fixed(args[0])?;
    let b = hand_mem.read_fixed(args[1])?;
    let out = a % b;
    hand_mem.write_fixed(args[2], out)?;
    Ok(None)
  });

  cpu!(powi8 => fn(args, hand_mem) {
    let ra = hand_mem.read_fractal(args[0])?;
    let rb = hand_mem.read_fractal(args[1])?;
    if ra.read_fixed(0)? == 0 {
      hand_mem.write_fractal(args[2], &ra)?;
      return Ok(None);
    }
    if rb.read_fixed(0)? == 0 {
      hand_mem.write_fractal(args[2], &rb)?;
      return Ok(None);
    }
    let a = ra.read_fixed(1)? as i8;
    let b = rb.read_fixed(1)? as i8;
    hand_mem.init_fractal(args[2])?;
    if a > 0 && b > 1 && (a as f64) > f64::powf(std::i8::MAX as f64, 1.0 / (b as f64)) {
      hand_mem.push_fixed(args[2], 0)?;
      hand_mem.push_fractal(args[2], HandlerMemory::str_to_fractal("overflow"))?;
      return Ok(None);
    }
    if a < 0 && b > 1 && (a as f64) < f64::powf(std::i8::MIN as f64, 1.0 / (b as f64)) {
      hand_mem.push_fixed(args[2], 0)?;
      hand_mem.push_fractal(args[2], HandlerMemory::str_to_fractal("underflow"))?;
      return Ok(None);
    }
    hand_mem.push_fixed(args[2], 1)?;
    let out = if b < 0 { 0i64 } else { i8::pow(a, b as u32) as i64 };
    hand_mem.push_fixed(args[2], out)?;
    Ok(None)
  });
  cpu!(powi16 => fn(args, hand_mem) {
    let ra = hand_mem.read_fractal(args[0])?;
    let rb = hand_mem.read_fractal(args[1])?;
    if ra.read_fixed(0)? == 0 {
      hand_mem.write_fractal(args[2], &ra)?;
      return Ok(None);
    }
    if rb.read_fixed(0)? == 0 {
      hand_mem.write_fractal(args[2], &rb)?;
      return Ok(None);
    }
    let a = ra.read_fixed(1)? as i16;
    let b = rb.read_fixed(1)? as i16;
    hand_mem.init_fractal(args[2])?;
    if a > 0 && b > 1 && (a as f64) > f64::powf(std::i16::MAX as f64, 1.0 / (b as f64)) {
      hand_mem.push_fixed(args[2], 0)?;
      hand_mem.push_fractal(args[2], HandlerMemory::str_to_fractal("overflow"))?;
      return Ok(None);
    }
    if a < 0 && b > 1 && (a as f64) < f64::powf(std::i16::MIN as f64, 1.0 / (b as f64)) {
      hand_mem.push_fixed(args[2], 0)?;
      hand_mem.push_fractal(args[2], HandlerMemory::str_to_fractal("underflow"))?;
      return Ok(None);
    }
    hand_mem.push_fixed(args[2], 1)?;
    let out = if b < 0 { 0i64 } else { i16::pow(a, b as u32) as i64 };
    hand_mem.push_fixed(args[2], out)?;
    Ok(None)
  });
  cpu!(powi32 => fn(args, hand_mem) {
    let ra = hand_mem.read_fractal(args[0])?;
    let rb = hand_mem.read_fractal(args[1])?;
    if ra.read_fixed(0)? == 0 {
      hand_mem.write_fractal(args[2], &ra)?;
      return Ok(None);
    }
    if rb.read_fixed(0)? == 0 {
      hand_mem.write_fractal(args[2], &rb)?;
      return Ok(None);
    }
    let a = ra.read_fixed(1)? as i32;
    let b = rb.read_fixed(1)? as i32;
    hand_mem.init_fractal(args[2])?;
    if a > 0 && b > 1 && (a as f64) > f64::powf(std::i32::MAX as f64, 1.0 / (b as f64)) {
      hand_mem.push_fixed(args[2], 0)?;
      hand_mem.push_fractal(args[2], HandlerMemory::str_to_fractal("overflow"))?;
      return Ok(None);
    }
    if a < 0 && b > 1 && (a as f64) < f64::powf(std::i32::MIN as f64, 1.0 / (b as f64)) {
      hand_mem.push_fixed(args[2], 0)?;
      hand_mem.push_fractal(args[2], HandlerMemory::str_to_fractal("underflow"))?;
      return Ok(None);
    }
    hand_mem.push_fixed(args[2], 1)?;
    let out = if b < 0 { 0i64 } else { i32::pow(a, b as u32) as i64 };
    hand_mem.push_fixed(args[2], out)?;
    Ok(None)
  });
  cpu!(powi64 => fn(args, hand_mem) {
    let ra = hand_mem.read_fractal(args[0])?;
    let rb = hand_mem.read_fractal(args[1])?;
    if ra.read_fixed(0)? == 0 {
      hand_mem.write_fractal(args[2], &ra)?;
      return Ok(None);
    }
    if rb.read_fixed(0)? == 0 {
      hand_mem.write_fractal(args[2], &rb)?;
      return Ok(None);
    }
    let a = ra.read_fixed(1)? as i64;
    let b = rb.read_fixed(1)? as i64;
    hand_mem.init_fractal(args[2])?;
    if a > 0 && b > 1 && (a as f64) > f64::powf(std::i64::MAX as f64, 1.0 / (b as f64)) {
      hand_mem.push_fixed(args[2], 0)?;
      hand_mem.push_fractal(args[2], HandlerMemory::str_to_fractal("overflow"))?;
      return Ok(None);
    }
    if a < 0 && b > 1 && (a as f64) < f64::powf(std::i64::MIN as f64, 1.0 / (b as f64)) {
      hand_mem.push_fixed(args[2], 0)?;
      hand_mem.push_fractal(args[2], HandlerMemory::str_to_fractal("underflow"))?;
      return Ok(None);
    }
    hand_mem.push_fixed(args[2], 1)?;
    let out = if b < 0 { 0i64 } else { i64::pow(a, b as u32) as i64 };
    hand_mem.push_fixed(args[2], out)?;
    Ok(None)
  });
  cpu!(powf32 => fn(args, hand_mem) {
    let ra = hand_mem.read_fractal(args[0])?;
    let rb = hand_mem.read_fractal(args[1])?;
    if ra.read_fixed(0)? == 0 {
      hand_mem.write_fractal(args[2], &ra)?;
      return Ok(None);
    }
    if rb.read_fixed(0)? == 0 {
      hand_mem.write_fractal(args[2], &rb)?;
      return Ok(None);
    }
    let a = f32::from_ne_bytes((ra.read_fixed(1)? as i32).to_ne_bytes());
    let b = f32::from_ne_bytes((rb.read_fixed(1)? as i32).to_ne_bytes());
    let out = f32::powf(a, b);
    hand_mem.init_fractal(args[2])?;
    if out == std::f32::INFINITY {
      hand_mem.push_fixed(args[2], 0)?;
      hand_mem.push_fractal(args[2], HandlerMemory::str_to_fractal("overflow"))?;
      return Ok(None);
    }
    if out == std::f32::NEG_INFINITY {
      hand_mem.push_fixed(args[2], 0)?;
      hand_mem.push_fractal(args[2], HandlerMemory::str_to_fractal("underflow"))?;
      return Ok(None);
    }
    let num = i32::from_ne_bytes(out.to_ne_bytes()) as i64;
    hand_mem.push_fixed(args[2], 1)?;
    hand_mem.push_fixed(args[2], num)?;
    Ok(None)
  });
  cpu!(powf64 => fn(args, hand_mem) {
    let ra = hand_mem.read_fractal(args[0])?;
    let rb = hand_mem.read_fractal(args[1])?;
    if ra.read_fixed(0)? == 0 {
      hand_mem.write_fractal(args[2], &ra)?;
      return Ok(None);
    }
    if rb.read_fixed(0)? == 0 {
      hand_mem.write_fractal(args[2], &rb)?;
      return Ok(None);
    }
    let a = f64::from_ne_bytes(ra.read_fixed(1)?.to_ne_bytes());
    let b = f64::from_ne_bytes(rb.read_fixed(1)?.to_ne_bytes());
    let out = f64::powf(a, b);
    hand_mem.init_fractal(args[2])?;
    if out == std::f64::INFINITY {
      hand_mem.push_fixed(args[2], 0)?;
      hand_mem.push_fractal(args[2], HandlerMemory::str_to_fractal("overflow"))?;
      return Ok(None);
    }
    if out == std::f64::NEG_INFINITY {
      hand_mem.push_fixed(args[2], 0)?;
      hand_mem.push_fractal(args[2], HandlerMemory::str_to_fractal("underflow"))?;
      return Ok(None);
    }
    let num = i64::from_ne_bytes(out.to_ne_bytes());
    hand_mem.push_fixed(args[2], 1)?;
    hand_mem.push_fixed(args[2], num)?;
    Ok(None)
  });

  cpu!(sqrtf32 => fn(args, hand_mem) {
    let a = f32::from_ne_bytes((hand_mem.read_fixed(args[0])? as i32).to_ne_bytes());
    let out = i32::from_ne_bytes(f32::sqrt(a).to_ne_bytes()) as i64;
    hand_mem.write_fixed(args[2], out)?;
    Ok(None)
  });
  cpu!(sqrtf64 => fn(args, hand_mem) {
    let a = f64::from_ne_bytes(hand_mem.read_fixed(args[0])?.to_ne_bytes());
    let out = i64::from_ne_bytes(f64::sqrt(a).to_ne_bytes());
    hand_mem.write_fixed(args[2], out)?;
    Ok(None)
  });

  // Saturating Arithmetic opcodes
  cpu!(saddi8 => fn(args, hand_mem) {
    let a = hand_mem.read_fixed(args[0])? as i8;
    let b = hand_mem.read_fixed(args[1])? as i8;
    hand_mem.write_fixed(args[2], a.saturating_add(b) as i64)?;
    Ok(None)
  });
  cpu!(saddi16 => fn(args, hand_mem) {
    let a = hand_mem.read_fixed(args[0])? as i16;
    let b = hand_mem.read_fixed(args[1])? as i16;
    hand_mem.write_fixed(args[2], a.saturating_add(b) as i64)?;
    Ok(None)
  });
  cpu!(saddi32 => fn(args, hand_mem) {
    let a = hand_mem.read_fixed(args[0])? as i32;
    let b = hand_mem.read_fixed(args[1])? as i32;
    hand_mem.write_fixed(args[2], a.saturating_add(b) as i64)?;
    Ok(None)
  });
  cpu!(saddi64 => fn(args, hand_mem) {
    let a = hand_mem.read_fixed(args[0])?;
    let b = hand_mem.read_fixed(args[1])?;
    hand_mem.write_fixed(args[2], a.saturating_add(b))?;
    Ok(None)
  });
  cpu!(saddf32 => fn(args, hand_mem) {
    let a = f32::from_ne_bytes((hand_mem.read_fixed(args[0])? as i32).to_ne_bytes());
    let b = f32::from_ne_bytes((hand_mem.read_fixed(args[1])? as i32).to_ne_bytes());
    let out = a + b;
    let num = i32::from_ne_bytes(out.to_ne_bytes()) as i64;
    hand_mem.write_fixed(args[2], num)?;
    Ok(None)
  });
  cpu!(saddf64 => fn(args, hand_mem) {
    let a = f64::from_ne_bytes(hand_mem.read_fixed(args[0])?.to_ne_bytes());
    let b = f64::from_ne_bytes(hand_mem.read_fixed(args[1])?.to_ne_bytes());
    let out = a + b;
    let num = i64::from_ne_bytes(out.to_ne_bytes());
    hand_mem.write_fixed(args[2], num)?;
    Ok(None)
  });

  cpu!(ssubi8 => fn(args, hand_mem) {
    let a = hand_mem.read_fixed(args[0])? as i8;
    let b = hand_mem.read_fixed(args[1])? as i8;
    hand_mem.write_fixed(args[2], a.saturating_sub(b) as i64)?;
    Ok(None)
  });
  cpu!(ssubi16 => fn(args, hand_mem) {
    let a = hand_mem.read_fixed(args[0])? as i16;
    let b = hand_mem.read_fixed(args[1])? as i16;
    hand_mem.write_fixed(args[2], a.saturating_sub(b) as i64)?;
    Ok(None)
  });
  cpu!(ssubi32 => fn(args, hand_mem) {
    let a = hand_mem.read_fixed(args[0])? as i32;
    let b = hand_mem.read_fixed(args[1])? as i32;
    hand_mem.write_fixed(args[2], a.saturating_sub(b) as i64)?;
    Ok(None)
  });
  cpu!(ssubi64 => fn(args, hand_mem) {
    let a = hand_mem.read_fixed(args[0])?;
    let b = hand_mem.read_fixed(args[1])?;
    hand_mem.write_fixed(args[2], a.saturating_sub(b))?;
    Ok(None)
  });
  cpu!(ssubf32 => fn(args, hand_mem) {
    let a = f32::from_ne_bytes((hand_mem.read_fixed(args[0])? as i32).to_ne_bytes());
    let b = f32::from_ne_bytes((hand_mem.read_fixed(args[1])? as i32).to_ne_bytes());
    let out = a - b;
    let num = i32::from_ne_bytes(out.to_ne_bytes()) as i64;
    hand_mem.write_fixed(args[2], num)?;
    Ok(None)
  });
  cpu!(ssubf64 => fn(args, hand_mem) {
    let a = f64::from_ne_bytes(hand_mem.read_fixed(args[0])?.to_ne_bytes());
    let b = f64::from_ne_bytes(hand_mem.read_fixed(args[1])?.to_ne_bytes());
    let out = a - b;
    let num = i64::from_ne_bytes(out.to_ne_bytes());
    hand_mem.write_fixed(args[2], num)?;
    Ok(None)
  });

  cpu!(snegi8 => fn(args, hand_mem) {
    let a = hand_mem.read_fixed(args[0])? as i8;
    let out = a.saturating_neg() as i64;
    hand_mem.write_fixed(args[2], out)?;
    Ok(None)
  });
  cpu!(snegi16 => fn(args, hand_mem) {
    let a = hand_mem.read_fixed(args[0])? as i16;
    let out = a.saturating_neg() as i64;
    hand_mem.write_fixed(args[2], out)?;
    Ok(None)
  });
  cpu!(snegi32 => fn(args, hand_mem) {
    let a = hand_mem.read_fixed(args[0])? as i32;
    let out = a.saturating_neg() as i64;
    hand_mem.write_fixed(args[2], out)?;
    Ok(None)
  });
  cpu!(snegi64 => fn(args, hand_mem) {
    let a = hand_mem.read_fixed(args[0])?;
    let out = a.saturating_neg();
    hand_mem.write_fixed(args[2], out)?;
    Ok(None)
  });
  cpu!(snegf32 => fn(args, hand_mem) {
    let a = f32::from_ne_bytes((hand_mem.read_fixed(args[0])? as i32).to_ne_bytes());
    let out = i32::from_ne_bytes((0.0 - a).to_ne_bytes()) as i64;
    hand_mem.write_fixed(args[2], out)?;
    Ok(None)
  });
  cpu!(snegf64 => fn(args, hand_mem) {
    let a = f64::from_ne_bytes(hand_mem.read_fixed(args[0])?.to_ne_bytes());
    let out = i64::from_ne_bytes((0.0 - a).to_ne_bytes());
    hand_mem.write_fixed(args[2], out)?;
    Ok(None)
  });

  cpu!(sabsi8 => fn(args, hand_mem) {
    let a = hand_mem.read_fixed(args[0])? as i8;
    let out = a.saturating_abs() as i64;
    hand_mem.write_fixed(args[2], out)?;
    Ok(None)
  });
  cpu!(sabsi16 => fn(args, hand_mem) {
    let a = hand_mem.read_fixed(args[0])? as i16;
    let out = a.saturating_abs() as i64;
    hand_mem.write_fixed(args[2], out)?;
    Ok(None)
  });
  cpu!(sabsi32 => fn(args, hand_mem) {
    let a = hand_mem.read_fixed(args[0])? as i32;
    let out = a.saturating_abs() as i64;
    hand_mem.write_fixed(args[2], out)?;
    Ok(None)
  });
  cpu!(sabsi64 => fn(args, hand_mem) {
    let a = hand_mem.read_fixed(args[0])?;
    let out = a.saturating_abs() as i64;
    hand_mem.write_fixed(args[2], out)?;
    Ok(None)
  });
  cpu!(sabsf32 => fn(args, hand_mem) {
    let a = f32::from_ne_bytes((hand_mem.read_fixed(args[0])? as i32).to_ne_bytes());
    let out = i32::from_ne_bytes(a.abs().to_ne_bytes()) as i64;
    hand_mem.write_fixed(args[2], out)?;
    Ok(None)
  });
  cpu!(sabsf64 => fn(args, hand_mem) {
    let a = f64::from_ne_bytes(hand_mem.read_fixed(args[0])?.to_ne_bytes());
    let out = i64::from_ne_bytes(a.abs().to_ne_bytes());
    hand_mem.write_fixed(args[2], out)?;
    Ok(None)
  });

  cpu!(smuli8 => fn(args, hand_mem) {
    let a = hand_mem.read_fixed(args[0])? as i8;
    let b = hand_mem.read_fixed(args[1])? as i8;
    hand_mem.write_fixed(args[2], a.saturating_mul(b) as i64)?;
    Ok(None)
  });
  cpu!(smuli16 => fn(args, hand_mem) {
    let a = hand_mem.read_fixed(args[0])? as i16;
    let b = hand_mem.read_fixed(args[1])? as i16;
    hand_mem.write_fixed(args[2], a.saturating_mul(b) as i64)?;
    Ok(None)
  });
  cpu!(smuli32 => fn(args, hand_mem) {
    let a = hand_mem.read_fixed(args[0])? as i32;
    let b = hand_mem.read_fixed(args[1])? as i32;
    hand_mem.write_fixed(args[2], a.saturating_mul(b) as i64)?;
    Ok(None)
  });
  cpu!(smuli64 => fn(args, hand_mem) {
    let a = hand_mem.read_fixed(args[0])?;
    let b = hand_mem.read_fixed(args[1])?;
    hand_mem.write_fixed(args[2], a.saturating_mul(b))?;
    Ok(None)
  });
  cpu!(smulf32 => fn(args, hand_mem) {
    let a = f32::from_ne_bytes((hand_mem.read_fixed(args[0])? as i32).to_ne_bytes());
    let b = f32::from_ne_bytes((hand_mem.read_fixed(args[1])? as i32).to_ne_bytes());
    let out = a * b;
    let num = i32::from_ne_bytes(out.to_ne_bytes()) as i64;
    hand_mem.write_fixed(args[2], num)?;
    Ok(None)
  });
  cpu!(smulf64 => fn(args, hand_mem) {
    let a = f64::from_ne_bytes(hand_mem.read_fixed(args[0])?.to_ne_bytes());
    let b = f64::from_ne_bytes(hand_mem.read_fixed(args[1])?.to_ne_bytes());
    let out = a * b;
    let num = i64::from_ne_bytes(out.to_ne_bytes());
    hand_mem.write_fixed(args[2], num)?;
    Ok(None)
  });

  cpu!(sdivi8 => fn(args, hand_mem) {
    let a = hand_mem.read_fixed(args[0])? as i8;
    let b = hand_mem.read_fixed(args[1])? as i8;
    if b == 0 {
      let out = if a > 0 { std::i8::MAX as i64 } else { std::i8::MIN as i64 };
      hand_mem.write_fixed(args[2], out)?;
      return Ok(None);
    }
    hand_mem.write_fixed(args[2], (a / b) as i64)?;
    Ok(None)
  });
  cpu!(sdivi16 => fn(args, hand_mem) {
    let a = hand_mem.read_fixed(args[0])? as i16;
    let b = hand_mem.read_fixed(args[1])? as i16;
    if b == 0 {
      let out = if a > 0 { std::i16::MAX as i64 } else { std::i16::MIN as i64 };
      hand_mem.write_fixed(args[2], out)?;
      return Ok(None);
    }
    hand_mem.write_fixed(args[2], (a / b) as i64)?;
    Ok(None)
  });
  cpu!(sdivi32 => fn(args, hand_mem) {
    let a = hand_mem.read_fixed(args[0])? as i32;
    let b = hand_mem.read_fixed(args[1])? as i32;
    if b == 0 {
      let out = if a > 0 { std::i32::MAX as i64 } else { std::i32::MIN as i64 };
      hand_mem.write_fixed(args[2], out)?;
      return Ok(None);
    }
    hand_mem.write_fixed(args[2], (a / b) as i64)?;
    Ok(None)
  });
  cpu!(sdivi64 => fn(args, hand_mem) {
    let a = hand_mem.read_fixed(args[0])?;
    let b = hand_mem.read_fixed(args[1])?;
    if b == 0 {
      let out = if a > 0 { std::i64::MAX } else { std::i64::MIN };
      hand_mem.write_fixed(args[2], out)?;
      return Ok(None);
    }
    hand_mem.write_fixed(args[2], a / b)?;
    Ok(None)
  });
  cpu!(sdivf32 => fn(args, hand_mem) {
    let a = f32::from_ne_bytes((hand_mem.read_fixed(args[0])? as i32).to_ne_bytes());
    let b = f32::from_ne_bytes((hand_mem.read_fixed(args[1])? as i32).to_ne_bytes());
    let out = a / b;
    let num = i32::from_ne_bytes(out.to_ne_bytes()) as i64;
    hand_mem.write_fixed(args[2], num)?;
    Ok(None)
  });
  cpu!(sdivf64 => fn(args, hand_mem) {
    let a = f64::from_ne_bytes(hand_mem.read_fixed(args[0])?.to_ne_bytes());
    let b = f64::from_ne_bytes(hand_mem.read_fixed(args[1])?.to_ne_bytes());
    let out = a / b;
    let num = i64::from_ne_bytes(out.to_ne_bytes());
    hand_mem.write_fixed(args[2], num)?;
    Ok(None)
  });

  cpu!(spowi8 => fn(args, hand_mem) {
    let a = hand_mem.read_fixed(args[0])? as i8;
    let b = hand_mem.read_fixed(args[1])? as i8;
    let out = if b < 0 { 0i64 } else { i8::saturating_pow(a, b as u32) as i64 };
    hand_mem.write_fixed(args[2], out)?;
    Ok(None)
  });
  cpu!(spowi16 => fn(args, hand_mem) {
    let a = hand_mem.read_fixed(args[0])? as i16;
    let b = hand_mem.read_fixed(args[1])? as i16;
    let out = if b < 0 { 0i64 } else { i16::saturating_pow(a, b as u32) as i64 };
    hand_mem.write_fixed(args[2], out)?;
    Ok(None)
  });
  cpu!(spowi32 => fn(args, hand_mem) {
    let a = hand_mem.read_fixed(args[0])? as i32;
    let b = hand_mem.read_fixed(args[1])? as i32;
    let out = if b < 0 { 0i64 } else { i32::saturating_pow(a, b as u32) as i64 };
    hand_mem.write_fixed(args[2], out)?;
    Ok(None)
  });
  cpu!(spowi64 => fn(args, hand_mem) {
    let a = hand_mem.read_fixed(args[0])?;
    let b = hand_mem.read_fixed(args[1])?;
    let out = if b < 0 { 0i64 } else { i64::saturating_pow(a, b as u32) as i64 };
    hand_mem.write_fixed(args[2], out)?;
    Ok(None)
  });
  cpu!(spowf32 => fn(args, hand_mem) {
    let a = f32::from_ne_bytes((hand_mem.read_fixed(args[0])? as i32).to_ne_bytes());
    let b = f32::from_ne_bytes((hand_mem.read_fixed(args[1])? as i32).to_ne_bytes());
    let out = f32::powf(a, b);
    let num = i32::from_ne_bytes(out.to_ne_bytes()) as i64;
    hand_mem.write_fixed(args[2], num)?;
    Ok(None)
  });
  cpu!(spowf64 => fn(args, hand_mem) {
    let a = f64::from_ne_bytes(hand_mem.read_fixed(args[0])?.to_ne_bytes());
    let b = f64::from_ne_bytes(hand_mem.read_fixed(args[1])?.to_ne_bytes());
    let out = f64::powf(a, b);
    let num = i64::from_ne_bytes(out.to_ne_bytes());
    hand_mem.write_fixed(args[2], num)?;
    Ok(None)
  });

  // Boolean and bitwise opcodes
  cpu!(andi8 => fn(args, hand_mem) {
    let a = hand_mem.read_fixed(args[0])? as i8;
    let b = hand_mem.read_fixed(args[1])? as i8;
    let out = (a & b) as i64;
    hand_mem.write_fixed(args[2], out)?;
    Ok(None)
  });
  cpu!(andi16 => fn(args, hand_mem) {
    let a = hand_mem.read_fixed(args[0])? as i16;
    let b = hand_mem.read_fixed(args[1])? as i16;
    let out = (a & b) as i64;
    hand_mem.write_fixed(args[2], out)?;
    Ok(None)
  });
  cpu!(andi32 => fn(args, hand_mem) {
    let a = hand_mem.read_fixed(args[0])? as i32;
    let b = hand_mem.read_fixed(args[1])? as i32;
    let out = (a & b) as i64;
    hand_mem.write_fixed(args[2], out)?;
    Ok(None)
  });
  cpu!(andi64 => fn(args, hand_mem) {
    let a = hand_mem.read_fixed(args[0])?;
    let b = hand_mem.read_fixed(args[1])?;
    let out = a & b;
    hand_mem.write_fixed(args[2], out)?;
    Ok(None)
  });
  cpu!(andbool => fn(args, hand_mem) {
    let a = hand_mem.read_fixed(args[0])?;
    let b = hand_mem.read_fixed(args[1])?;
    let a_bool = if a == 1 { true } else { false };
    let b_bool = if b == 1 { true } else { false };
    let out = if a_bool & b_bool { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out)?;
    Ok(None)
  });

  cpu!(ori8 => fn(args, hand_mem) {
    let a = hand_mem.read_fixed(args[0])? as i8;
    let b = hand_mem.read_fixed(args[1])? as i8;
    let out = (a | b) as i64;
    hand_mem.write_fixed(args[2], out)?;
    Ok(None)
  });
  cpu!(ori16 => fn(args, hand_mem) {
    let a = hand_mem.read_fixed(args[0])? as i16;
    let b = hand_mem.read_fixed(args[1])? as i16;
    let out = (a | b) as i64;
    hand_mem.write_fixed(args[2], out)?;
    Ok(None)
  });
  cpu!(ori32 => fn(args, hand_mem) {
    let a = hand_mem.read_fixed(args[0])? as i32;
    let b = hand_mem.read_fixed(args[1])? as i32;
    let out = (a | b) as i64;
    hand_mem.write_fixed(args[2], out)?;
    Ok(None)
  });
  cpu!(ori64 => fn(args, hand_mem) {
    let a = hand_mem.read_fixed(args[0])?;
    let b = hand_mem.read_fixed(args[1])?;
    let out = a | b;
    hand_mem.write_fixed(args[2], out)?;
    Ok(None)
  });
  cpu!(orbool => fn(args, hand_mem) {
    let a = hand_mem.read_fixed(args[0])?;
    let b = hand_mem.read_fixed(args[1])?;
    let a_bool = if a == 1 { true } else { false };
    let b_bool = if b == 1 { true } else { false };
    let out = if a_bool | b_bool { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out)?;
    Ok(None)
  });

  cpu!(xori8 => fn(args, hand_mem) {
    let a = hand_mem.read_fixed(args[0])? as i8;
    let b = hand_mem.read_fixed(args[1])? as i8;
    let out = (a ^ b) as i64;
    hand_mem.write_fixed(args[2], out)?;
    Ok(None)
  });
  cpu!(xori16 => fn(args, hand_mem) {
    let a = hand_mem.read_fixed(args[0])? as i16;
    let b = hand_mem.read_fixed(args[1])? as i16;
    let out = (a ^ b) as i64;
    hand_mem.write_fixed(args[2], out)?;
    Ok(None)
  });
  cpu!(xori32 => fn(args, hand_mem) {
    let a = hand_mem.read_fixed(args[0])? as i32;
    let b = hand_mem.read_fixed(args[1])? as i32;
    let out = (a ^ b) as i64;
    hand_mem.write_fixed(args[2], out)?;
    Ok(None)
  });
  cpu!(xori64 => fn(args, hand_mem) {
    let a = hand_mem.read_fixed(args[0])?;
    let b = hand_mem.read_fixed(args[1])?;
    let out = a ^ b;
    hand_mem.write_fixed(args[2], out)?;
    Ok(None)
  });
  cpu!(xorbool => fn(args, hand_mem) {
    let a = hand_mem.read_fixed(args[0])?;
    let b = hand_mem.read_fixed(args[1])?;
    let a_bool = if a == 1 { true } else { false };
    let b_bool = if b == 1 { true } else { false };
    let out = if a_bool ^ b_bool { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out)?;
    Ok(None)
  });

  cpu!(noti8 => fn(args, hand_mem) {
    let a = hand_mem.read_fixed(args[0])? as i8;
    let out = !a as i64;
    hand_mem.write_fixed(args[2], out)?;
    Ok(None)
  });
  cpu!(noti16 => fn(args, hand_mem) {
    let a = hand_mem.read_fixed(args[0])? as i16;
    let out = !a as i64;
    hand_mem.write_fixed(args[2], out)?;
    Ok(None)
  });
  cpu!(noti32 => fn(args, hand_mem) {
    let a = hand_mem.read_fixed(args[0])? as i32;
    let out = !a as i64;
    hand_mem.write_fixed(args[2], out)?;
    Ok(None)
  });
  cpu!(noti64 => fn(args, hand_mem) {
    let a = hand_mem.read_fixed(args[0])?;
    let out = !a;
    hand_mem.write_fixed(args[2], out)?;
    Ok(None)
  });
  cpu!(notbool => fn(args, hand_mem) {
    let a = hand_mem.read_fixed(args[0])?;
    let a_bool = if a == 1 { true } else { false };
    let out = if !a_bool { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out)?;
    Ok(None)
  });

  cpu!(nandi8 => fn(args, hand_mem) {
    let a = hand_mem.read_fixed(args[0])? as i8;
    let b = hand_mem.read_fixed(args[1])? as i8;
    let out = !(a & b) as i64;
    hand_mem.write_fixed(args[2], out)?;
    Ok(None)
  });
  cpu!(nandi16 => fn(args, hand_mem) {
    let a = hand_mem.read_fixed(args[0])? as i16;
    let b = hand_mem.read_fixed(args[1])? as i16;
    let out = !(a & b) as i64;
    hand_mem.write_fixed(args[2], out)?;
    Ok(None)
  });
  cpu!(nandi32 => fn(args, hand_mem) {
    let a = hand_mem.read_fixed(args[0])? as i32;
    let b = hand_mem.read_fixed(args[1])? as i32;
    let out = !(a & b) as i64;
    hand_mem.write_fixed(args[2], out)?;
    Ok(None)
  });
  cpu!(nandi64 => fn(args, hand_mem) {
    let a = hand_mem.read_fixed(args[0])?;
    let b = hand_mem.read_fixed(args[1])?;
    let out = !(a & b);
    hand_mem.write_fixed(args[2], out)?;
    Ok(None)
  });
  cpu!(nandboo => fn(args, hand_mem) {
    let a = hand_mem.read_fixed(args[0])?;
    let b = hand_mem.read_fixed(args[1])?;
    let a_bool = if a == 1 { true } else { false };
    let b_bool = if b == 1 { true } else { false };
    let out = if !(a_bool & b_bool) { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out)?;
    Ok(None)
  });

  cpu!(nori8 => fn(args, hand_mem) {
    let a = hand_mem.read_fixed(args[0])? as i8;
    let b = hand_mem.read_fixed(args[1])? as i8;
    let out = !(a | b) as i64;
    hand_mem.write_fixed(args[2], out)?;
    Ok(None)
  });
  cpu!(nori16 => fn(args, hand_mem) {
    let a = hand_mem.read_fixed(args[0])? as i16;
    let b = hand_mem.read_fixed(args[1])? as i16;
    let out = !(a | b) as i64;
    hand_mem.write_fixed(args[2], out)?;
    Ok(None)
  });
  cpu!(nori32 => fn(args, hand_mem) {
    let a = hand_mem.read_fixed(args[0])? as i32;
    let b = hand_mem.read_fixed(args[1])? as i32;
    let out = !(a | b) as i64;
    hand_mem.write_fixed(args[2], out)?;
    Ok(None)
  });
  cpu!(nori64 => fn(args, hand_mem) {
    let a = hand_mem.read_fixed(args[0])?;
    let b = hand_mem.read_fixed(args[1])?;
    let out = !(a | b);
    hand_mem.write_fixed(args[2], out)?;
    Ok(None)
  });
  cpu!(norbool => fn(args, hand_mem) {
    let a = hand_mem.read_fixed(args[0])?;
    let b = hand_mem.read_fixed(args[1])?;
    let a_bool = if a == 1 { true } else { false };
    let b_bool = if b == 1 { true } else { false };
    let out = if !(a_bool | b_bool) { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out)?;
    Ok(None)
  });

  cpu!(xnori8 => fn(args, hand_mem) {
    let a = hand_mem.read_fixed(args[0])? as i8;
    let b = hand_mem.read_fixed(args[1])? as i8;
    let out = !(a ^ b) as i64;
    hand_mem.write_fixed(args[2], out)?;
    Ok(None)
  });
  cpu!(xnori16 => fn(args, hand_mem) {
    let a = hand_mem.read_fixed(args[0])? as i16;
    let b = hand_mem.read_fixed(args[1])? as i16;
    let out = !(a ^ b) as i64;
    hand_mem.write_fixed(args[2], out)?;
    Ok(None)
  });
  cpu!(xnori32 => fn(args, hand_mem) {
    let a = hand_mem.read_fixed(args[0])? as i32;
    let b = hand_mem.read_fixed(args[1])? as i32;
    let out = !(a ^ b) as i64;
    hand_mem.write_fixed(args[2], out)?;
    Ok(None)
  });
  cpu!(xnori64 => fn(args, hand_mem) {
    let a = hand_mem.read_fixed(args[0])?;
    let b = hand_mem.read_fixed(args[1])?;
    let out = !(a ^ b);
    hand_mem.write_fixed(args[2], out)?;
    Ok(None)
  });
  cpu!(xnorboo => fn(args, hand_mem) {
    let a = hand_mem.read_fixed(args[0])?;
    let b = hand_mem.read_fixed(args[1])?;
    let a_bool = if a == 1 { true } else { false };
    let b_bool = if b == 1 { true } else { false };
    let out = if !(a_bool ^ b_bool) { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out)?;
    Ok(None)
  });

  // Equality and order opcodes
  cpu!(eqi8 => fn(args, hand_mem) {
    let a = hand_mem.read_fixed(args[0])? as i8;
    let b = hand_mem.read_fixed(args[1])? as i8;
    let out = if a == b { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out)?;
    Ok(None)
  });
  cpu!(eqi16 => fn(args, hand_mem) {
    let a = hand_mem.read_fixed(args[0])? as i16;
    let b = hand_mem.read_fixed(args[1])? as i16;
    let out = if a == b { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out)?;
    Ok(None)
  });
  cpu!(eqi32 => fn(args, hand_mem) {
    let a = hand_mem.read_fixed(args[0])? as i32;
    let b = hand_mem.read_fixed(args[1])? as i32;
    let out = if a == b { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out)?;
    Ok(None)
  });
  cpu!(eqi64 => fn(args, hand_mem) {
    let a = hand_mem.read_fixed(args[0])?;
    let b = hand_mem.read_fixed(args[1])?;
    let out = if a == b { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out)?;
    Ok(None)
  });
  cpu!(eqf32 => fn(args, hand_mem) {
    let a = f32::from_ne_bytes((hand_mem.read_fixed(args[0])? as i32).to_ne_bytes());
    let b = f32::from_ne_bytes((hand_mem.read_fixed(args[1])? as i32).to_ne_bytes());
    let out = if a == b { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out)?;
    Ok(None)
  });
  cpu!(eqf64 => fn(args, hand_mem) {
    let a = f64::from_ne_bytes(hand_mem.read_fixed(args[0])?.to_ne_bytes());
    let b = f64::from_ne_bytes(hand_mem.read_fixed(args[1])?.to_ne_bytes());
    let out = if a == b { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out)?;
    Ok(None)
  });
  cpu!(eqstr => fn(args, hand_mem) {
    let a_pascal_string = hand_mem.read_fractal(args[0])?;
    let b_pascal_string = hand_mem.read_fractal(args[1])?;
    let out = if args[0] < 0 || args[1] < 0 {
      // Special path for global memory stored strings, they aren't represented the same way
      let a_str = HandlerMemory::fractal_to_string(a_pascal_string)?;
      let b_str = HandlerMemory::fractal_to_string(b_pascal_string)?;
      if a_str == b_str { 1i64 } else { 0i64 }
    } else if a_pascal_string == b_pascal_string {
      1i64
    } else {
      0i64
    };
    hand_mem.write_fixed(args[2], out)?;
    Ok(None)
  });
  cpu!(eqbool => fn(args, hand_mem) {
    let a = hand_mem.read_fixed(args[0])? as i8;
    let b = hand_mem.read_fixed(args[1])? as i8;
    let out = if a == b { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out)?;
    Ok(None)
  });

  cpu!(neqi8 => fn(args, hand_mem) {
    let a = hand_mem.read_fixed(args[0])? as i8;
    let b = hand_mem.read_fixed(args[1])? as i8;
    let out = if a != b { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out)?;
    Ok(None)
  });
  cpu!(neqi16 => fn(args, hand_mem) {
    let a = hand_mem.read_fixed(args[0])? as i16;
    let b = hand_mem.read_fixed(args[1])? as i16;
    let out = if a != b { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out)?;
    Ok(None)
  });
  cpu!(neqi32 => fn(args, hand_mem) {
    let a = hand_mem.read_fixed(args[0])? as i32;
    let b = hand_mem.read_fixed(args[1])? as i32;
    let out = if a != b { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out)?;
    Ok(None)
  });
  cpu!(neqi64 => fn(args, hand_mem) {
    let a = hand_mem.read_fixed(args[0])?;
    let b = hand_mem.read_fixed(args[1])?;
    let out = if a != b { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out)?;
    Ok(None)
  });
  cpu!(neqf32 => fn(args, hand_mem) {
    let a = f32::from_ne_bytes((hand_mem.read_fixed(args[0])? as i32).to_ne_bytes());
    let b = f32::from_ne_bytes((hand_mem.read_fixed(args[1])? as i32).to_ne_bytes());
    let out = if a != b { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out)?;
    Ok(None)
  });
  cpu!(neqf64 => fn(args, hand_mem) {
    let a = f64::from_ne_bytes(hand_mem.read_fixed(args[0])?.to_ne_bytes());
    let b = f64::from_ne_bytes(hand_mem.read_fixed(args[1])?.to_ne_bytes());
    let out = if a != b { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out)?;
    Ok(None)
  });
  cpu!(neqstr => fn(args, hand_mem) {
    let a_pascal_string = hand_mem.read_fractal(args[0])?;
    let b_pascal_string = hand_mem.read_fractal(args[1])?;
    let out = if a_pascal_string != b_pascal_string {
      1i64
    } else {
      0i64
    };
    hand_mem.write_fixed(args[2], out)?;
    Ok(None)
  });
  cpu!(neqbool => fn(args, hand_mem) {
    let a = hand_mem.read_fixed(args[0])? as i8;
    let b = hand_mem.read_fixed(args[1])? as i8;
    let out = if a != b { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out)?;
    Ok(None)
  });

  cpu!(lti8 => fn(args, hand_mem) {
    let a = hand_mem.read_fixed(args[0])? as i8;
    let b = hand_mem.read_fixed(args[1])? as i8;
    let out = if a < b { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out)?;
    Ok(None)
  });
  cpu!(lti16 => fn(args, hand_mem) {
    let a = hand_mem.read_fixed(args[0])? as i16;
    let b = hand_mem.read_fixed(args[1])? as i16;
    let out = if a < b { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out)?;
    Ok(None)
  });
  cpu!(lti32 => fn(args, hand_mem) {
    let a = hand_mem.read_fixed(args[0])? as i32;
    let b = hand_mem.read_fixed(args[1])? as i32;
    let out = if a < b { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out)?;
    Ok(None)
  });
  cpu!(lti64 => fn(args, hand_mem) {
    let a = hand_mem.read_fixed(args[0])?;
    let b = hand_mem.read_fixed(args[1])?;
    let out = if a < b { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out)?;
    Ok(None)
  });
  cpu!(ltf32 => fn(args, hand_mem) {
    let a = f32::from_ne_bytes((hand_mem.read_fixed(args[0])? as i32).to_ne_bytes());
    let b = f32::from_ne_bytes((hand_mem.read_fixed(args[1])? as i32).to_ne_bytes());
    let out = if a < b { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out)?;
    Ok(None)
  });
  cpu!(ltf64 => fn(args, hand_mem) {
    let a = f64::from_ne_bytes(hand_mem.read_fixed(args[0])?.to_ne_bytes());
    let b = f64::from_ne_bytes(hand_mem.read_fixed(args[1])?.to_ne_bytes());
    let out = if a < b { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out)?;
    Ok(None)
  });
  cpu!(ltstr => fn(args, hand_mem) {
    let a_str = HandlerMemory::fractal_to_string(hand_mem.read_fractal(args[0])?)?;
    let b_str = HandlerMemory::fractal_to_string(hand_mem.read_fractal(args[1])?)?;
    let out = if a_str < b_str { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out)?;
    Ok(None)
  });

  cpu!(ltei8 => fn(args, hand_mem) {
    let a = hand_mem.read_fixed(args[0])? as i8;
    let b = hand_mem.read_fixed(args[1])? as i8;
    let out = if a <= b { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out)?;
    Ok(None)
  });
  cpu!(ltei16 => fn(args, hand_mem) {
    let a = hand_mem.read_fixed(args[0])? as i16;
    let b = hand_mem.read_fixed(args[1])? as i16;
    let out = if a <= b { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out)?;
    Ok(None)
  });
  cpu!(ltei32 => fn(args, hand_mem) {
    let a = hand_mem.read_fixed(args[0])? as i32;
    let b = hand_mem.read_fixed(args[1])? as i32;
    let out = if a <= b { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out)?;
    Ok(None)
  });
  cpu!(ltei64 => fn(args, hand_mem) {
    let a = hand_mem.read_fixed(args[0])?;
    let b = hand_mem.read_fixed(args[1])?;
    let out = if a <= b { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out)?;
    Ok(None)
  });
  cpu!(ltef32 => fn(args, hand_mem) {
    let a = f32::from_ne_bytes((hand_mem.read_fixed(args[0])? as i32).to_ne_bytes());
    let b = f32::from_ne_bytes((hand_mem.read_fixed(args[1])? as i32).to_ne_bytes());
    let out = if a <= b { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out)?;
    Ok(None)
  });
  cpu!(ltef64 => fn(args, hand_mem) {
    let a = f64::from_ne_bytes(hand_mem.read_fixed(args[0])?.to_ne_bytes());
    let b = f64::from_ne_bytes(hand_mem.read_fixed(args[1])?.to_ne_bytes());
    let out = if a <= b { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out)?;
    Ok(None)
  });
  cpu!(ltestr => fn(args, hand_mem) {
    let a_str = HandlerMemory::fractal_to_string(hand_mem.read_fractal(args[0])?)?;
    let b_str = HandlerMemory::fractal_to_string(hand_mem.read_fractal(args[1])?)?;
    let out = if a_str <= b_str { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out)?;
    Ok(None)
  });

  cpu!(gti8 => fn(args, hand_mem) {
    let a = hand_mem.read_fixed(args[0])? as i8;
    let b = hand_mem.read_fixed(args[1])? as i8;
    let out = if a > b { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out)?;
    Ok(None)
  });
  cpu!(gti16 => fn(args, hand_mem) {
    let a = hand_mem.read_fixed(args[0])? as i16;
    let b = hand_mem.read_fixed(args[1])? as i16;
    let out = if a > b { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out)?;
    Ok(None)
  });
  cpu!(gti32 => fn(args, hand_mem) {
    let a = hand_mem.read_fixed(args[0])? as i32;
    let b = hand_mem.read_fixed(args[1])? as i32;
    let out = if a > b { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out)?;
    Ok(None)
  });
  cpu!(gti64 => fn(args, hand_mem) {
    let a = hand_mem.read_fixed(args[0])?;
    let b = hand_mem.read_fixed(args[1])?;
    let out = if a > b { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out)?;
    Ok(None)
  });
  cpu!(gtf32 => fn(args, hand_mem) {
    let a = f32::from_ne_bytes((hand_mem.read_fixed(args[0])? as i32).to_ne_bytes());
    let b = f32::from_ne_bytes((hand_mem.read_fixed(args[1])? as i32).to_ne_bytes());
    let out = if a > b { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out)?;
    Ok(None)
  });
  cpu!(gtf64 => fn(args, hand_mem) {
    let a = f64::from_ne_bytes(hand_mem.read_fixed(args[0])?.to_ne_bytes());
    let b = f64::from_ne_bytes(hand_mem.read_fixed(args[1])?.to_ne_bytes());
    let out = if a > b { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out)?;
    Ok(None)
  });
  cpu!(gtstr => fn(args, hand_mem) {
    let a_str = HandlerMemory::fractal_to_string(hand_mem.read_fractal(args[0])?)?;
    let b_str = HandlerMemory::fractal_to_string(hand_mem.read_fractal(args[1])?)?;
    let out = if a_str > b_str { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out)?;
    Ok(None)
  });

  cpu!(gtei8 => fn(args, hand_mem) {
    let a = hand_mem.read_fixed(args[0])? as i8;
    let b = hand_mem.read_fixed(args[1])? as i8;
    let out = if a >= b { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out)?;
    Ok(None)
  });
  cpu!(gtei16 => fn(args, hand_mem) {
    let a = hand_mem.read_fixed(args[0])? as i16;
    let b = hand_mem.read_fixed(args[1])? as i16;
    let out = if a >= b { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out)?;
    Ok(None)
  });
  cpu!(gtei32 => fn(args, hand_mem) {
    let a = hand_mem.read_fixed(args[0])? as i32;
    let b = hand_mem.read_fixed(args[1])? as i32;
    let out = if a >= b { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out)?;
    Ok(None)
  });
  cpu!(gtei64 => fn(args, hand_mem) {
    let a = hand_mem.read_fixed(args[0])?;
    let b = hand_mem.read_fixed(args[1])?;
    let out = if a >= b { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out)?;
    Ok(None)
  });
  cpu!(gtef32 => fn(args, hand_mem) {
    let a = f32::from_ne_bytes((hand_mem.read_fixed(args[0])? as i32).to_ne_bytes());
    let b = f32::from_ne_bytes((hand_mem.read_fixed(args[1])? as i32).to_ne_bytes());
    let out = if a >= b { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out)?;
    Ok(None)
  });
  cpu!(gtef64 => fn(args, hand_mem) {
    let a = f64::from_ne_bytes(hand_mem.read_fixed(args[0])?.to_ne_bytes());
    let b = f64::from_ne_bytes(hand_mem.read_fixed(args[1])?.to_ne_bytes());
    let out = if a >= b { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out)?;
    Ok(None)
  });
  cpu!(gtestr => fn(args, hand_mem) {
    let a_str = HandlerMemory::fractal_to_string(hand_mem.read_fractal(args[0])?)?;
    let b_str = HandlerMemory::fractal_to_string(hand_mem.read_fractal(args[1])?)?;
    let out = if a_str >= b_str { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out)?;
    Ok(None)
  });

  // String opcodes
  cpu!(catstr => fn(args, hand_mem) {
    let a_str = HandlerMemory::fractal_to_string(hand_mem.read_fractal(args[0])?)?;
    let b_str = HandlerMemory::fractal_to_string(hand_mem.read_fractal(args[1])?)?;
    let out_str = format!("{}{}", a_str, b_str);
    hand_mem.write_fractal(args[2], &HandlerMemory::str_to_fractal(&out_str))?;
    Ok(None)
  });
  cpu!(split => fn(args, hand_mem) {
    let a_str = HandlerMemory::fractal_to_string(hand_mem.read_fractal(args[0])?)?;
    let b_str = HandlerMemory::fractal_to_string(hand_mem.read_fractal(args[1])?)?;
    let out_hms = a_str.split(&b_str).map(|out_str| HandlerMemory::str_to_fractal(&out_str));
    hand_mem.init_fractal(args[2])?;
    for out in out_hms {
      hand_mem.push_fractal(args[2], out)?;
    }
    Ok(None)
  });
  cpu!(repstr => fn(args, hand_mem) {
    let a_str = HandlerMemory::fractal_to_string(hand_mem.read_fractal(args[0])?)?;
    let n = hand_mem.read_fixed(args[1])?;
    let out_str = a_str.repeat(n as usize);
    hand_mem.write_fractal(args[2], &HandlerMemory::str_to_fractal(&out_str))?;
    Ok(None)
  });
  cpu!(matches => fn(args, hand_mem) {
    let a_str = HandlerMemory::fractal_to_string(hand_mem.read_fractal(args[0])?)?;
    let b_str = HandlerMemory::fractal_to_string(hand_mem.read_fractal(args[1])?)?;
    let b_regex = Regex::new(&b_str).map_err(|regex_err| VMError::Other(format!("Bad regex construction: {}", regex_err)))?;
    let out = if b_regex.is_match(&a_str) { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out)?;
    Ok(None)
  });
  cpu!(indstr => fn(args, hand_mem) {
    let a_str = HandlerMemory::fractal_to_string(hand_mem.read_fractal(args[0])?)?;
    let b_str = HandlerMemory::fractal_to_string(hand_mem.read_fractal(args[1])?)?;
    let out_option = a_str.find(&b_str);
    hand_mem.init_fractal(args[2])?;
    match out_option {
      Some(out) => {
        hand_mem.push_fixed(args[2], 1)?;
        hand_mem.push_fixed(args[2], out as i64)?;
      },
      None => {
        hand_mem.push_fixed(args[2], 0)?;
        hand_mem.write_fractal(args[2], &HandlerMemory::str_to_fractal("substring not found"))?;
      },
    }
    Ok(None)
  });
  cpu!(lenstr => fn(args, hand_mem) {
    let pascal_string = hand_mem.read_fractal(args[0])?;
    let val = pascal_string.read_fixed(0)?;
    hand_mem.write_fixed(args[2], val)?;
    Ok(None)
  });
  cpu!(trim => fn(args, hand_mem) {
    let in_str = HandlerMemory::fractal_to_string(hand_mem.read_fractal(args[0])?)?;
    let out_str = in_str.trim();
    hand_mem.write_fractal(args[2], &HandlerMemory::str_to_fractal(&out_str))?;
    Ok(None)
  });

  // Array opcodes
  cpu!(register => fn(args, hand_mem) {
    // args[2] is the register address
    // args[0] point to an array in memory
    // args[1] is the address within the array to register
    let inner_addr = hand_mem.read_fixed(args[1])?;
    hand_mem.register_out(args[0], inner_addr as usize, args[2])?;
    Ok(None)
  });
  cpu!(copyfrom => fn(args, hand_mem) {
    // args = [arr_addr, arr_idx_addr, outer_addr]
    // copy data from outer_addr to inner_addr of the array in reg_addr
    // The array index instead of inner address is provided to keep interaction with the js-runtime
    // sane.
    let inner_addr = hand_mem.read_fixed(args[1])?;
    hand_mem.register_out(args[0], inner_addr as usize, args[2])?;
    Ok(None)
  });
  cpu!(copytof => fn(args, hand_mem) {
    // args = [arr_addr, inner_addr, outer_addr]
    // copy data from outer_addr to inner_addr in arr_addr
    let inner = hand_mem.read_fixed(args[1])?;
    hand_mem.register_in(args[2], args[0], inner)?;
    Ok(None)
  });
  cpu!(copytov => fn(args, hand_mem) {
    // args = [arr_addr, inner_addr, outer_addr]
    // copy data from outer_addr to inner_addr in arr_addr
    let inner = hand_mem.read_fixed(args[1])?;
    hand_mem.register_in(args[2], args[0], inner)?;
    Ok(None)
  });
  cpu!(lenarr => fn(args, hand_mem) {
    let arr = hand_mem.read_fractal(args[0])?;
    let len = arr.len() as i64;
    hand_mem.write_fixed(args[2], len)?;
    Ok(None)
  });
  cpu!(indarrf => fn(args, hand_mem) {
    let val = hand_mem.read_fixed(args[1])?;
    let mem = hand_mem.read_fractal(args[0])?;
    let len = mem.len();
    let mut idx = -1i64;
    for i in 0..len {
      let check = mem.read_fixed(i)?;
      if val == check {
        idx = i as i64;
        break;
      }
    }
    hand_mem.init_fractal(args[2])?;
    if idx == -1i64 {
      hand_mem.push_fixed(args[2], 0i64)?;
      hand_mem.write_fractal(args[2], &HandlerMemory::str_to_fractal("element not found"))?;
    } else {
      hand_mem.push_fixed(args[2], 1i64)?;
      hand_mem.push_fixed(args[2], idx)?;
    }
    Ok(None)
  });
  cpu!(indarrv => fn(args, hand_mem) {
    let val = hand_mem.read_fractal(args[1])?;
    let fractal = hand_mem.read_fractal(args[0])?;
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
    hand_mem.init_fractal(args[2])?;
    if let Some(idx) = idx {
      hand_mem.push_fixed(args[2], 1i64)?;
      hand_mem.push_fixed(args[2], idx)?;
    } else {
      hand_mem.push_fixed(args[2], 0i64)?;
      hand_mem.write_fractal(args[2], &HandlerMemory::str_to_fractal("element not found"))?;
    }
    Ok(None)
  });
  cpu!(join => fn(args, hand_mem) {
    let sep_str = HandlerMemory::fractal_to_string(hand_mem.read_fractal(args[1])?)?;
    let fractal = hand_mem.read_fractal(args[0])?;
    let mut strs = Vec::with_capacity(fractal.len());
    for i in 0..fractal.len() {
      match hand_mem.read_from_fractal(&fractal, i) {
        (data, true) => {
          let v_str = HandlerMemory::fractal_to_string(data)?;
          strs.push(v_str);
        },
        (_, false) => todo!("handle joining non-fractal strings I think?"),
      }
    }
    let out_str = strs.join(&sep_str);
    hand_mem.write_fractal(args[2], &HandlerMemory::str_to_fractal(&out_str))?;
    Ok(None)
  });
  cpu!(pusharr => fn(args, hand_mem) {
    let val_size = hand_mem.read_fixed(args[2])?;
    if val_size == 0 {
      hand_mem.push_register(args[0], args[1])?;
    } else {
      let val = hand_mem.read_fixed(args[1])?;
      hand_mem.push_fixed(args[0], val)?;
    }
    Ok(None)
  });
  cpu!(poparr => fn(args, hand_mem) {
    let last = hand_mem.pop(args[0]);
    hand_mem.init_fractal(args[2])?;
    match last {
      Ok(record) => {
        hand_mem.push_fixed(args[2], 1i64)?;
        hand_mem.push_register_out(args[2], &record, 0)?;
      },
      Err(error) => {
        hand_mem.push_fixed(args[2], 0i64)?;
        hand_mem.push_fractal(args[2], HandlerMemory::str_to_fractal(&format!("{}", error)))?;
      },
    }
    Ok(None)
  });
  cpu!(delindx => fn(args, hand_mem) {
    let idx = hand_mem.read_fixed(args[1])? as usize;
    let el = hand_mem.delete(args[0], idx);
    hand_mem.init_fractal(args[2])?;
    match el {
      Ok(record) => {
        hand_mem.push_fixed(args[2], 1i64)?;
        hand_mem.push_register_out(args[2], &record, 0)?;
      },
      Err(error) => {
        hand_mem.push_fixed(args[2], 0i64)?;
        hand_mem.push_fractal(args[2], HandlerMemory::str_to_fractal(&format!("{}", error)))?;
      },
    }
    Ok(None)
  });
  cpu!(newarr => fn(args, hand_mem) {
    hand_mem.init_fractal(args[2])?;
    Ok(None)
  });
  io!(map => fn(args, mut hand_mem) {
    Box::pin(async move {
      let fractal = hand_mem.read_fractal(args[0])?;
      let subhandler = HandlerFragment::new(args[1], 0);
      let mut mappers = Vec::with_capacity(fractal.len());
      for i in 0..fractal.len() {
        let mut hm = HandlerMemory::fork(hand_mem.clone())?;
        hm.register_out(args[0], i, CLOSURE_ARG_MEM_START + 1)?;
        hm.write_fixed(CLOSURE_ARG_MEM_START + 2, i as i64)?;
        mappers.push({
          let subhandler = subhandler.clone();
          async move {
            let hm = subhandler.run(hm).await?;
            Ok(hm.drop_parent()?)
          }
        });
      }
      let hms = join_all(mappers).await;
      hand_mem.init_fractal(args[2])?;
      for hm in hms {
        hand_mem.join(hm?)?;
        hand_mem.push_register(args[2], CLOSURE_ARG_MEM_START)?;
      }
      Ok(hand_mem)
    })
  });
  unpred_cpu!(mapl => fn(args, mut hand_mem) {
    Box::pin(async move {
      let fractal = hand_mem.read_fractal(args[0])?;
      let subhandler = HandlerFragment::new(args[1], 0);
      hand_mem.init_fractal(args[2])?;
      for i in 0..fractal.len() {
        hand_mem.register_out(args[0], i, CLOSURE_ARG_MEM_START + 1)?;
        hand_mem.write_fixed(CLOSURE_ARG_MEM_START + 2, i as i64)?;
        hand_mem = subhandler.clone().run(hand_mem).await?;
        hand_mem.push_register(args[2], CLOSURE_ARG_MEM_START)?;
      }
      Ok(hand_mem)
    })
  });
  cpu!(reparr => fn(args, hand_mem) {
    hand_mem.init_fractal(args[2])?;
    let n = hand_mem.read_fixed(args[1])?;
    if n == 0 {
      return Ok(None);
    }
    let fractal = hand_mem.read_fractal(args[0])?;
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
          hand_mem.push_fixed(args[2], val.read_fixed(0)?)?;
        } else {
          hand_mem.push_fractal(args[2], val.clone())?;
        }
      }
    }
    Ok(None)
  });
  io!(each => fn(args, hand_mem) {
    Box::pin(async move {
      if args[1] == NOP_ID {
        // each is expected to result in purely side effects
        return Ok(hand_mem);
      }
      let fractal = hand_mem.read_fractal(args[0])?;
      let subhandler = HandlerFragment::new(args[1], 0);
      let mut runners = Vec::with_capacity(fractal.len());
      for i in 0..fractal.len() {
        let mut hm = HandlerMemory::fork(hand_mem.clone())?;
        hm.register_out(args[0], i, CLOSURE_ARG_MEM_START + 1)?;
        hm.write_fixed(CLOSURE_ARG_MEM_START + 2, i as i64)?;
        runners.push({
          let subhandler = subhandler.clone();
          async move {
            let hm = subhandler.run(hm).await?;
            Ok(hm.drop_parent()?)
          }
        });
      }
      let runners: Vec<VMResult<_>> = join_all(runners).await;
      runners.into_iter().collect::<VMResult<Vec<_>>>()?;
      Ok(hand_mem)
    })
  });
  unpred_cpu!(eachl => fn(args, mut hand_mem) {
    Box::pin(async move {
      if args[1] == NOP_ID {
        // eachl is expected to result in purely side effects
        return Ok(hand_mem);
      }
      let n = hand_mem.read_fractal(args[0])?.len();
      let subhandler = HandlerFragment::new(args[1], 0);
      for i in 0..n {
        hand_mem.register_out(args[0], i, CLOSURE_ARG_MEM_START + 1)?;
        hand_mem.write_fixed(CLOSURE_ARG_MEM_START + 2, i as i64)?;
        hand_mem = subhandler.clone().run(hand_mem).await?;
      }
      Ok(hand_mem)
    })
  });
  io!(find => fn(args, mut hand_mem) {
    Box::pin(async move {
      if args[1] == NOP_ID {
        return Err(VMError::InvalidNOP);
      }
      let fractal = hand_mem.read_fractal(args[0])?;
      let len = fractal.len();
      let subhandler = HandlerFragment::new(args[1], 0);
      let mut finders = Vec::with_capacity(fractal.len());
      for i in 0..len {
        let mut hm = HandlerMemory::fork(hand_mem.clone())?;
        hm.register_out(args[0], i, CLOSURE_ARG_MEM_START + 1)?;
        finders.push(subhandler.clone().run(hm));
      }
      let hms = join_all(finders).await;
      let mut idx = None;
      for (i, hm) in hms.into_iter().enumerate() {
        let hm = hm?;
        let val = hm.read_fixed(CLOSURE_ARG_MEM_START)?;
        hm.drop_parent()?;
        if idx.is_none() && val == 1 {
          idx = Some(i);
        }
      }
      hand_mem.init_fractal(args[2])?;
      if let Some(idx) = idx {
        hand_mem.push_fixed(args[2], 1)?;
        hand_mem.push_register_out(args[2], &fractal, idx)?;
      } else {
        hand_mem.push_fixed(args[2], 0)?;
        hand_mem.push_fractal(args[2], HandlerMemory::str_to_fractal("no element matches"))?;
      }
      Ok(hand_mem)
    })
  });
  unpred_cpu!(findl => fn(args, mut hand_mem) {
    Box::pin(async move {
      if args[1] == NOP_ID {
        return Err(VMError::InvalidNOP);
      }
      let fractal = hand_mem.read_fractal(args[0])?;
      let subhandler = HandlerFragment::new(args[1], 0);
      for i in 0..fractal.len() {
        hand_mem.register_out(args[0], i, CLOSURE_ARG_MEM_START + 1)?;
        hand_mem = subhandler.clone().run(hand_mem).await?;
        let val = hand_mem.read_fixed(CLOSURE_ARG_MEM_START)?;
        if val == 1 {
          hand_mem.init_fractal(args[2])?;
          hand_mem.push_fixed(args[2], 1)?;
          hand_mem.push_register_out(args[2], &fractal, i)?;
          return Ok(hand_mem);
        }
      }
      hand_mem.init_fractal(args[2])?;
      hand_mem.push_fixed(args[2], 0)?;
      hand_mem.push_fractal(args[2], HandlerMemory::str_to_fractal("no element matches"))?;
      Ok(hand_mem)
    })
  });
  io!(some => fn(args, mut hand_mem) {
    Box::pin(async move {
      if args[1] == NOP_ID {
        return Err(VMError::InvalidNOP);
      }
      let fractal = hand_mem.read_fractal(args[0])?;
      let subhandler = HandlerFragment::new(args[1], 0);
      let mut somers = Vec::with_capacity(fractal.len());
      for i in 0..fractal.len() {
        let mut hm = HandlerMemory::fork(hand_mem.clone())?;
        hm.register_out(args[0], i, CLOSURE_ARG_MEM_START + 1)?;
        somers.push(subhandler.clone().run(hm));
      }
      let hms = join_all(somers).await;
      let mut ret_val = 0;
      for hm in hms {
        let hm = hm?;
        let val = hm.read_fixed(CLOSURE_ARG_MEM_START)?;
        hm.drop_parent()?;
        if val == 1 {
          ret_val = 1;
        }
      }
      hand_mem.write_fixed(args[2], ret_val)?;
      Ok(hand_mem)
    })
  });
  unpred_cpu!(somel => fn(args, mut hand_mem) {
    Box::pin(async move {
      if args[1] == NOP_ID {
        return Err(VMError::InvalidNOP);
      }
      let fractal = hand_mem.read_fractal(args[0])?;
      let subhandler = HandlerFragment::new(args[1], 0);
      for i in 0..fractal.len() {
        hand_mem.register_out(args[0], i, CLOSURE_ARG_MEM_START + 1)?;
        hand_mem = subhandler.clone().run(hand_mem).await?;
        let val = hand_mem.read_fixed(CLOSURE_ARG_MEM_START)?;
        if val == 1 {
          hand_mem.write_fixed(args[2], 1)?;
          return Ok(hand_mem);
        }
      }
      hand_mem.write_fixed(args[2], 0)?;
      Ok(hand_mem)
    })
  });
  io!(every => fn(args, mut hand_mem) {
    Box::pin(async move {
      if args[1] == NOP_ID {
        return Err(VMError::InvalidNOP);
      }
      let fractal = hand_mem.read_fractal(args[0])?;
      let subhandler = HandlerFragment::new(args[1], 0);
      let mut somers = Vec::with_capacity(fractal.len());
      for i in 0..fractal.len() {
        let mut hm = HandlerMemory::fork(hand_mem.clone())?;
        hm.register_out(args[0], i, CLOSURE_ARG_MEM_START + 1)?;
        somers.push(subhandler.clone().run(hm));
      }
      let hms = join_all(somers).await;
      let mut ret_val = 1;
      for hm in hms {
        let hm = hm?;
        let val = hm.read_fixed(CLOSURE_ARG_MEM_START)?;
        hm.drop_parent()?;
        if val == 0 {
          ret_val = 0;
        }
      }
      hand_mem.write_fixed(args[2], ret_val)?;
      Ok(hand_mem)
    })
  });
  unpred_cpu!(everyl => fn(args, mut hand_mem) {
    Box::pin(async move {
      if args[1] == NOP_ID {
        return Err(VMError::InvalidNOP);
      }
      let fractal = hand_mem.read_fractal(args[0])?;
      let subhandler = HandlerFragment::new(args[1], 0);
      for i in 0..fractal.len() {
        hand_mem.register_out(args[0], i, CLOSURE_ARG_MEM_START + 1)?;
        hand_mem = subhandler.clone().run(hand_mem).await?;
        let val = hand_mem.read_fixed(CLOSURE_ARG_MEM_START)?;
        if val == 0 {
          hand_mem.write_fixed(args[2], 0)?;
          return Ok(hand_mem);
        }
      }
      hand_mem.write_fixed(args[2], 1)?;
      Ok(hand_mem)
    })
  });
  cpu!(catarr => fn(args, hand_mem) {
    let fractal1 = hand_mem.read_fractal(args[0])?;
    let fractal2 = hand_mem.read_fractal(args[1])?;
    hand_mem.init_fractal(args[2])?;
    for i in 0..fractal1.len() {
      hand_mem.push_register_out(args[2], &fractal1, i)?;
    }
    for i in 0..fractal2.len() {
      hand_mem.push_register_out(args[2], &fractal2, i)?;
    }
    Ok(None)
  });
  io!(reducep => fn(args, mut hand_mem) {
    Box::pin(async move {
      if args[1] == NOP_ID {
        return Err(VMError::InvalidNOP);
      }
      let fractal = hand_mem.read_fractal(args[0])?;
      let mut vals = Vec::with_capacity(fractal.len());
      for i in 0..fractal.len() {
        let mut hm = HandlerMemory::new(None, 1)?;
        hand_mem.register_out(args[0], i, CLOSURE_ARG_MEM_START)?;
        HandlerMemory::transfer(&hand_mem, CLOSURE_ARG_MEM_START, &mut hm, 0)?;
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
          HandlerMemory::transfer(&a, 0, &mut hm, CLOSURE_ARG_MEM_START + 1)?;
          HandlerMemory::transfer(&b, 0, &mut hm, CLOSURE_ARG_MEM_START + 2)?;
          reducers.push(subhandler.clone().run(hm));
        }
        // Check if one of the records was skipped over this round, and if so, pop it into a
        // special field
        let maybe_hm = if vals.len() == 1 { Some(vals.remove(0)) } else { None };
        let hms = join_all(reducers).await;
        for hm in hms {
          let mut hm = hm?;
          hm.register(0, CLOSURE_ARG_MEM_START, false)?;
          vals.push(hm);
        }
        if let Some(hm) = maybe_hm {
          vals.push(hm);
        }
      }
      // There can be only one
      HandlerMemory::transfer(&vals[0], 0, &mut hand_mem, args[2])?;
      Ok(hand_mem)
    })
  });
  unpred_cpu!(reducel => fn(args, mut hand_mem) {
    Box::pin(async move {
      if args[1] == NOP_ID {
        return Err(VMError::InvalidNOP);
      }
      let fractal = hand_mem.read_fractal(args[0])?;
      if fractal.len() == 0 {
        return Ok(hand_mem);
      }
      let mut vals = Vec::with_capacity(fractal.len());
      for i in 0..fractal.len() {
        let mut hm = HandlerMemory::new(None, 1)?;
        hand_mem.register_out(args[0], i, CLOSURE_ARG_MEM_START)?;
        HandlerMemory::transfer(&hand_mem, CLOSURE_ARG_MEM_START, &mut hm, 0)?;
        vals.push(hm);
      }
      let subhandler = HandlerFragment::new(args[1], 0);
      let mut cumulative = vals.remove(0);
      for i in 0..vals.len() {
        let current = &vals[i];
        HandlerMemory::transfer(&cumulative, 0, &mut hand_mem, CLOSURE_ARG_MEM_START + 1)?;
        HandlerMemory::transfer(&current, 0, &mut hand_mem, CLOSURE_ARG_MEM_START + 2)?;
        hand_mem = subhandler.clone().run(hand_mem).await?;
        HandlerMemory::transfer(&hand_mem, CLOSURE_ARG_MEM_START, &mut cumulative, 0)?;
      }
      HandlerMemory::transfer(&cumulative, 0, &mut hand_mem, args[2])?;
      Ok(hand_mem)
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
        return Err(VMError::InvalidNOP);
      }
      let obj = hand_mem.read_fractal(args[0])?;
      let (arr, _) = hand_mem.read_from_fractal(&obj, 0);
      let mut vals = Vec::with_capacity(arr.len());
      for i in 0..arr.len() {
        let mut hm = HandlerMemory::new(None, 1)?;
        hand_mem.register_from_fractal(CLOSURE_ARG_MEM_START, &arr, i)?;
        HandlerMemory::transfer(&hand_mem, CLOSURE_ARG_MEM_START, &mut hm, 0)?;
        vals.push(hm);
      }
      let subhandler = HandlerFragment::new(args[1], 0);
      hand_mem.register_out(args[0], 1, CLOSURE_ARG_MEM_START)?;
      let mut cumulative = HandlerMemory::new(None, 1)?;
      HandlerMemory::transfer(&hand_mem, CLOSURE_ARG_MEM_START, &mut cumulative, 0)?;
      for i in 0..vals.len() {
        let current = &vals[i];
        HandlerMemory::transfer(&cumulative, 0, &mut hand_mem, CLOSURE_ARG_MEM_START + 1)?;
        HandlerMemory::transfer(current, 0, &mut hand_mem, CLOSURE_ARG_MEM_START + 2)?;
        hand_mem = subhandler.clone().run(hand_mem).await?;
        HandlerMemory::transfer(&hand_mem, CLOSURE_ARG_MEM_START, &mut cumulative, 0)?;
      }
      hand_mem.register(args[2], CLOSURE_ARG_MEM_START, false)?;
      Ok(hand_mem)
    })
  });
  io!(filter => fn(args, mut hand_mem) {
    Box::pin(async move {
      if args[1] == NOP_ID {
        return Err(VMError::InvalidNOP);
      }
      let fractal = hand_mem.read_fractal(args[0])?;
      let len = fractal.len();
      let subhandler = HandlerFragment::new(args[1], 0);
      let mut filters = Vec::with_capacity(len);
      for i in 0..len {
        let mut hm = HandlerMemory::fork(hand_mem.clone())?;
        hm.register_out(args[0], i, CLOSURE_ARG_MEM_START + 1)?;
        filters.push(subhandler.clone().run(hm));
      }
      let hms = join_all(filters).await;
      let mut idxs = vec![];
      for (i, hm) in hms.into_iter().enumerate() {
        let hm = hm?;
        let val = hm.read_fixed(CLOSURE_ARG_MEM_START)?;
        hm.drop_parent()?;
        if val == 1 {
          idxs.push(i);
        }
      }
      hand_mem.init_fractal(args[2])?;
      for i in idxs {
        hand_mem.push_register_out(args[2], &fractal, i)?;
      }
      Ok(hand_mem)
    })
  });
  unpred_cpu!(filterl => fn(args, mut hand_mem) {
    Box::pin(async move {
      if args[1] == NOP_ID {
        return Err(VMError::InvalidNOP);
      }
      let fractal = hand_mem.read_fractal(args[0])?;
      let len = fractal.len();
      let subhandler = HandlerFragment::new(args[1], 0);
      hand_mem.init_fractal(args[2])?;
      for i in 0..len {
        hand_mem.register_out(args[0], i, CLOSURE_ARG_MEM_START + 1)?;
        hand_mem = subhandler.clone().run(hand_mem).await?;
        let val = hand_mem.read_fixed(CLOSURE_ARG_MEM_START)?;
        if val == 1 {
          hand_mem.push_register_out(args[2], &fractal, i)?;
        }
      }
      Ok(hand_mem)
    })
  });

  // Conditional opcode
  unpred_cpu!(condfn => fn(args, mut hand_mem) {
    Box::pin(async move {
      let cond = hand_mem.read_fixed(args[0])?;
      let subhandler = HandlerFragment::new(args[1], 0);
      if cond == 1 {
        hand_mem = subhandler.run(hand_mem).await?;
      }
      Ok(hand_mem)
    })
  });

  // Std opcodes
  io!(execop => fn(args, mut hand_mem) {
    Box::pin(async move {
      let cmd = HandlerMemory::fractal_to_string(hand_mem.read_fractal(args[0])?)?;
      let output = if cfg!(target_os = "windows") {
        Command::new("cmd").arg("/C").arg(cmd).output().await
      } else {
        Command::new("sh").arg("-c").arg(cmd).output().await
      };
      hand_mem.init_fractal(args[2])?;
      match output {
        Err(e) => {
          hand_mem.push_fixed(args[2], 127)?;
          hand_mem.push_fractal(args[2], FractalMemory::new(vec![(0, 0)]))?;
          let error_string = e.to_string();
          hand_mem.push_fractal(args[2], HandlerMemory::str_to_fractal(&error_string))?;
        },
        Ok(output_res) => {
          let status_code = output_res.status.code().unwrap_or(127) as i64;
          hand_mem.push_fixed(args[2], status_code)?;
          let stdout_str = String::from_utf8(output_res.stdout).unwrap_or("".to_string());
          hand_mem.push_fractal(args[2], HandlerMemory::str_to_fractal(&stdout_str))?;
          let stderr_str = String::from_utf8(output_res.stderr).unwrap_or("".to_string());
          hand_mem.push_fractal(args[2], HandlerMemory::str_to_fractal(&stderr_str))?;
        },
      };
      Ok(hand_mem)
    })
  });

  unpred_cpu!(waitop => fn(args, hand_mem) {
    Box::pin(async move {
      let ms = hand_mem.read_fixed(args[0])? as u64;
      sleep(Duration::from_millis(ms)).await;
      Ok(hand_mem)
    })
  });

  unpred_cpu!(syncop => fn(args, mut hand_mem) {
    Box::pin(async move {
      let closure = HandlerFragment::new(args[0], 0);
      hand_mem.register(CLOSURE_ARG_MEM_START + 1, args[1], true)?;
      hand_mem = closure.clone().run(hand_mem).await?;
      hand_mem.register(args[2], CLOSURE_ARG_MEM_START, true)?;
      Ok(hand_mem)
    })
  });

  // IO opcodes
  fn __httpreq(
    method: String,
    uri: String,
    headers: Vec<(String, String)>,
    body: Option<String>,
  ) -> Result<ResponseFuture, String> {
    let mut req = Request::builder().method(method.as_str()).uri(uri.as_str());
    for header in headers {
      req = req.header(header.0.as_str(), header.1.as_str());
    }
    let req_obj = if let Some(body) = body {
      req.body(Body::from(body))
    } else {
      req.body(Body::empty())
    };
    match req_obj {
      Ok(req) => Ok(HTTP_CLIENT.request(req)),
      Err(_) => Err("Failed to construct request, invalid body provided".to_string()),
    }
  }
  io!(httpreq => fn(args, mut hand_mem) {
    Box::pin(async move {
      let req = hand_mem.read_fractal(args[0])?;
      let method = HandlerMemory::fractal_to_string(hand_mem.read_from_fractal(&req, 0).0)?;
      let url = HandlerMemory::fractal_to_string(hand_mem.read_from_fractal(&req, 1).0)?;
      let headers = hand_mem.read_from_fractal(&req, 2).0;
      let mut out_headers = Vec::new();
      for i in 0..headers.len() {
        let header = hand_mem.read_from_fractal(&headers, i).0;
        let key = HandlerMemory::fractal_to_string(hand_mem.read_from_fractal(&header, 0).0)?;
        let val = HandlerMemory::fractal_to_string(hand_mem.read_from_fractal(&header, 1).0)?;
        out_headers.push((key, val));
      }
      let body = HandlerMemory::fractal_to_string(hand_mem.read_from_fractal(&req, 3).0)?;
      let out_body = if body.len() > 0 { Some(body) /* once told me... */ } else { None };
      hand_mem.init_fractal(args[2])?;
      let res = match __httpreq(method, url, out_headers, out_body) {
        Ok(res) => res,
        Err(estring) => {
          hand_mem.push_fixed(args[2], 0i64)?;
          hand_mem.push_fractal(args[2], HandlerMemory::str_to_fractal(&estring))?;
          return Ok(hand_mem);
        },
      };
      let mut res = match res.await {
        Ok(res) => res,
        Err(ee) => {
          hand_mem.push_fixed(args[2], 0i64)?;
          hand_mem.push_fractal(
            args[2],
            HandlerMemory::str_to_fractal(format!("{}", ee).as_str())
          )?;
          return Ok(hand_mem);
        },
      };
      // The headers and body can fail, so check those first
      let headers = res.headers();
      let mut headers_hm = HandlerMemory::new(None, headers.len() as i64)?;
      headers_hm.init_fractal(CLOSURE_ARG_MEM_START)?;
      for (i, (key, val)) in headers.iter().enumerate() {
        let key_str = key.as_str();
        let val_str = val.to_str();
        match val_str {
          Ok(val_str) => {
            headers_hm.init_fractal(i as i64)?;
            headers_hm.push_fractal(i as i64, HandlerMemory::str_to_fractal(key_str))?;
            headers_hm.push_fractal(i as i64, HandlerMemory::str_to_fractal(val_str))?;
            headers_hm.push_register(CLOSURE_ARG_MEM_START, i as i64)?;
          },
          Err(_) => {
            hand_mem.push_fixed(args[2], 0i64)?;
            hand_mem.push_fractal(
              args[2],
              HandlerMemory::str_to_fractal("Malformed headers encountered")
            )?;
            return Ok(hand_mem);
          },
        }
      }
      let body = match hyper::body::to_bytes(res.body_mut()).await {
        Ok(body) => body,
        Err(ee) => {
          hand_mem.push_fixed(args[2], 0i64)?;
          hand_mem.push_fractal(
            args[2],
            HandlerMemory::str_to_fractal(format!("{}", ee).as_str())
          )?;
          return Ok(hand_mem);
        },
      };
      let body_str = match String::from_utf8(body.to_vec()) {
        Ok(body_str) => body_str,
        Err(ee) => {
          hand_mem.push_fixed(args[2], 0i64)?;
          hand_mem.push_fractal(
            args[2],
            HandlerMemory::str_to_fractal(format!("{}", ee).as_str())
          )?;
          return Ok(hand_mem);
        },
      };
      hand_mem.push_fixed(args[2], 1i64)?;
      let mut res_hm = HandlerMemory::new(None, 3)?;
      res_hm.init_fractal(0)?;
      res_hm.push_fixed(0, res.status().as_u16() as i64)?;
      HandlerMemory::transfer(&headers_hm, CLOSURE_ARG_MEM_START, &mut res_hm, CLOSURE_ARG_MEM_START)?;
      res_hm.push_register(0, CLOSURE_ARG_MEM_START)?;
      res_hm.push_fractal(0, HandlerMemory::str_to_fractal(&body_str))?;
      res_hm.push_fixed(0, 0i64)?;
      HandlerMemory::transfer(&res_hm, 0, &mut hand_mem, CLOSURE_ARG_MEM_START)?;
      hand_mem.push_register(args[2], CLOSURE_ARG_MEM_START)?;
      Ok(hand_mem)
    })
  });

  async fn http_listener(req: Request<Body>) -> VMResult<Response<Body>> {
    // Grab the headers
    let headers = req.headers();
    // Check if we should load balance this request. If the special `x-alan-rr` header is present,
    // that means it was already load-balanced to us and we should process it locally. If not, then
    // use a random number generator to decide if we should process this here or if we should
    // distribute the load to one of our local-region peers. This adds an extra network hop, but
    // within the same firewall group inside of the datacenter, so that part should be a minimal
    // impact on the total latency. This is done because cloudflare's routing is "sticky" to an
    // individual IP address without moving to a more expensive tier, so there's no actual load
    // balancing going on, just fallbacks in case of an outage. This adds stochastic load balancing
    // to the cluster even if we didn't have cloudflare fronting things.
    if !headers.contains_key("x-alan-rr") {
      let l = REGION_VMS.read().unwrap().len();
      let i = async move {
        let mut rng = thread_rng();
        rng.gen_range(0..=l)
      }
      .await;
      // If it's equal to the length process this request normally, otherwise, load balance this
      // request to another instance
      if i != l {
        // Otherwise, round-robin this to another node in the cluster and increment the counter
        let headers = headers.clone();
        let host = &REGION_VMS.read().unwrap()[i].clone();
        let method_str = req.method().to_string();
        let orig_uri = req.uri().clone();
        let orig_query = match orig_uri.query() {
          Some(q) => format!("?{}", q),
          None => format!(""),
        };
        let uri_str = format!("https://{}{}{}", host, orig_uri.path(), orig_query);
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
        let mut rr_req = Request::builder().method(method_str.as_str()).uri(uri_str);
        let rr_headers = rr_req.headers_mut().unwrap();
        let name = HeaderName::from_bytes("x-alan-rr".as_bytes()).unwrap();
        let value = HeaderValue::from_str("true").unwrap();
        rr_headers.insert(name, value);
        for (key, val) in headers.iter() {
          rr_headers.insert(key, val.clone());
        }
        let req_obj = if body_str.len() > 0 {
          rr_req.body(Body::from(body_str))
        } else {
          rr_req.body(Body::empty())
        };
        let req_obj = match req_obj {
          Ok(req_obj) => req_obj,
          Err(ee) => {
            return Ok(Response::new(
              format!("Connection terminated: {}", ee).into(),
            ));
          }
        };
        let mut rr_res = match NAIVE_CLIENT.get().unwrap().request(req_obj).await {
          Ok(res) => res,
          Err(ee) => {
            return Ok(Response::new(
              format!("Connection terminated: {}", ee).into(),
            ));
          }
        };
        // Get the status from the round-robin response and begin building the response object
        let status = rr_res.status();
        let mut res = Response::builder().status(status);
        // Get the headers and populate the response object
        let headers = res.headers_mut().unwrap();
        for (key, val) in rr_res.headers().iter() {
          headers.insert(key, val.clone());
        }
        let body = match hyper::body::to_bytes(rr_res.body_mut()).await {
          Ok(body) => body,
          Err(ee) => {
            return Ok(Response::new(
              format!("Connection terminated: {}", ee).into(),
            ));
          }
        };
        return Ok(res.body(body.into()).unwrap());
      }
    }
    // Create a new event handler memory to add to the event queue
    let mut event = HandlerMemory::new(None, 1)?;
    // Grab the method
    let method_str = req.method().to_string();
    let method = HandlerMemory::str_to_fractal(&method_str);
    // Grab the URL
    let orig_uri = req.uri().clone();
    let orig_query = match orig_uri.query() {
      Some(q) => format!("?{}", q),
      None => format!(""),
    };
    let url_str = format!("{}{}", orig_uri.path(), orig_query);
    //let url_str = req.uri().to_string();
    let url = HandlerMemory::str_to_fractal(&url_str);
    let mut headers_hm = HandlerMemory::new(None, headers.len() as i64)?;
    headers_hm.init_fractal(CLOSURE_ARG_MEM_START)?;
    for (i, (key, val)) in headers.iter().enumerate() {
      let key_str = key.as_str();
      // TODO: get rid of the potential panic here
      let val_str = val.to_str().unwrap();
      headers_hm.init_fractal(i as i64)?;
      headers_hm.push_fractal(i as i64, HandlerMemory::str_to_fractal(key_str))?;
      headers_hm.push_fractal(i as i64, HandlerMemory::str_to_fractal(val_str))?;
      headers_hm.push_register(CLOSURE_ARG_MEM_START, i as i64)?;
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
    // TODO: get rid of the potential panic here
    let body_str = str::from_utf8(&body_req).unwrap().to_string();
    let body = HandlerMemory::str_to_fractal(&body_str);
    // Populate the event and emit it
    event.init_fractal(0)?;
    event.push_fractal(0, method)?;
    event.push_fractal(0, url)?;
    HandlerMemory::transfer(
      &headers_hm,
      CLOSURE_ARG_MEM_START,
      &mut event,
      CLOSURE_ARG_MEM_START,
    )?;
    event.push_register(0, CLOSURE_ARG_MEM_START)?;
    event.push_fractal(0, body)?;
    // Generate a threadsafe raw ptr to the tx of a watch channel
    // A ptr is unsafely created from the raw ptr in httpsend once the
    // user's code has completed and sends the new HandlerMemory so we
    // can resume execution of this HTTP request
    let (tx, rx): (Sender<Arc<HandlerMemory>>, Receiver<Arc<HandlerMemory>>) = oneshot::channel();
    let tx_ptr = Box::into_raw(Box::new(tx)) as i64;
    event.push_fixed(0, tx_ptr)?;
    let event_emit = EventEmit {
      id: i64::from(BuiltInEvents::HTTPCONN),
      payload: Some(event),
    };
    let event_tx = EVENT_TX.get().ok_or(VMError::ShutDown)?;
    let mut err_res = Response::new("Error synchronizing `send` for HTTP request".into());
    *err_res.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
    if event_tx.send(event_emit).is_err() {
      return Ok(err_res);
    }
    // Await HTTP response from the user code
    let response_hm = match rx.await {
      Ok(hm) => hm,
      Err(_) => {
        return Ok(err_res);
      }
    };
    // Get the status from the user response and begin building the response object
    let status = response_hm.read_fixed(0)? as u16;
    let mut res = Response::builder()
      .status(StatusCode::from_u16(status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR));
    // Get the headers and populate the response object
    // TODO: figure out how to handle this potential panic
    let headers = res.headers_mut().unwrap();
    let header_hms = response_hm.read_fractal(1)?;
    for i in 0..header_hms.len() {
      let (h, _) = response_hm.read_from_fractal(&header_hms.clone(), i);
      let (key_hm, _) = response_hm.read_from_fractal(&h, 0);
      let (val_hm, _) = response_hm.read_from_fractal(&h, 1);
      let key = HandlerMemory::fractal_to_string(key_hm)?;
      let val = HandlerMemory::fractal_to_string(val_hm)?;
      // TODO: figure out how to handle this potential panic
      let name = HeaderName::from_bytes(key.as_bytes()).unwrap();
      // TODO: figure out how to handle this potential panic
      let value = HeaderValue::from_str(&val).unwrap();
      headers.insert(name, value);
    }
    // Get the body, populate the response object, and fire it out
    let body = HandlerMemory::fractal_to_string(response_hm.read_fractal(2)?)?;
    // TODO: figure out how to handle this potential panic
    Ok(res.body(body.into()).unwrap())
  }
  io!(httplsn => fn(_args, hand_mem) {
    Box::pin(async move {
      // this extra fn is so that we can just use `?` inside of http_listener instead of
      // having a bunch of `match`es that call a closure
      async fn listen(req: Request<Body>) -> Result<Response<Body>, Infallible> {
        match http_listener(req).await {
          Ok(res) => Ok(res),
          Err(_) => {
            // TODO: log the error?
            Ok(Response::builder().status(500).body(Body::empty()).unwrap())
          }
        }
      }
      make_server!(&Program::global().http_config, listen);
      return Ok(hand_mem);
    })
  });
  cpu!(httpsend => fn(args, hand_mem) {
    hand_mem.dupe(args[0], args[0])?; // Make sure there's no pointers involved
    let mut hm = HandlerMemory::new(None, 1)?;
    HandlerMemory::transfer(&hand_mem, args[0], &mut hm, CLOSURE_ARG_MEM_START)?;
    let res_out = hm.read_fractal(CLOSURE_ARG_MEM_START)?;
    for i in 0..res_out.len() {
      hm.register_from_fractal(i as i64, &res_out, i)?;
    }
    // Get the oneshot channel tx from the raw ptr previously generated in http_listener
    let fractal = hand_mem.read_fractal(args[0])?;
    let tx_ptr = NonNull::new(fractal.read_fixed(3)? as *mut Sender<Arc<HandlerMemory>>);
    if let Some(tx_nonnull) = tx_ptr {
      let tx = unsafe { Box::from_raw(tx_nonnull.as_ptr()) };
      let (status, string) = match tx.send(hm) {
        Ok(_) => (1, "ok"),
        Err(_) => (0, "could not send response to server"),
      };
      hand_mem.init_fractal(args[2])?;
      hand_mem.push_fixed(args[2], status)?;
      hand_mem.push_fractal(args[2], HandlerMemory::str_to_fractal(string))?;
    } else {
      hand_mem.init_fractal(args[2])?;
      hand_mem.push_fixed(args[2], 0)?;
      hand_mem.push_fractal(
        args[2],
        HandlerMemory::str_to_fractal("cannot call send twice for the same connection")
      )?;
    }
    Ok(None)
  });

  // Datastore opcodes
  io!(dssetf => fn(args, hand_mem) {
    Box::pin(async move {
      let val = hand_mem.read_fixed(args[2])?;
      let mut hm = HandlerMemory::new(None, 1)?;
      hm.write_fixed(0, val)?;
      let ns = HandlerMemory::fractal_to_string(hand_mem.read_fractal(args[0])?)?;
      let key = HandlerMemory::fractal_to_string(hand_mem.read_fractal(args[1])?)?;
      let nskey = format!("{}:{}", ns, key);
      let ctrl_port = CONTROL_PORT_CHANNEL.get();
      let ctrl_port = match ctrl_port {
        Some(ctrl_port) => Some(ctrl_port.borrow().clone()), // TODO: Use thread-local storage
        None => None,
      };
      let is_key_owner = match ctrl_port {
        Some(ref ctrl_port) => ctrl_port.is_key_owner(&nskey),
        None => true,
      };
      if is_key_owner {
        DS.insert(nskey, hm);
      } else {
        ctrl_port.unwrap().dssetf(&nskey, &hm).await;
      }
      Ok(hand_mem)
    })
  });
  io!(dssetv => fn(args, hand_mem) {
    Box::pin(async move {
      let mut hm = HandlerMemory::new(None, 1)?;
      HandlerMemory::transfer(&hand_mem, args[2], &mut hm, 0)?;
      let ns = HandlerMemory::fractal_to_string(hand_mem.read_fractal(args[0])?)?;
      let key = HandlerMemory::fractal_to_string(hand_mem.read_fractal(args[1])?)?;
      let nskey = format!("{}:{}", ns, key);
      let ctrl_port = CONTROL_PORT_CHANNEL.get();
      let ctrl_port = match ctrl_port {
        Some(ctrl_port) => Some(ctrl_port.borrow().clone()), // TODO: Use thread-local storage
        None => None,
      };
      let is_key_owner = match ctrl_port {
        Some(ref ctrl_port) => ctrl_port.is_key_owner(&nskey),
        None => true,
      };
      if is_key_owner {
        DS.insert(nskey, hm);
      } else {
        ctrl_port.unwrap().dssetv(&nskey, &hm).await;
      }
      Ok(hand_mem)
    })
  });
  io!(dshas => fn(args, mut hand_mem) {
    Box::pin(async move {
      let ns = HandlerMemory::fractal_to_string(hand_mem.read_fractal(args[0])?)?;
      let key = HandlerMemory::fractal_to_string(hand_mem.read_fractal(args[1])?)?;
      let nskey = format!("{}:{}", ns, key);
      let ctrl_port = CONTROL_PORT_CHANNEL.get();
      let ctrl_port = match ctrl_port {
        Some(ctrl_port) => Some(ctrl_port.borrow().clone()), // TODO: Use thread-local storage
        None => None,
      };
      let is_key_owner = match ctrl_port {
        Some(ref ctrl_port) => ctrl_port.is_key_owner(&nskey),
        None => true,
      };
      let has = if is_key_owner {
        DS.contains_key(&nskey)
      } else {
        ctrl_port.unwrap().dshas(&nskey).await
      };
      hand_mem.write_fixed(args[2], if has { 1i64 } else { 0i64 })?;
      Ok(hand_mem)
    })
  });
  io!(dsdel => fn(args, mut hand_mem) {
    Box::pin(async move {
      let ns = HandlerMemory::fractal_to_string(hand_mem.read_fractal(args[0])?)?;
      let key = HandlerMemory::fractal_to_string(hand_mem.read_fractal(args[1])?)?;
      let nskey = format!("{}:{}", ns, key);
      // If it exists locally, remove it here, too
      let removed = DS.remove(&nskey).is_some();
      let ctrl_port = CONTROL_PORT_CHANNEL.get();
      let ctrl_port = match ctrl_port {
        Some(ctrl_port) => Some(ctrl_port.borrow().clone()), // TODO: Use thread-local storage
        None => None,
      };
      let is_key_owner = match ctrl_port {
        Some(ref ctrl_port) => ctrl_port.is_key_owner(&nskey),
        None => true,
      };
      let removed = if is_key_owner {
        removed
      } else {
        ctrl_port.unwrap().dsdel(&nskey).await
      };
      hand_mem.write_fixed(args[2], if removed { 1i64 } else { 0i64 })?;
      Ok(hand_mem)
    })
  });
  io!(dsgetf => fn(args, mut hand_mem) {
    Box::pin(async move {
      let ns = HandlerMemory::fractal_to_string(hand_mem.read_fractal(args[0])?)?;
      let key = HandlerMemory::fractal_to_string(hand_mem.read_fractal(args[1])?)?;
      let nskey = format!("{}:{}", ns, key);
      let ctrl_port = CONTROL_PORT_CHANNEL.get();
      let ctrl_port = match ctrl_port {
        Some(ctrl_port) => Some(ctrl_port.borrow().clone()), // TODO: Use thread-local storage
        None => None,
      };
      let is_key_owner = match ctrl_port {
        Some(ref ctrl_port) => ctrl_port.is_key_owner(&nskey),
        None => true,
      };
      if is_key_owner {
        hand_mem.init_fractal(args[2])?;
        let maybe_hm = DS.get(&nskey);
        match maybe_hm {
          Some(hm) => {
            hand_mem.push_fixed(args[2], 1i64)?;
            hand_mem.push_fixed(args[2], hm.read_fixed(0)?)?;
          },
          None => {
            hand_mem.push_fixed(args[2], 0i64)?;
            hand_mem.push_fractal(args[2], HandlerMemory::str_to_fractal("namespace-key pair not found"))?;
          },
        }
      } else {
        let maybe_hm = ctrl_port.unwrap().dsgetf(&nskey).await;
        match maybe_hm {
          Some(hm) => {
            HandlerMemory::transfer(&hm, 0, &mut hand_mem, args[2])?;
          },
          None => {
            hand_mem.init_fractal(args[2])?;
            hand_mem.push_fixed(args[2], 0i64)?;
            hand_mem.push_fractal(args[2], HandlerMemory::str_to_fractal("namespace-key pair not found"))?;
          },
        }
      };
      Ok(hand_mem)
    })
  });
  io!(dsgetv => fn(args, mut hand_mem) {
    Box::pin(async move {
      let ns = HandlerMemory::fractal_to_string(hand_mem.read_fractal(args[0])?)?;
      let key = HandlerMemory::fractal_to_string(hand_mem.read_fractal(args[1])?)?;
      let nskey = format!("{}:{}", ns, key);
      let ctrl_port = CONTROL_PORT_CHANNEL.get();
      let ctrl_port = match ctrl_port {
        Some(ctrl_port) => Some(ctrl_port.borrow().clone()), // TODO: Use thread-local storage
        None => None,
      };
      let is_key_owner = match ctrl_port {
        Some(ref ctrl_port) => ctrl_port.is_key_owner(&nskey),
        None => true,
      };
      if is_key_owner {
        hand_mem.init_fractal(args[2])?;
        let maybe_hm = DS.get(&nskey);
        match maybe_hm {
          Some(hm) => {
            hand_mem.push_fixed(args[2], 1i64)?;
            HandlerMemory::transfer(&hm, 0, &mut hand_mem, CLOSURE_ARG_MEM_START)?;
            hand_mem.push_register(args[2], CLOSURE_ARG_MEM_START)?;
          },
          None => {
            hand_mem.push_fixed(args[2], 0i64)?;
            hand_mem.push_fractal(args[2], HandlerMemory::str_to_fractal("namespace-key pair not found"))?;
          },
        }
      } else {
        let maybe_hm = ctrl_port.unwrap().dsgetv(&nskey).await;
        match maybe_hm {
          Some(hm) => {
            HandlerMemory::transfer(&hm, 0, &mut hand_mem, args[2])?;
          },
          None => {
            hand_mem.init_fractal(args[2])?;
            hand_mem.push_fixed(args[2], 0i64)?;
            hand_mem.push_fractal(args[2], HandlerMemory::str_to_fractal("namespace-key pair not found"))?;
          },
        }
      }
      Ok(hand_mem)
    })
  });
  io!(getclustersecret => fn(args, mut hand_mem) {
    Box::pin(async move {
      hand_mem.init_fractal(args[2])?;
      match CLUSTER_SECRET.get().unwrap() {
        Some(cluster_secret) => {
          hand_mem.push_fixed(args[2], 1i64)?;
          hand_mem.push_fractal(args[2], HandlerMemory::str_to_fractal(cluster_secret))?;
        },
        None => {
          hand_mem.push_fixed(args[2], 0i64)?;
          hand_mem.push_fractal(args[2], HandlerMemory::str_to_fractal(""))?;
        },
      };
      Ok(hand_mem)
    })
  });

  // seq opcodes
  cpu!(newseq => fn(args, hand_mem) {
    hand_mem.init_fractal(args[2])?;
    hand_mem.push_fixed(args[2], 0i64)?;
    hand_mem.push_fixed(args[2], hand_mem.read_fixed(args[0])?)?;
    Ok(None)
  });
  cpu!(seqnext => fn(args, hand_mem) {
    hand_mem.init_fractal(args[2])?;
    let mut seq = hand_mem.read_fractal(args[0])?;
    let current = seq.read_fixed(0)?;
    let limit = seq.read_fixed(1)?;
    if current < limit {
      hand_mem.write_fixed_in_fractal(&mut seq, 0, current + 1)?;
      hand_mem.push_fixed(args[2], 1i64)?;
      hand_mem.push_fixed(args[2], current)?;
    } else {
      hand_mem.push_fixed(args[2], 0i64)?;
      let err_msg = "error: sequence out-of-bounds";
      hand_mem.push_fractal(args[2], HandlerMemory::str_to_fractal(&err_msg))?;
    }
    Ok(None)
  });
  unpred_cpu!(seqeach => fn(args, mut hand_mem) {
    Box::pin(async move {
      if args[1] == NOP_ID {
        // same as `each`
        return Ok(hand_mem);
      }
      let mut seq = hand_mem.read_fractal(args[0])?;
      let current = seq.read_fixed(0)?;
      let limit = seq.read_fixed(1)?;
      let subhandler = HandlerFragment::new(args[1], 0);
      if current >= limit {
        return Ok(hand_mem);
      }
      hand_mem.write_fixed_in_fractal(&mut seq, 0, limit)?;
      // array of potentially many levels of nested fractals
      for i in current..limit {
        // array element is $1 argument of the closure memory space
        hand_mem.write_fixed(CLOSURE_ARG_MEM_START + 1, i)?;
        hand_mem = subhandler.clone().run(hand_mem).await?;
      }
      Ok(hand_mem)
    })
  });
  unpred_cpu!(seqwhile => fn(args, mut hand_mem) {
    Box::pin(async move {
      if args[1] == NOP_ID {
        return Err(VMError::InvalidNOP);
      }
      let seq = hand_mem.read_fractal(args[0])?;
      let mut current = seq.read_fixed(0)?;
      let limit = seq.read_fixed(1)?;
      drop(seq);
      let cond_handler = HandlerFragment::new(args[1], 0);
      let body_handler = HandlerFragment::new(args[2], 0);
      if current >= limit {
        return Ok(hand_mem);
      }
      hand_mem = cond_handler.clone().run(hand_mem).await?;
      while current < limit && hand_mem.read_fixed(CLOSURE_ARG_MEM_START)? > 0 {
        if args[2] != NOP_ID {
          hand_mem = body_handler.clone().run(hand_mem).await?;
        }
        current = current + 1;
        hand_mem = cond_handler.clone().run(hand_mem).await?;
      }
      let mut seq = hand_mem.read_fractal(args[0])?;
      hand_mem.write_fixed_in_fractal(&mut seq, 0, current)?;
      Ok(hand_mem)
    })
  });
  unpred_cpu!(seqdo => fn(args, mut hand_mem) {
    Box::pin(async move {
      let seq = hand_mem.read_fractal(args[0])?;
      let mut current = seq.read_fixed(0)?;
      let limit = seq.read_fixed(1)?;
      drop(seq);
      let subhandler = HandlerFragment::new(args[1], 0);
      loop {
        if args[1] != NOP_ID {
          hand_mem = subhandler.clone().run(hand_mem).await?;
        }
        current = current + 1;
        if current >= limit || hand_mem.read_fixed(CLOSURE_ARG_MEM_START)? == 0 {
          break;
        }
      }
      let mut seq = hand_mem.read_fractal(args[0])?;
      hand_mem.write_fixed_in_fractal(&mut seq, 0, current)?;
      Ok(hand_mem)
    })
  });
  unpred_cpu!(selfrec => fn(args, mut hand_mem) {
    Box::pin(async move {
      let mut hm = HandlerMemory::fork(hand_mem.clone())?;
      // MUST read these first in case the arguments are themselves closure args being overwritten
      // for the recursive function.
      // Since we mutate the `Self` object in this, it *must* be read as mutable *first* to make
      // sure that the later registration of the `Self` object is pointing at the correct copy
      let slf = hm.read_mut_fractal(args[0])?;
      let recurse_fn = HandlerFragment::new(slf[1].1, 0);
      let seq_addr = slf[0].0;
      drop(slf);
      hm.register(CLOSURE_ARG_MEM_START + 1, args[0], false)?;
      hm.register(CLOSURE_ARG_MEM_START + 2, args[1], false)?;
      let seq = hm.read_mut_fractal_by_idx(seq_addr)?;
      let curr = seq[0].1;
      if curr < seq[1].1 {
        seq[0].1 = curr + 1;
        hm = recurse_fn.run(hm).await?;
        hm = hm.drop_parent()?;
        // CANNOT `join` the memory like usual because the nested `recurse` calls have set "future"
        // values in the handler and will cause weird behavior. Only transfer the Self mutation and
        // the return value between iterations
        HandlerMemory::transfer(&mut hm, CLOSURE_ARG_MEM_START, &mut hand_mem, args[2])?;
        HandlerMemory::transfer(&mut hm, CLOSURE_ARG_MEM_START + 1, &mut hand_mem, args[0])?;
      } else {
        hand_mem.init_fractal(args[2])?;
        hand_mem.push_fixed(args[2], 0)?;
        hand_mem.push_fractal(args[2], HandlerMemory::str_to_fractal("error: sequence out-of-bounds"))?;
      }
      Ok(hand_mem)
    })
  });
  cpu!(seqrec => fn(args, hand_mem) {
    if args[1] == NOP_ID {
      return Err(VMError::InvalidNOP);
    }
    hand_mem.init_fractal(args[2])?;
    hand_mem.push_register(args[2], args[0])?;
    hand_mem.push_fixed(args[2], args[1])?;
    Ok(None)
  });

  // "Special" opcodes
  cpu!(exitop => fn(args, hand_mem) {
    io::stdout().flush().map_err(VMError::IOError)?;
    io::stderr().flush().map_err(VMError::IOError)?;
    std::process::exit(hand_mem.read_fixed(args[0])? as i32);
  });
  cpu!(stdoutp => fn(args, hand_mem) {
    let out_str = HandlerMemory::fractal_to_string(hand_mem.read_fractal(args[0])?)?;
    print!("{}", out_str);
    Ok(None)
  });
  cpu!(stderrp => fn(args, hand_mem) {
    let err_str = HandlerMemory::fractal_to_string(hand_mem.read_fractal(args[0])?)?;
    eprint!("{}", err_str);
    Ok(None)
  });

  // set opcodes use args[0] directly, since the relevant value directly
  // fits in i64, and write it to args[2]
  cpu!(seti64 => fn(args, hand_mem) {
    let data = args[0];
    hand_mem.write_fixed(args[2], data)?;
    Ok(None)
  });
  cpu!(seti32 => fn(args, hand_mem) {
    let data = (args[0] as i32) as i64;
    hand_mem.write_fixed(args[2], data)?;
    Ok(None)
  });
  cpu!(seti16 => fn(args, hand_mem) {
    let data = (args[0] as i16) as i64;
    hand_mem.write_fixed(args[2], data)?;
    Ok(None)
  });
  cpu!(seti8 => fn(args, hand_mem) {
    let data = (args[0] as i8) as i64;
    hand_mem.write_fixed(args[2], data)?;
    Ok(None)
  });
  cpu!(setf64 => fn(args, hand_mem) {
    let data = i64::from_ne_bytes((args[0] as f64).to_ne_bytes());
    hand_mem.write_fixed(args[2], data)?;
    Ok(None)
  });
  cpu!(setf32 => fn(args, hand_mem) {
    let data = i32::from_ne_bytes((args[0] as f32).to_ne_bytes()) as i64;
    hand_mem.write_fixed(args[2], data)?;
    Ok(None)
  });
  cpu!(setbool => fn(args, hand_mem) {
    let data = if args[0] == 0 { 0i64 } else { 1i64 };
    hand_mem.write_fixed(args[2], data)?;
    Ok(None)
  });
  cpu!(setestr => fn(args, hand_mem) {
    let empty_str = FractalMemory::new(vec![(0, 0)]);
    hand_mem.write_fractal(args[2], &empty_str)?;
    Ok(None)
  });

  // copy opcodes used for let variable reassignments
  cpu!(copyi8 => fn(args, hand_mem) {
    let val = hand_mem.read_fixed(args[0])?;
    hand_mem.write_fixed(args[2], val)?;
    Ok(None)
  });
  cpu!(copyi16 => fn(args, hand_mem) {
    let val = hand_mem.read_fixed(args[0])?;
    hand_mem.write_fixed(args[2], val)?;
    Ok(None)
  });
  cpu!(copyi32 => fn(args, hand_mem) {
    let val = hand_mem.read_fixed(args[0])?;
    hand_mem.write_fixed(args[2], val)?;
    Ok(None)
  });
  cpu!(copyi64 => fn(args, hand_mem) {
    let val = hand_mem.read_fixed(args[0])?;
    hand_mem.write_fixed(args[2], val)?;
    Ok(None)
  });
  cpu!(copyvoid => fn(args, hand_mem) {
    let val = hand_mem.read_fixed(args[0])?;
    hand_mem.write_fixed(args[2], val)?;
    Ok(None)
  });
  cpu!(copyf32 => fn(args, hand_mem) {
    let val = hand_mem.read_fixed(args[0])?;
    hand_mem.write_fixed(args[2], val)?;
    Ok(None)
  });
  cpu!(copyf64 => fn(args, hand_mem) {
    let val = hand_mem.read_fixed(args[0])?;
    hand_mem.write_fixed(args[2], val)?;
    Ok(None)
  });
  cpu!(copybool => fn(args, hand_mem) {
    let val = hand_mem.read_fixed(args[0])?;
    hand_mem.write_fixed(args[2], val)?;
    Ok(None)
  });
  cpu!(copystr => fn(args, hand_mem) {
    let pascal_string = hand_mem.read_fractal(args[0])?;
    hand_mem.write_fractal(args[2], &pascal_string)?;
    Ok(None)
  });
  cpu!(copyarr => fn(args, hand_mem) {
    // args = [in_addr, unused, out_addr]
    hand_mem.dupe(args[0], args[2])?;
    Ok(None)
  });
  cpu!(zeroed => fn(args, hand_mem) {
    hand_mem.write_fixed(args[2], 0)?;
    Ok(None)
  });

  // Trig opcodes
  cpu!(lnf64 => fn(args, hand_mem) {
    let a = f64::from_ne_bytes(hand_mem.read_fixed(args[0])?.to_ne_bytes());
    let out = i64::from_ne_bytes(a.ln().to_ne_bytes());
    hand_mem.write_fixed(args[2], out)?;
    Ok(None)
  });
  cpu!(logf64 => fn(args, hand_mem) {
    let a = f64::from_ne_bytes(hand_mem.read_fixed(args[0])?.to_ne_bytes());
    let out = i64::from_ne_bytes(a.log10().to_ne_bytes());
    hand_mem.write_fixed(args[2], out)?;
    Ok(None)
  });
  cpu!(sinf64 => fn(args, hand_mem) {
    let a = f64::from_ne_bytes(hand_mem.read_fixed(args[0])?.to_ne_bytes());
    let out = i64::from_ne_bytes(a.sin().to_ne_bytes());
    hand_mem.write_fixed(args[2], out)?;
    Ok(None)
  });
  cpu!(cosf64 => fn(args, hand_mem) {
    let a = f64::from_ne_bytes(hand_mem.read_fixed(args[0])?.to_ne_bytes());
    let out = i64::from_ne_bytes(a.cos().to_ne_bytes());
    hand_mem.write_fixed(args[2], out)?;
    Ok(None)
  });
  cpu!(tanf64 => fn(args, hand_mem) {
    let a = f64::from_ne_bytes(hand_mem.read_fixed(args[0])?.to_ne_bytes());
    let out = i64::from_ne_bytes(a.tan().to_ne_bytes());
    hand_mem.write_fixed(args[2], out)?;
    Ok(None)
  });
  cpu!(asinf64 => fn(args, hand_mem) {
    let a = f64::from_ne_bytes(hand_mem.read_fixed(args[0])?.to_ne_bytes());
    let out = i64::from_ne_bytes(a.asin().to_ne_bytes());
    hand_mem.write_fixed(args[2], out)?;
    Ok(None)
  });
  cpu!(acosf64 => fn(args, hand_mem) {
    let a = f64::from_ne_bytes(hand_mem.read_fixed(args[0])?.to_ne_bytes());
    let out = i64::from_ne_bytes(a.acos().to_ne_bytes());
    hand_mem.write_fixed(args[2], out)?;
    Ok(None)
  });
  cpu!(atanf64 => fn(args, hand_mem) {
    let a = f64::from_ne_bytes(hand_mem.read_fixed(args[0])?.to_ne_bytes());
    let out = i64::from_ne_bytes(a.atan().to_ne_bytes());
    hand_mem.write_fixed(args[2], out)?;
    Ok(None)
  });
  cpu!(sinhf64 => fn(args, hand_mem) {
    let a = f64::from_ne_bytes(hand_mem.read_fixed(args[0])?.to_ne_bytes());
    let out = i64::from_ne_bytes(a.sinh().to_ne_bytes());
    hand_mem.write_fixed(args[2], out)?;
    Ok(None)
  });
  cpu!(coshf64 => fn(args, hand_mem) {
    let a = f64::from_ne_bytes(hand_mem.read_fixed(args[0])?.to_ne_bytes());
    let out = i64::from_ne_bytes(a.cosh().to_ne_bytes());
    hand_mem.write_fixed(args[2], out)?;
    Ok(None)
  });
  cpu!(tanhf64 => fn(args, hand_mem) {
    let a = f64::from_ne_bytes(hand_mem.read_fixed(args[0])?.to_ne_bytes());
    let out = i64::from_ne_bytes(a.tanh().to_ne_bytes());
    hand_mem.write_fixed(args[2], out)?;
    Ok(None)
  });

  // Error, Maybe, Result, Either opcodes
  cpu!(error => fn(args, hand_mem) {
    hand_mem.register(args[2], args[0], true)?;
    Ok(None)
  });
  cpu!(refv => fn(args, hand_mem) {
    hand_mem.register(args[2], args[0], true)?;
    Ok(None)
  });
  cpu!(reff => fn(args, hand_mem) {
    hand_mem.register(args[2], args[0], false)?;
    Ok(None)
  });
  cpu!(noerr => fn(args, hand_mem) {
    let empty_string = FractalMemory::new(vec![(0, 0)]);
    hand_mem.write_fractal(args[2], &empty_string)?;
    Ok(None)
  });
  cpu!(errorstr => fn(args, hand_mem) {
    hand_mem.register(args[2], args[0], true)?;
    Ok(None)
  });
  cpu!(someM => fn(args, hand_mem) {
    hand_mem.init_fractal(args[2])?;
    hand_mem.push_fixed(args[2], 1i64)?;
    let val_size = hand_mem.read_fixed(args[1])?;
    if val_size == 0 {
      hand_mem.push_register(args[2], args[0])?;
    } else {
      let val = hand_mem.read_fixed(args[0])?;
      hand_mem.push_fixed(args[2], val)?;
    }
    Ok(None)
  });
  cpu!(noneM => fn(args, hand_mem) {
    hand_mem.init_fractal(args[2])?;
    hand_mem.push_fixed(args[2], 0i64)?;
    Ok(None)
  });
  cpu!(isSome => fn(args, hand_mem) {
    hand_mem.register_out(args[0], 0, args[2])?;
    Ok(None)
  });
  cpu!(isNone => fn(args, hand_mem) {
    let fractal = hand_mem.read_fractal(args[0])?;
    let val = fractal.read_fixed(0)?;
    hand_mem.write_fixed(args[2], if val == 0i64 { 1i64 } else { 0i64 })?;
    Ok(None)
  });
  cpu!(getOrM => fn(args, hand_mem) {
    let fractal = hand_mem.read_fractal(args[0])?;
    let val = fractal.read_fixed(0)?;
    if val == 1i64 {
      hand_mem.register_out(args[0], 1, args[2])?;
    } else {
      if args[1] < 0 {
        let val = hand_mem.read_fixed(args[1])?;
        hand_mem.write_fixed(args[2], val)?;
      } else {
        let (data, is_fractal) = hand_mem.read_either(args[1])?;
        if is_fractal {
          hand_mem.register(args[2], args[1], true)?;
        } else {
          hand_mem.write_fixed(args[2], data.read_fixed(0)?)?;
        }
      }
    }
    Ok(None)
  });
  cpu!(getMaybe => fn(args, hand_mem) {
    let fractal = hand_mem.read_fractal(args[0])?;
    let variant = fractal.read_fixed(0)?;
    if variant == 1 {
      hand_mem.register_out(args[0], 1, args[2])?;
      Ok(None)
    } else {
      Err(VMError::IllegalAccess)
    }
  });
  cpu!(okR => fn(args, hand_mem) {
    hand_mem.init_fractal(args[2])?;
    hand_mem.push_fixed(args[2], 1i64)?;
    let val_size = hand_mem.read_fixed(args[1])?;
    if val_size == 0 {
      hand_mem.push_register(args[2], args[0])?;
    } else {
      let val = hand_mem.read_fixed(args[0])?;
      hand_mem.push_fixed(args[2], val)?;
    }
    Ok(None)
  });
  cpu!(err => fn(args, hand_mem) {
    hand_mem.init_fractal(args[2])?;
    hand_mem.push_fixed(args[2], 0i64)?;
    hand_mem.push_register(args[2], args[0])?;
    Ok(None)
  });
  cpu!(isOk => fn(args, hand_mem) {
    hand_mem.register_out(args[0], 0, args[2])?;
    Ok(None)
  });
  cpu!(isErr => fn(args, hand_mem) {
    let fractal = hand_mem.read_fractal(args[0])?;
    let val = fractal.read_fixed(0)?;
    hand_mem.write_fixed(args[2], if val == 0i64 { 1i64 } else { 0i64 })?;
    Ok(None)
  });
  cpu!(getOrR => fn(args, hand_mem) {
    let fractal = hand_mem.read_fractal(args[0])?;
    let val = fractal.read_fixed(0)?;
    if val == 1i64 {
      hand_mem.register_out(args[0], 1, args[2])?;
    } else {
      let (data, is_fractal) = hand_mem.read_either(args[1])?;
      if is_fractal {
        hand_mem.register(args[2], args[1], true)?;
      } else {
        hand_mem.write_fixed(args[2], data.read_fixed(0)?)?;
      }
    }
    Ok(None)
  });
  cpu!(getOrRS => fn(args, hand_mem) {
    let fractal = hand_mem.read_fractal(args[0])?;
    let val = fractal.read_fixed(0)?;
    if val == 1i64 {
      hand_mem.register_out(args[0], 1, args[2])?;
    } else {
      let f = HandlerMemory::str_to_fractal(&HandlerMemory::fractal_to_string(hand_mem.read_fractal(args[1])?)?);
      hand_mem.write_fractal(args[2], &f)?;
    }
    Ok(None)
  });
  cpu!(getR => fn(args, hand_mem) {
    let fractal = hand_mem.read_fractal(args[0])?;
    let val = fractal.read_fixed(0)?;
    if val == 1i64 {
      hand_mem.register_out(args[0], 1, args[2])?;
      Ok(None)
    } else {
      Err(VMError::IllegalAccess)
    }
  });
  cpu!(getErr => fn(args, hand_mem) {
    let fractal = hand_mem.read_fractal(args[0])?;
    let val = fractal.read_fixed(0)?;
    if val == 0i64 {
      hand_mem.register_out(args[0], 1, args[2])?;
    } else {
      let (data, is_fractal) = hand_mem.read_either(args[1])?;
      if is_fractal {
        hand_mem.register(args[2], args[1], true)?;
      } else {
        hand_mem.write_fixed(args[2], data.read_fixed(0)?)?;
      }
    }
    Ok(None)
  });
  cpu!(resfrom => fn(args, hand_mem) {
    // args = [arr_addr, arr_idx_addr, outer_addr]
    // a guarded copy of data from an array to a result object
    hand_mem.init_fractal(args[2])?;
    let fractal = hand_mem.read_fractal(args[1])?;
    let val = fractal.read_fixed(0)?;
    if val == 0 {
      hand_mem.write_fractal(args[2], &fractal)?;
      return Ok(None);
    }
    let inner_addr = fractal.read_fixed(1)? as usize;
    let arr = hand_mem.read_fractal(args[0])?;
    if arr.len() > inner_addr {
      hand_mem.push_fixed(args[2], 1)?;
      hand_mem.push_register_out(args[2], &arr, inner_addr)?;
    } else {
      hand_mem.push_fixed(args[2], 0)?;
      hand_mem.push_fractal(args[2], HandlerMemory::str_to_fractal("out-of-bounds access"))?;
    }
    Ok(None)
  });
  cpu!(mainE => fn(args, hand_mem) {
    hand_mem.init_fractal(args[2])?;
    hand_mem.push_fixed(args[2], 1i64)?;
    let val_size = hand_mem.read_fixed(args[1])?;
    if val_size == 0 {
      hand_mem.push_register(args[2], args[0])?;
    } else {
      let val = hand_mem.read_fixed(args[0])?;
      hand_mem.push_fixed(args[2], val)?;
    }
    Ok(None)
  });
  cpu!(altE => fn(args, hand_mem) {
    hand_mem.init_fractal(args[2])?;
    hand_mem.push_fixed(args[2], 0i64)?;
    let val_size = hand_mem.read_fixed(args[1])?;
    if val_size == 0 {
      hand_mem.push_register(args[2], args[0])?;
    } else {
      let val = hand_mem.read_fixed(args[0])?;
      hand_mem.push_fixed(args[2], val)?;
    }
    Ok(None)
  });
  cpu!(isMain => fn(args, hand_mem) {
    hand_mem.register_out(args[0], 0, args[2])?;
    Ok(None)
  });
  cpu!(isAlt => fn(args, hand_mem) {
    let fractal = hand_mem.read_fractal(args[0])?;
    let val = fractal.read_fixed(0)?;
    hand_mem.write_fixed(args[2], if val == 0i64 { 1i64 } else { 0i64 })?;
    Ok(None)
  });
  cpu!(mainOr => fn(args, hand_mem) {
    let fractal = hand_mem.read_fractal(args[0])?;
    let val = fractal.read_fixed(0)?;
    if val == 1i64 {
      hand_mem.register_out(args[0], 1, args[2])?;
    } else {
      let (data, is_fractal) = hand_mem.read_either(args[1])?;
      if is_fractal {
        hand_mem.register(args[2], args[1], true)?;
      } else {
        hand_mem.write_fixed(args[2], data.read_fixed(0)?)?;
      }
    }
    Ok(None)
  });
  cpu!(altOr => fn(args, hand_mem) {
    let fractal = hand_mem.read_fractal(args[0])?;
    let val = fractal.read_fixed(0)?;
    if val == 0i64 {
      hand_mem.register_out(args[0], 1, args[2])?;
    } else {
      let (data, is_fractal) = hand_mem.read_either(args[1])?;
      if is_fractal {
        hand_mem.register(args[2], args[1], true)?;
      } else {
        hand_mem.write_fixed(args[2], data.read_fixed(0)?)?;
      }
    }
    Ok(None)
  });
  cpu!(getMain => fn(args, hand_mem) {
    let fractal = hand_mem.read_fractal(args[0])?;
    let variant = fractal.read_fixed(0)?;
    if variant == 1 {
      hand_mem.register_out(args[0], 1, args[2])?;
      Ok(None)
    } else {
      Err(VMError::IllegalAccess)
    }
  });
  cpu!(getAlt => fn(args, hand_mem) {
    let fractal = hand_mem.read_fractal(args[0])?;
    let variant = fractal.read_fixed(0)?;
    if variant == 0 {
      hand_mem.register_out(args[0], 1, args[2])?;
      Ok(None)
    } else {
      Err(VMError::IllegalAccess)
    }
  });

  cpu!(hashf => fn(args, hand_mem) {
    let val = hand_mem.read_fixed(args[0])?;
    let mut hasher = XxHash64::with_seed(0xfa57);
    hasher.write_i64(val);
    let out = i64::from_ne_bytes(hasher.finish().to_ne_bytes());
    hand_mem.write_fixed(args[2], out)?;
    Ok(None)
  });

  cpu!(hashv => fn(args, hand_mem) {
    let mut hasher = XxHash64::with_seed(0xfa57);
    let addr = args[0];
    if addr < 0 {
      // It's a string!
      let pascal_string = hand_mem.read_fractal(args[0])?;
      let strlen = pascal_string.read_fixed(0)? as f64;
      let intlen = 1 + (strlen / 8.0).ceil() as usize;
      for i in 0..intlen {
        hasher.write_i64(pascal_string.read_fixed(i)?);
      }
    } else {
      let mut stack: Vec<FractalMemory> = vec![hand_mem.read_fractal(args[0])?];
      while stack.len() > 0 {
        let fractal = stack.pop().ok_or(VMError::IllegalAccess)?;
        for i in 0..fractal.len() {
          let (data, is_fractal) = hand_mem.read_from_fractal(&fractal, i);
          if is_fractal {
            stack.push(data);
          } else {
            hasher.write_i64(data.read_fixed(0)?);
          }
        }
      }
    }
    let out = i64::from_ne_bytes(hasher.finish().to_ne_bytes());
    hand_mem.write_fixed(args[2], out)?;
    Ok(None)
  });

  // king opcode
  cpu!(emit => fn(args, hand_mem) {
    let event = EventEmit {
      id: args[0],
      payload: HandlerMemory::alloc_payload(args[0], args[1], &hand_mem)?,
    };
    Ok(Some(event))
  });

  o
});

impl From<i64> for &ByteOpcode {
  fn from(v: i64) -> Self {
    let opc = OPCODES.get(&v);
    if opc.is_none() {
      panic!(
        "Illegal byte opcode {} ({})",
        v,
        str::from_utf8(&v.to_ne_bytes()).unwrap()
      );
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
