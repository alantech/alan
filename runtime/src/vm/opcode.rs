use std::collections::HashMap;
use std::convert::TryInto;
use std::future::Future;
use std::pin::Pin;
use std::process::Command;
use std::io::{self, Write};
use std::str;
use std::time::Duration;

use byteorder::{ByteOrder, LittleEndian};
use once_cell::sync::Lazy;
use tokio::time::delay_for;
use regex::Regex;

use crate::vm::event::{EventEmit, HandlerFragment};
use crate::vm::memory::HandlerMemory;

// type aliases
/// Futures implement an Unpin marker that guarantees to the compiler that the future will not move while it is running
/// so it can be polled. If it is moved, the implementation would be unsafe. We have to manually pin the future because
/// we are creating it dynamically. We must also specify that the `Box`ed Future can be moved across threads with a `+ Send`.
/// For more information see:
/// https://stackoverflow.com/questions/58354633/cannot-use-impl-future-to-store-async-function-in-a-vector
/// https://stackoverflow.com/questions/51485410/unable-to-tokiorun-a-boxed-future-because-the-trait-bound-send-is-not-satisfie
pub type EmptyFuture = Pin<Box<dyn Future<Output = ()> + Send>>;
/// Function pointer for io bound opcodes
type AsyncFnPtr = fn(
  &Vec<i64>,
  &mut HandlerMemory,
  &mut HandlerFragment
) -> EmptyFuture;
/// Function pointer for cpu bound opcodes
type FnPtr = fn(
  &Vec<i64>,
  &mut HandlerMemory,
  &mut HandlerFragment
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
  pub(crate) id: i64,
  /// Human readable name for id
  pub(crate) name: String,
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
        id,
        name: $name.to_string(),
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
        id,
        name: $name.to_string(),
        pred_exec: true,
        func: Some($func),
        async_func: None,
      };
      o.insert(id, opcode);
    };
  }

  macro_rules! unpred_cpu {
    ($name:expr, $func:expr) => {
      let id = opcode_id($name);
      let opcode = ByteOpcode {
        id,
        name: $name.to_string(),
        pred_exec: false,
        func: Some($func),
        async_func: None,
      };
      o.insert(id, opcode);
    };
  }

  // Type conversion opcodes
  cpu!("i8f64", |args, hand_mem, _| {
    let a = i8::from_le_bytes(hand_mem.read(args[0], 1).try_into().unwrap());
    let out = a as f64;
    hand_mem.write(args[2], 8, &out.to_le_bytes());
    None
  });
  cpu!("i16f64", |args, hand_mem, _| {
    let a = LittleEndian::read_i16(hand_mem.read(args[0], 2));
    let out = a as f64;
    hand_mem.write(args[2], 8, &out.to_le_bytes());
    None
  });
  cpu!("i32f64", |args, hand_mem, _| {
    let a = LittleEndian::read_i32(hand_mem.read(args[0], 4));
    let out = a as f64;
    hand_mem.write(args[2], 8, &out.to_le_bytes());
    None
  });
  cpu!("i64f64", |args, hand_mem, _| {
    let a = LittleEndian::read_i64(hand_mem.read(args[0], 8));
    let out = a as f64;
    hand_mem.write(args[2], 8, &out.to_le_bytes());
    None
  });
  cpu!("f32f64", |args, hand_mem, _| {
    let a = LittleEndian::read_f32(hand_mem.read(args[0], 4));
    let out = a as f64;
    hand_mem.write(args[2], 8, &out.to_le_bytes());
    None
  });
  cpu!("strf64", |args, hand_mem, _| {
    let pascal_string = hand_mem.read(args[0], 0);
    let size = LittleEndian::read_u64(&pascal_string[0..8]) as usize;
    let out_str = str::from_utf8(&pascal_string[8..size + 8]).unwrap();
    let out: f64 = out_str.parse().unwrap();
    hand_mem.write(args[2], 8, &out.to_le_bytes());
    None
  });
  cpu!("boolf64", |args, hand_mem, _| {
    let a = hand_mem.read(args[0], 1)[0];
    let out = a as f64; // This works because bools are 0 or 1 internally
    hand_mem.write(args[2], 8, &out.to_le_bytes());
    None
  });

  cpu!("i8f32", |args, hand_mem, _| {
    let a = i8::from_le_bytes(hand_mem.read(args[0], 1).try_into().unwrap());
    let out = a as f32;
    hand_mem.write(args[2], 4, &out.to_le_bytes());
    None
  });
  cpu!("i16f32", |args, hand_mem, _| {
    let a = LittleEndian::read_i16(hand_mem.read(args[0], 2));
    let out = a as f32;
    hand_mem.write(args[2], 4, &out.to_le_bytes());
    None
  });
  cpu!("i32f32", |args, hand_mem, _| {
    let a = LittleEndian::read_i32(hand_mem.read(args[0], 4));
    let out = a as f32;
    hand_mem.write(args[2], 4, &out.to_le_bytes());
    None
  });
  cpu!("i64f32", |args, hand_mem, _| {
    let a = LittleEndian::read_i64(hand_mem.read(args[0], 8));
    let out = a as f32;
    hand_mem.write(args[2], 4, &out.to_le_bytes());
    None
  });
  cpu!("f64f32", |args, hand_mem, _| {
    let a = LittleEndian::read_f64(hand_mem.read(args[0], 8));
    let out = a as f32;
    hand_mem.write(args[2], 4, &out.to_le_bytes());
    None
  });
  cpu!("strf32", |args, hand_mem, _| {
    let pascal_string = hand_mem.read(args[0], 0);
    let size = LittleEndian::read_u64(&pascal_string[0..8]) as usize;
    let out_str = str::from_utf8(&pascal_string[8..size + 8]).unwrap();
    let out: f32 = out_str.parse().unwrap();
    hand_mem.write(args[2], 4, &out.to_le_bytes());
    None
  });
  cpu!("boolf32", |args, hand_mem, _| {
    let a = hand_mem.read(args[0], 1)[0];
    let out = a as f32; // This works because bools are 0 or 1 internally
    hand_mem.write(args[2], 4, &out.to_le_bytes());
    None
  });

  cpu!("i8i64", |args, hand_mem, _| {
    let a = i8::from_le_bytes(hand_mem.read(args[0], 1).try_into().unwrap());
    let out = a as i64;
    hand_mem.write(args[2], 8, &out.to_le_bytes());
    None
  });
  cpu!("i16i64", |args, hand_mem, _| {
    let a = LittleEndian::read_i16(hand_mem.read(args[0], 2));
    let out = a as i64;
    hand_mem.write(args[2], 8, &out.to_le_bytes());
    None
  });
  cpu!("i32i64", |args, hand_mem, _| {
    let a = LittleEndian::read_i32(hand_mem.read(args[0], 4));
    let out = a as i64;
    hand_mem.write(args[2], 8, &out.to_le_bytes());
    None
  });
  cpu!("f64i64", |args, hand_mem, _| {
    let a = LittleEndian::read_f64(hand_mem.read(args[0], 8));
    let out = a as i64;
    hand_mem.write(args[2], 8, &out.to_le_bytes());
    None
  });
  cpu!("f32i64", |args, hand_mem, _| {
    let a = LittleEndian::read_f32(hand_mem.read(args[0], 4));
    let out = a as i64;
    hand_mem.write(args[2], 8, &out.to_le_bytes());
    None
  });
  cpu!("stri64", |args, hand_mem, _| {
    let pascal_string = hand_mem.read(args[0], 0);
    let size = LittleEndian::read_u64(&pascal_string[0..8]) as usize;
    let out_str = str::from_utf8(&pascal_string[8..size + 8]).unwrap();
    let out: i64 = out_str.parse().unwrap();
    hand_mem.write(args[2], 8, &out.to_le_bytes());
    None
  });
  cpu!("booli64", |args, hand_mem, _| {
    let a = hand_mem.read(args[0], 1)[0];
    let out = a as i64; // This works because bools are 0 or 1 internally
    hand_mem.write(args[2], 8, &out.to_le_bytes());
    None
  });

  cpu!("i8i32", |args, hand_mem, _| {
    let a = i8::from_le_bytes(hand_mem.read(args[0], 1).try_into().unwrap());
    let out = a as i32;
    hand_mem.write(args[2], 4, &out.to_le_bytes());
    None
  });
  cpu!("i16i32", |args, hand_mem, _| {
    let a = LittleEndian::read_i16(hand_mem.read(args[0], 2));
    let out = a as i32;
    hand_mem.write(args[2], 4, &out.to_le_bytes());
    None
  });
  cpu!("i64i32", |args, hand_mem, _| {
    let a = LittleEndian::read_i64(hand_mem.read(args[0], 8));
    let out = a as i32;
    hand_mem.write(args[2], 4, &out.to_le_bytes());
    None
  });
  cpu!("f64i32", |args, hand_mem, _| {
    let a = LittleEndian::read_f64(hand_mem.read(args[0], 8));
    let out = a as i32;
    hand_mem.write(args[2], 4, &out.to_le_bytes());
    None
  });
  cpu!("f32i32", |args, hand_mem, _| {
    let a = LittleEndian::read_f32(hand_mem.read(args[0], 4));
    let out = a as i32;
    hand_mem.write(args[2], 4, &out.to_le_bytes());
    None
  });
  cpu!("stri32", |args, hand_mem, _| {
    let pascal_string = hand_mem.read(args[0], 0);
    let size = LittleEndian::read_u64(&pascal_string[0..8]) as usize;
    let out_str = str::from_utf8(&pascal_string[8..size + 8]).unwrap();
    let out: i32 = out_str.parse().unwrap();
    hand_mem.write(args[2], 4, &out.to_le_bytes());
    None
  });
  cpu!("booli32", |args, hand_mem, _| {
    let a = hand_mem.read(args[0], 1)[0];
    let out = a as i32; // This works because bools are 0 or 1 internally
    hand_mem.write(args[2], 4, &out.to_le_bytes());
    None
  });

  cpu!("i8i16", |args, hand_mem, _| {
    let a = i8::from_le_bytes(hand_mem.read(args[0], 1).try_into().unwrap());
    let out = a as i16;
    hand_mem.write(args[2], 2, &out.to_le_bytes());
    None
  });
  cpu!("i32i16", |args, hand_mem, _| {
    let a = LittleEndian::read_i32(hand_mem.read(args[0], 4));
    let out = a as i16;
    hand_mem.write(args[2], 2, &out.to_le_bytes());
    None
  });
  cpu!("i64i16", |args, hand_mem, _| {
    let a = LittleEndian::read_i64(hand_mem.read(args[0], 8));
    let out = a as i16;
    hand_mem.write(args[2], 2, &out.to_le_bytes());
    None
  });
  cpu!("f64i16", |args, hand_mem, _| {
    let a = LittleEndian::read_f64(hand_mem.read(args[0], 8));
    let out = a as i16;
    hand_mem.write(args[2], 2, &out.to_le_bytes());
    None
  });
  cpu!("f32i16", |args, hand_mem, _| {
    let a = LittleEndian::read_f32(hand_mem.read(args[0], 4));
    let out = a as i16;
    hand_mem.write(args[2], 2, &out.to_le_bytes());
    None
  });
  cpu!("stri16", |args, hand_mem, _| {
    let pascal_string = hand_mem.read(args[0], 0);
    let size = LittleEndian::read_u64(&pascal_string[0..8]) as usize;
    let out_str = str::from_utf8(&pascal_string[8..size + 8]).unwrap();
    let out: i16 = out_str.parse().unwrap();
    hand_mem.write(args[2], 2, &out.to_le_bytes());
    None
  });
  cpu!("booli16", |args, hand_mem, _| {
    let a = hand_mem.read(args[0], 1)[0];
    let out = a as i16; // This works because bools are 0 or 1 internally
    hand_mem.write(args[2], 2, &out.to_le_bytes());
    None
  });

  cpu!("i16i8", |args, hand_mem, _| {
    let a = LittleEndian::read_i16(hand_mem.read(args[0], 2));
    let out = a as i8;
    hand_mem.write(args[2], 1, &out.to_le_bytes());
    None
  });
  cpu!("i32i8", |args, hand_mem, _| {
    let a = LittleEndian::read_i32(hand_mem.read(args[0], 4));
    let out = a as i8;
    hand_mem.write(args[2], 1, &out.to_le_bytes());
    None
  });
  cpu!("i64i8", |args, hand_mem, _| {
    let a = LittleEndian::read_i64(hand_mem.read(args[0], 8));
    let out = a as i8;
    hand_mem.write(args[2], 1, &out.to_le_bytes());
    None
  });
  cpu!("f64i8", |args, hand_mem, _| {
    let a = LittleEndian::read_f64(hand_mem.read(args[0], 8));
    let out = a as i8;
    hand_mem.write(args[2], 1, &out.to_le_bytes());
    None
  });
  cpu!("f32i8", |args, hand_mem, _| {
    let a = LittleEndian::read_f32(hand_mem.read(args[0], 4));
    let out = a as i8;
    hand_mem.write(args[2], 1, &out.to_le_bytes());
    None
  });
  cpu!("stri8", |args, hand_mem, _| {
    let pascal_string = hand_mem.read(args[0], 0);
    let size = LittleEndian::read_u64(&pascal_string[0..8]) as usize;
    let out_str = str::from_utf8(&pascal_string[8..size + 8]).unwrap();
    let out: i8 = out_str.parse().unwrap();
    hand_mem.write(args[2], 1, &out.to_le_bytes());
    None
  });
  cpu!("booli8", |args, hand_mem, _| {
    let a = hand_mem.read(args[0], 1)[0];
    let out = a as i8; // This works because bools are 0 or 1 internally
    hand_mem.write(args[2], 1, &out.to_le_bytes());
    None
  });

  cpu!("i8bool", |args, hand_mem, _| {
    let a = i8::from_le_bytes(hand_mem.read(args[0], 1).try_into().unwrap());
    let out = if a != 0 { 1u8 } else { 0u8 };
    hand_mem.write(args[2], 1, &out.to_le_bytes());
    None
  });
  cpu!("i16bool", |args, hand_mem, _| {
    let a = LittleEndian::read_i16(hand_mem.read(args[0], 2));
    let out = if a != 0 { 1u8 } else { 0u8 };
    hand_mem.write(args[2], 1, &out.to_le_bytes());
    None
  });
  cpu!("i32bool", |args, hand_mem, _| {
    let a = LittleEndian::read_i32(hand_mem.read(args[0], 4));
    let out = if a != 0 { 1u8 } else { 0u8 };
    hand_mem.write(args[2], 1, &out.to_le_bytes());
    None
  });
  cpu!("i64bool", |args, hand_mem, _| {
    let a = LittleEndian::read_i64(hand_mem.read(args[0], 8));
    let out = if a != 0 { 1u8 } else { 0u8 };
    hand_mem.write(args[2], 1, &out.to_le_bytes());
    None
  });
  cpu!("f64bool", |args, hand_mem, _| {
    let a = LittleEndian::read_f64(hand_mem.read(args[0], 8));
    let out = if a != 0.0 { 1u8 } else { 0u8 };
    hand_mem.write(args[2], 1, &out.to_le_bytes());
    None
  });
  cpu!("f32bool", |args, hand_mem, _| {
    let a = LittleEndian::read_f32(hand_mem.read(args[0], 4));
    let out = if a != 0.0 { 1u8 } else { 0u8 };
    hand_mem.write(args[2], 1, &out.to_le_bytes());
    None
  });
  cpu!("strbool", |args, hand_mem, _| {
    let pascal_string = hand_mem.read(args[0], 0);
    let size = LittleEndian::read_u64(&pascal_string[0..8]) as usize;
    let a_str = str::from_utf8(&pascal_string[8..size + 8]).unwrap();
    let out = if a_str == "true" { 1u8 } else { 0u8 };
    hand_mem.write(args[2], 1, &out.to_le_bytes());
    None
  });

  cpu!("i8str", |args, hand_mem, _| {
    let a = i8::from_le_bytes(hand_mem.read(args[0], 1).try_into().unwrap());
    let a_str = a.to_string();
    let mut out = (a_str.len() as u64).to_le_bytes().to_vec();
    out.append(&mut a_str.as_bytes().to_vec());
    hand_mem.write(args[2], 0, &out);
    None
  });
  cpu!("i16str", |args, hand_mem, _| {
    let a = LittleEndian::read_i16(hand_mem.read(args[0], 2));
    let a_str = a.to_string();
    let mut out = (a_str.len() as u64).to_le_bytes().to_vec();
    out.append(&mut a_str.as_bytes().to_vec());
    hand_mem.write(args[2], 0, &out);
    None
  });
  cpu!("i32str", |args, hand_mem, _| {
    let a = LittleEndian::read_i32(hand_mem.read(args[0], 4));
    let a_str = a.to_string();
    let mut out = (a_str.len() as u64).to_le_bytes().to_vec();
    out.append(&mut a_str.as_bytes().to_vec());
    hand_mem.write(args[2], 0, &out);
    None
  });
  cpu!("i64str", |args, hand_mem, _| {
    let a = LittleEndian::read_i64(hand_mem.read(args[0], 8));
    let a_str = a.to_string();
    let mut out = (a_str.len() as u64).to_le_bytes().to_vec();
    out.append(&mut a_str.as_bytes().to_vec());
    hand_mem.write(args[2], 0, &out);
    None
  });
  cpu!("f64str", |args, hand_mem, _| {
    let a = LittleEndian::read_f64(hand_mem.read(args[0], 8));
    let a_str = a.to_string();
    let mut out = (a_str.len() as u64).to_le_bytes().to_vec();
    out.append(&mut a_str.as_bytes().to_vec());
    hand_mem.write(args[2], 0, &out);
    None
  });
  cpu!("f32str", |args, hand_mem, _| {
    let a = LittleEndian::read_f32(hand_mem.read(args[0], 4));
    let a_str = a.to_string();
    let mut out = (a_str.len() as u64).to_le_bytes().to_vec();
    out.append(&mut a_str.as_bytes().to_vec());
    hand_mem.write(args[2], 0, &out);
    None
  });
  cpu!("boolstr", |args, hand_mem, _| {
    let a = hand_mem.read(args[0], 1)[0];
    let a_str = if a == 1 { "true" } else { "false" };
    let mut out = (a_str.len() as u64).to_le_bytes().to_vec();
    out.append(&mut a_str.as_bytes().to_vec());
    hand_mem.write(args[2], 0, &out);
    None
  });

  // Arithmetic opcodes
  cpu!("addi8", |args, hand_mem, _| {
    let a = i8::from_le_bytes(hand_mem.read(args[0], 1).try_into().unwrap());
    let b = i8::from_le_bytes(hand_mem.read(args[1], 1).try_into().unwrap());
    let out = a + b;
    hand_mem.write(args[2], 1, &out.to_le_bytes());
    None
  });
  cpu!("addi16", |args, hand_mem, _| {
    let a = LittleEndian::read_i16(hand_mem.read(args[0], 2));
    let b = LittleEndian::read_i16(hand_mem.read(args[1], 2));
    let out = a + b;
    hand_mem.write(args[2], 2, &out.to_le_bytes());
    None
  });
  cpu!("addi32", |args, hand_mem, _| {
    let a = LittleEndian::read_i32(hand_mem.read(args[0], 4));
    let b = LittleEndian::read_i32(hand_mem.read(args[1], 4));
    let out = a + b;
    hand_mem.write(args[2], 4, &out.to_le_bytes());
    None
  });
  cpu!("addi64", |args, hand_mem, _| {
    let a = LittleEndian::read_i64(hand_mem.read(args[0], 8));
    let b = LittleEndian::read_i64(hand_mem.read(args[1], 8));
    let out = a + b;
    hand_mem.write(args[2], 8, &out.to_le_bytes());
    None
  });
  cpu!("addf32", |args, hand_mem, _| {
    let a = LittleEndian::read_f32(hand_mem.read(args[0], 4));
    let b = LittleEndian::read_f32(hand_mem.read(args[1], 4));
    let out = a + b;
    hand_mem.write(args[2], 4, &out.to_le_bytes());
    None
  });
  cpu!("addf64", |args, hand_mem, _| {
    let a = LittleEndian::read_f64(hand_mem.read(args[0], 8));
    let b = LittleEndian::read_f64(hand_mem.read(args[1], 8));
    let out = a + b;
    hand_mem.write(args[2], 8, &out.to_le_bytes());
    None
  });

  cpu!("subi8", |args, hand_mem, _| {
    let a = i8::from_le_bytes(hand_mem.read(args[0], 1).try_into().unwrap());
    let b = i8::from_le_bytes(hand_mem.read(args[1], 1).try_into().unwrap());
    let out = a - b;
    hand_mem.write(args[2], 1, &out.to_le_bytes());
    None
  });
  cpu!("subi16", |args, hand_mem, _| {
    let a = LittleEndian::read_i16(hand_mem.read(args[0], 2));
    let b = LittleEndian::read_i16(hand_mem.read(args[1], 2));
    let out = a - b;
    hand_mem.write(args[2], 2, &out.to_le_bytes());
    None
  });
  cpu!("subi32", |args, hand_mem, _| {
    let a = LittleEndian::read_i32(hand_mem.read(args[0], 4));
    let b = LittleEndian::read_i32(hand_mem.read(args[1], 4));
    let out = a - b;
    hand_mem.write(args[2], 4, &out.to_le_bytes());
    None
  });
  cpu!("subi64", |args, hand_mem, _| {
    let a = LittleEndian::read_i64(hand_mem.read(args[0], 8));
    let b = LittleEndian::read_i64(hand_mem.read(args[1], 8));
    let out = a - b;
    hand_mem.write(args[2], 8, &out.to_le_bytes());
    None
  });
  cpu!("subf32", |args, hand_mem, _| {
    let a = LittleEndian::read_f32(hand_mem.read(args[0], 4));
    let b = LittleEndian::read_f32(hand_mem.read(args[1], 4));
    let out = a - b;
    hand_mem.write(args[2], 4, &out.to_le_bytes());
    None
  });
  cpu!("subf64", |args, hand_mem, _| {
    let a = LittleEndian::read_f64(hand_mem.read(args[0], 8));
    let b = LittleEndian::read_f64(hand_mem.read(args[1], 8));
    let out = a - b;
    hand_mem.write(args[2], 8, &out.to_le_bytes());
    None
  });

  cpu!("negi8", |args, hand_mem, _| {
    let a = i8::from_le_bytes(hand_mem.read(args[0], 1).try_into().unwrap());
    let out = 0 - a;
    hand_mem.write(args[2], 1, &out.to_le_bytes());
    None
  });
  cpu!("negi16", |args, hand_mem, _| {
    let a = LittleEndian::read_i16(hand_mem.read(args[0], 2));
    let out = 0 - a;
    hand_mem.write(args[2], 2, &out.to_le_bytes());
    None
  });
  cpu!("negi32", |args, hand_mem, _| {
    let a = LittleEndian::read_i32(hand_mem.read(args[0], 4));
    let out = 0 - a;
    hand_mem.write(args[2], 4, &out.to_le_bytes());
    None
  });
  cpu!("negi64", |args, hand_mem, _| {
    let a = LittleEndian::read_i64(hand_mem.read(args[0], 8));
    let out = 0 - a;
    hand_mem.write(args[2], 8, &out.to_le_bytes());
    None
  });
  cpu!("negf32", |args, hand_mem, _| {
    let a = LittleEndian::read_f32(hand_mem.read(args[0], 4));
    let out = 0.0 - a;
    hand_mem.write(args[2], 4, &out.to_le_bytes());
    None
  });
  cpu!("negf64", |args, hand_mem, _| {
    let a = LittleEndian::read_f64(hand_mem.read(args[0], 8));
    let out = 0.0 - a;
    hand_mem.write(args[2], 8, &out.to_le_bytes());
    None
  });

  cpu!("muli8", |args, hand_mem, _| {
    let a = i8::from_le_bytes(hand_mem.read(args[0], 1).try_into().unwrap());
    let b = i8::from_le_bytes(hand_mem.read(args[1], 1).try_into().unwrap());
    let out = a * b;
    hand_mem.write(args[2], 1, &out.to_le_bytes());
    None
  });
  cpu!("muli16", |args, hand_mem, _| {
    let a = LittleEndian::read_i16(hand_mem.read(args[0], 2));
    let b = LittleEndian::read_i16(hand_mem.read(args[1], 2));
    let out = a * b;
    hand_mem.write(args[2], 2, &out.to_le_bytes());
    None
  });
  cpu!("muli32", |args, hand_mem, _| {
    let a = LittleEndian::read_i32(hand_mem.read(args[0], 4));
    let b = LittleEndian::read_i32(hand_mem.read(args[1], 4));
    let out = a * b;
    hand_mem.write(args[2], 4, &out.to_le_bytes());
    None
  });
  cpu!("muli64", |args, hand_mem, _| {
    let a = LittleEndian::read_i64(hand_mem.read(args[0], 8));
    let b = LittleEndian::read_i64(hand_mem.read(args[1], 8));
    let out = a * b;
    hand_mem.write(args[2], 8, &out.to_le_bytes());
    None
  });
  cpu!("mulf32", |args, hand_mem, _| {
    let a = LittleEndian::read_f32(hand_mem.read(args[0], 4));
    let b = LittleEndian::read_f32(hand_mem.read(args[1], 4));
    let out = a * b;
    hand_mem.write(args[2], 4, &out.to_le_bytes());
    None
  });
  cpu!("mulf64", |args, hand_mem, _| {
    let a = LittleEndian::read_f64(hand_mem.read(args[0], 8));
    let b = LittleEndian::read_f64(hand_mem.read(args[1], 8));
    let out = a * b;
    hand_mem.write(args[2], 8, &out.to_le_bytes());
    None
  });

  cpu!("divi8", |args, hand_mem, _| {
    let a = i8::from_le_bytes(hand_mem.read(args[0], 1).try_into().unwrap());
    let b = i8::from_le_bytes(hand_mem.read(args[1], 1).try_into().unwrap());
    let out = a / b;
    hand_mem.write(args[2], 1, &out.to_le_bytes());
    None
  });
  cpu!("divi16", |args, hand_mem, _| {
    let a = LittleEndian::read_i16(hand_mem.read(args[0], 2));
    let b = LittleEndian::read_i16(hand_mem.read(args[1], 2));
    let out = a / b;
    hand_mem.write(args[2], 2, &out.to_le_bytes());
    None
  });
  cpu!("divi32", |args, hand_mem, _| {
    let a = LittleEndian::read_i32(hand_mem.read(args[0], 4));
    let b = LittleEndian::read_i32(hand_mem.read(args[1], 4));
    let out = a / b;
    hand_mem.write(args[2], 4, &out.to_le_bytes());
    None
  });
  cpu!("divi64", |args, hand_mem, _| {
    let a = LittleEndian::read_i64(hand_mem.read(args[0], 8));
    let b = LittleEndian::read_i64(hand_mem.read(args[1], 8));
    let out = a / b;
    hand_mem.write(args[2], 8, &out.to_le_bytes());
    None
  });
  cpu!("divf32", |args, hand_mem, _| {
    let a = LittleEndian::read_f32(hand_mem.read(args[0], 4));
    let b = LittleEndian::read_f32(hand_mem.read(args[1], 4));
    let out = a / b;
    hand_mem.write(args[2], 4, &out.to_le_bytes());
    None
  });
  cpu!("divf64", |args, hand_mem, _| {
    let a = LittleEndian::read_f64(hand_mem.read(args[0], 8));
    let b = LittleEndian::read_f64(hand_mem.read(args[1], 8));
    let out = a / b;
    hand_mem.write(args[2], 8, &out.to_le_bytes());
    None
  });

  cpu!("modi8", |args, hand_mem, _| {
    let a = i8::from_le_bytes(hand_mem.read(args[0], 1).try_into().unwrap());
    let b = i8::from_le_bytes(hand_mem.read(args[1], 1).try_into().unwrap());
    let out = a % b;
    hand_mem.write(args[2], 1, &out.to_le_bytes());
    None
  });
  cpu!("modi16", |args, hand_mem, _| {
    let a = LittleEndian::read_i16(hand_mem.read(args[0], 2));
    let b = LittleEndian::read_i16(hand_mem.read(args[1], 2));
    let out = a % b;
    hand_mem.write(args[2], 2, &out.to_le_bytes());
    None
  });
  cpu!("modi32", |args, hand_mem, _| {
    let a = LittleEndian::read_i32(hand_mem.read(args[0], 4));
    let b = LittleEndian::read_i32(hand_mem.read(args[1], 4));
    let out = a % b;
    hand_mem.write(args[2], 4, &out.to_le_bytes());
    None
  });
  cpu!("modi64", |args, hand_mem, _| {
    let a = LittleEndian::read_i64(hand_mem.read(args[0], 8));
    let b = LittleEndian::read_i64(hand_mem.read(args[1], 8));
    let out = a % b;
    hand_mem.write(args[2], 8, &out.to_le_bytes());
    None
  });

  cpu!("powi8", |args, hand_mem, _| {
    let a = i8::from_le_bytes(hand_mem.read(args[0], 1).try_into().unwrap());
    let b = i8::from_le_bytes(hand_mem.read(args[1], 1).try_into().unwrap());
    let out = if b < 0 { 0i8 } else { i8::pow(a, b as u32) };
    hand_mem.write(args[2], 1, &out.to_le_bytes());
    None
  });
  cpu!("powi16", |args, hand_mem, _| {
    let a = LittleEndian::read_i16(hand_mem.read(args[0], 2));
    let b = LittleEndian::read_i16(hand_mem.read(args[1], 2));
    let out = if b < 0 { 0i16 } else { i16::pow(a, b as u32) };
    hand_mem.write(args[2], 2, &out.to_le_bytes());
    None
  });
  cpu!("powi32", |args, hand_mem, _| {
    let a = LittleEndian::read_i32(hand_mem.read(args[0], 4));
    let b = LittleEndian::read_i32(hand_mem.read(args[1], 4));
    let out = if b < 0 { 0i32 } else { i32::pow(a, b as u32) };
    hand_mem.write(args[2], 4, &out.to_le_bytes());
    None
  });
  cpu!("powi64", |args, hand_mem, _| {
    // The inputs may be from local memory or global
    let a = LittleEndian::read_i64(hand_mem.read(args[0], 8));
    let b = LittleEndian::read_i64(hand_mem.read(args[1], 8));
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
    hand_mem.write(args[2], 8, &out.to_le_bytes());
    None
  });
  cpu!("powf32", |args, hand_mem, _| {
    let a = LittleEndian::read_f32(hand_mem.read(args[0], 4));
    let b = LittleEndian::read_f32(hand_mem.read(args[1], 4));
    let out = f32::powf(a, b);
    hand_mem.write(args[2], 4, &out.to_le_bytes());
    None
  });
  cpu!("powf64", |args, hand_mem, _| {
    let a = LittleEndian::read_f64(hand_mem.read(args[0], 8));
    let b = LittleEndian::read_f64(hand_mem.read(args[1], 8));
    let out = f64::powf(a, b);
    hand_mem.write(args[2], 8, &out.to_le_bytes());
    None
  });

  cpu!("sqrtf32", |args, hand_mem, _| {
    let a = LittleEndian::read_f32(hand_mem.read(args[0], 4));
    let out = f32::sqrt(a);
    hand_mem.write(args[2], 4, &out.to_le_bytes());
    None
  });
  cpu!("sqrtf64", |args, hand_mem, _| {
    let a = LittleEndian::read_f64(hand_mem.read(args[0], 8));
    let out = f64::sqrt(a);
    hand_mem.write(args[2], 8, &out.to_le_bytes());
    None
  });

  // Boolean and bitwise opcodes
  cpu!("andi8", |args, hand_mem, _| {
    let a = i8::from_le_bytes(hand_mem.read(args[0], 1).try_into().unwrap());
    let b = i8::from_le_bytes(hand_mem.read(args[1], 1).try_into().unwrap());
    let out = a & b;
    hand_mem.write(args[2], 1, &out.to_le_bytes());
    None
  });
  cpu!("andi16", |args, hand_mem, _| {
    let a = LittleEndian::read_i16(hand_mem.read(args[0], 2));
    let b = LittleEndian::read_i16(hand_mem.read(args[1], 2));
    let out = a & b;
    hand_mem.write(args[2], 2, &out.to_le_bytes());
    None
  });
  cpu!("andi32", |args, hand_mem, _| {
    let a = LittleEndian::read_i32(hand_mem.read(args[0], 4));
    let b = LittleEndian::read_i32(hand_mem.read(args[1], 4));
    let out = a & b;
    hand_mem.write(args[2], 4, &out.to_le_bytes());
    None
  });
  cpu!("andi64", |args, hand_mem, _| {
    let a = LittleEndian::read_i64(hand_mem.read(args[0], 8));
    let b = LittleEndian::read_i64(hand_mem.read(args[1], 8));
    let out = a & b;
    hand_mem.write(args[2], 8, &out.to_le_bytes());
    None
  });
  cpu!("andbool", |args, hand_mem, _| {
    let a = hand_mem.read(args[0], 1)[0];
    let b = hand_mem.read(args[1], 1)[0];
    let a_bool = if a == 1u8 { true } else { false };
    let b_bool = if b == 1u8 { true } else { false };
    let out = if a_bool & b_bool { 1u8 } else { 0u8 };
    hand_mem.write(args[2], 1, &out.to_le_bytes());
    None
  });

  cpu!("ori8", |args, hand_mem, _| {
    let a = i8::from_le_bytes(hand_mem.read(args[0], 1).try_into().unwrap());
    let b = i8::from_le_bytes(hand_mem.read(args[1], 1).try_into().unwrap());
    let out = a | b;
    hand_mem.write(args[2], 1, &out.to_le_bytes());
    None
  });
  cpu!("ori16", |args, hand_mem, _| {
    let a = LittleEndian::read_i16(hand_mem.read(args[0], 2));
    let b = LittleEndian::read_i16(hand_mem.read(args[1], 2));
    let out = a | b;
    hand_mem.write(args[2], 2, &out.to_le_bytes());
    None
  });
  cpu!("ori32", |args, hand_mem, _| {
    let a = LittleEndian::read_i32(hand_mem.read(args[0], 4));
    let b = LittleEndian::read_i32(hand_mem.read(args[1], 4));
    let out = a | b;
    hand_mem.write(args[2], 4, &out.to_le_bytes());
    None
  });
  cpu!("ori64", |args, hand_mem, _| {
    let a = LittleEndian::read_i64(hand_mem.read(args[0], 8));
    let b = LittleEndian::read_i64(hand_mem.read(args[1], 8));
    let out = a | b;
    hand_mem.write(args[2], 8, &out.to_le_bytes());
    None
  });
  cpu!("orbool", |args, hand_mem, _| {
    let a = hand_mem.read(args[0], 1)[0];
    let b = hand_mem.read(args[1], 1)[0];
    let a_bool = if a == 1u8 { true } else { false };
    let b_bool = if b == 1u8 { true } else { false };
    let out = if a_bool | b_bool { 1u8 } else { 0u8 };
    hand_mem.write(args[2], 1, &out.to_le_bytes());
    None
  });

  cpu!("xori8", |args, hand_mem, _| {
    let a = i8::from_le_bytes(hand_mem.read(args[0], 1).try_into().unwrap());
    let b = i8::from_le_bytes(hand_mem.read(args[1], 1).try_into().unwrap());
    let out = a ^ b;
    hand_mem.write(args[2], 1, &out.to_le_bytes());
    None
  });
  cpu!("xori16", |args, hand_mem, _| {
    let a = LittleEndian::read_i16(hand_mem.read(args[0], 2));
    let b = LittleEndian::read_i16(hand_mem.read(args[1], 2));
    let out = a ^ b;
    hand_mem.write(args[2], 2, &out.to_le_bytes());
    None
  });
  cpu!("xori32", |args, hand_mem, _| {
    let a = LittleEndian::read_i32(hand_mem.read(args[0], 4));
    let b = LittleEndian::read_i32(hand_mem.read(args[1], 4));
    let out = a ^ b;
    hand_mem.write(args[2], 4, &out.to_le_bytes());
    None
  });
  cpu!("xori64", |args, hand_mem, _| {
    let a = LittleEndian::read_i64(hand_mem.read(args[0], 8));
    let b = LittleEndian::read_i64(hand_mem.read(args[1], 8));
    let out = a ^ b;
    hand_mem.write(args[2], 8, &out.to_le_bytes());
    None
  });
  cpu!("xorbool", |args, hand_mem, _| {
    let a = hand_mem.read(args[0], 1)[0];
    let b = hand_mem.read(args[1], 1)[0];
    let a_bool = if a == 1u8 { true } else { false };
    let b_bool = if b == 1u8 { true } else { false };
    let out = if a_bool ^ b_bool { 1u8 } else { 0u8 };
    hand_mem.write(args[2], 1, &out.to_le_bytes());
    None
  });

  cpu!("noti8", |args, hand_mem, _| {
    let a = i8::from_le_bytes(hand_mem.read(args[0], 1).try_into().unwrap());
    let out = !a;
    hand_mem.write(args[2], 1, &out.to_le_bytes());
    None
  });
  cpu!("noti16", |args, hand_mem, _| {
    let a = LittleEndian::read_i16(hand_mem.read(args[0], 2));
    let out = !a;
    hand_mem.write(args[2], 2, &out.to_le_bytes());
    None
  });
  cpu!("noti32", |args, hand_mem, _| {
    let a = LittleEndian::read_i32(hand_mem.read(args[0], 4));
    let out = !a;
    hand_mem.write(args[2], 4, &out.to_le_bytes());
    None
  });
  cpu!("noti64", |args, hand_mem, _| {
    let a = LittleEndian::read_i64(hand_mem.read(args[0], 8));
    let out = !a;
    hand_mem.write(args[2], 8, &out.to_le_bytes());
    None
  });
  cpu!("notbool", |args, hand_mem, _| {
    let a = hand_mem.read(args[0], 1)[0];
    let out = if a == 0u8 { 1u8 } else { 0u8 };
    hand_mem.write(args[2], 1, &out.to_le_bytes());
    None
  });

  cpu!("nandi8", |args, hand_mem, _| {
    let a = i8::from_le_bytes(hand_mem.read(args[0], 1).try_into().unwrap());
    let b = i8::from_le_bytes(hand_mem.read(args[1], 1).try_into().unwrap());
    let out = !(a & b);
    hand_mem.write(args[2], 1, &out.to_le_bytes());
    None
  });
  cpu!("nandi16", |args, hand_mem, _| {
    let a = LittleEndian::read_i16(hand_mem.read(args[0], 2));
    let b = LittleEndian::read_i16(hand_mem.read(args[1], 2));
    let out = !(a & b);
    hand_mem.write(args[2], 2, &out.to_le_bytes());
    None
  });
  cpu!("nandi32", |args, hand_mem, _| {
    let a = LittleEndian::read_i32(hand_mem.read(args[0], 4));
    let b = LittleEndian::read_i32(hand_mem.read(args[1], 4));
    let out = !(a & b);
    hand_mem.write(args[2], 4, &out.to_le_bytes());
    None
  });
  cpu!("nandi64", |args, hand_mem, _| {
    let a = LittleEndian::read_i64(hand_mem.read(args[0], 8));
    let b = LittleEndian::read_i64(hand_mem.read(args[1], 8));
    let out = !(a & b);
    hand_mem.write(args[2], 8, &out.to_le_bytes());
    None
  });
  cpu!("nandboo", |args, hand_mem, _| {
    let a = hand_mem.read(args[0], 1)[0];
    let b = hand_mem.read(args[1], 1)[0];
    let a_bool = if a == 1u8 { true } else { false };
    let b_bool = if b == 1u8 { true } else { false };
    let out = if !(a_bool & b_bool) { 1u8 } else { 0u8 };
    hand_mem.write(args[2], 1, &out.to_le_bytes());
    None
  });

  cpu!("nori8", |args, hand_mem, _| {
    let a = i8::from_le_bytes(hand_mem.read(args[0], 1).try_into().unwrap());
    let b = i8::from_le_bytes(hand_mem.read(args[1], 1).try_into().unwrap());
    let out = !(a | b);
    hand_mem.write(args[2], 1, &out.to_le_bytes());
    None
  });
  cpu!("nori16", |args, hand_mem, _| {
    let a = LittleEndian::read_i16(hand_mem.read(args[0], 2));
    let b = LittleEndian::read_i16(hand_mem.read(args[1], 2));
    let out = !(a | b);
    hand_mem.write(args[2], 2, &out.to_le_bytes());
    None
  });
  cpu!("nori32", |args, hand_mem, _| {
    let a = LittleEndian::read_i32(hand_mem.read(args[0], 4));
    let b = LittleEndian::read_i32(hand_mem.read(args[1], 4));
    let out = !(a | b);
    hand_mem.write(args[2], 4, &out.to_le_bytes());
    None
  });
  cpu!("nori64", |args, hand_mem, _| {
    let a = LittleEndian::read_i64(hand_mem.read(args[0], 8));
    let b = LittleEndian::read_i64(hand_mem.read(args[1], 8));
    let out = !(a | b);
    hand_mem.write(args[2], 8, &out.to_le_bytes());
    None
  });
  cpu!("norbool", |args, hand_mem, _| {
    let a = hand_mem.read(args[0], 1)[0];
    let b = hand_mem.read(args[1], 1)[0];
    let a_bool = if a == 1u8 { true } else { false };
    let b_bool = if b == 1u8 { true } else { false };
    let out = if !(a_bool | b_bool) { 1u8 } else { 0u8 };
    hand_mem.write(args[2], 1, &out.to_le_bytes());
    None
  });

  cpu!("xnori8", |args, hand_mem, _| {
    let a = i8::from_le_bytes(hand_mem.read(args[0], 1).try_into().unwrap());
    let b = i8::from_le_bytes(hand_mem.read(args[1], 1).try_into().unwrap());
    let out = !(a ^ b);
    hand_mem.write(args[2], 1, &out.to_le_bytes());
    None
  });
  cpu!("xnori16", |args, hand_mem, _| {
    let a = LittleEndian::read_i16(hand_mem.read(args[0], 2));
    let b = LittleEndian::read_i16(hand_mem.read(args[1], 2));
    let out = !(a ^ b);
    hand_mem.write(args[2], 2, &out.to_le_bytes());
    None
  });
  cpu!("xnori32", |args, hand_mem, _| {
    let a = LittleEndian::read_i32(hand_mem.read(args[0], 4));
    let b = LittleEndian::read_i32(hand_mem.read(args[1], 4));
    let out = !(a ^ b);
    hand_mem.write(args[2], 4, &out.to_le_bytes());
    None
  });
  cpu!("xnori64", |args, hand_mem, _| {
    let a = LittleEndian::read_i64(hand_mem.read(args[0], 8));
    let b = LittleEndian::read_i64(hand_mem.read(args[1], 8));
    let out = !(a ^ b);
    hand_mem.write(args[2], 8, &out.to_le_bytes());
    None
  });
  cpu!("xnorboo", |args, hand_mem, _| {
    let a = hand_mem.read(args[0], 1)[0];
    let b = hand_mem.read(args[1], 1)[0];
    let a_bool = if a == 1u8 { true } else { false };
    let b_bool = if b == 1u8 { true } else { false };
    let out = if !(a_bool ^ b_bool) { 1u8 } else { 0u8 };
    hand_mem.write(args[2], 1, &out.to_le_bytes());
    None
  });

  // Equality and order opcodes
  cpu!("eqi8", |args, hand_mem, _| {
    let a = i8::from_le_bytes(hand_mem.read(args[0], 1).try_into().unwrap());
    let b = i8::from_le_bytes(hand_mem.read(args[1], 1).try_into().unwrap());
    let out = if a == b { 1u8 } else { 0u8 };
    hand_mem.write(args[2], 1, &out.to_le_bytes());
    None
  });
  cpu!("eqi16", |args, hand_mem, _| {
    let a = LittleEndian::read_i16(hand_mem.read(args[0], 2));
    let b = LittleEndian::read_i16(hand_mem.read(args[1], 2));
    let out = if a == b { 1u8 } else { 0u8 };
    hand_mem.write(args[2], 1, &out.to_le_bytes());
    None
  });
  cpu!("eqi32", |args, hand_mem, _| {
    let a = LittleEndian::read_i32(hand_mem.read(args[0], 4));
    let b = LittleEndian::read_i32(hand_mem.read(args[1], 4));
    let out = if a == b { 1u8 } else { 0u8 };
    hand_mem.write(args[2], 1, &out.to_le_bytes());
    None
  });
  cpu!("eqi64", |args, hand_mem, _| {
    let a = LittleEndian::read_i64(hand_mem.read(args[0], 8));
    let b = LittleEndian::read_i64(hand_mem.read(args[1], 8));
    let out = if a == b { 1u8 } else { 0u8 };
    hand_mem.write(args[2], 1, &out.to_le_bytes());
    None
  });
  cpu!("eqf32", |args, hand_mem, _| {
    let a = LittleEndian::read_f32(hand_mem.read(args[0], 4));
    let b = LittleEndian::read_f32(hand_mem.read(args[1], 4));
    let out = if a == b { 1u8 } else { 0u8 };
    hand_mem.write(args[2], 1, &out.to_le_bytes());
    None
  });
  cpu!("eqf64", |args, hand_mem, _| {
    let a = LittleEndian::read_f64(hand_mem.read(args[0], 8));
    let b = LittleEndian::read_f64(hand_mem.read(args[1], 8));
    let out = if a == b { 1u8 } else { 0u8 };
    hand_mem.write(args[2], 1, &out.to_le_bytes());
    None
  });
  cpu!("eqstr", |args, hand_mem, _| {
    let a_pascal_string = hand_mem.read(args[0], 0);
    let a_size = LittleEndian::read_u64(&a_pascal_string[0..8]) as usize;
    let a_str = str::from_utf8(&a_pascal_string[8..a_size + 8]).unwrap();
    let b_pascal_string = hand_mem.read(args[1], 0);
    let b_size = LittleEndian::read_u64(&b_pascal_string[0..8]) as usize;
    let b_str = str::from_utf8(&b_pascal_string[8..b_size + 8]).unwrap();
    let out = if a_pascal_string == b_pascal_string { 1u8 } else { 0u8 };
    hand_mem.write(args[2], 1, &out.to_le_bytes());
    None
  });
  cpu!("eqbool", |args, hand_mem, _| {
    let a = hand_mem.read(args[0], 1)[0];
    let b = hand_mem.read(args[1], 1)[0];
    let out = if a == b { 1u8 } else { 0u8 };
    hand_mem.write(args[2], 1, &out.to_le_bytes());
    None
  });

  cpu!("neqi8", |args, hand_mem, _| {
    let a = i8::from_le_bytes(hand_mem.read(args[0], 1).try_into().unwrap());
    let b = i8::from_le_bytes(hand_mem.read(args[1], 1).try_into().unwrap());
    let out = if a != b { 1u8 } else { 0u8 };
    hand_mem.write(args[2], 1, &out.to_le_bytes());
    None
  });
  cpu!("neqi16", |args, hand_mem, _| {
    let a = LittleEndian::read_i16(hand_mem.read(args[0], 2));
    let b = LittleEndian::read_i16(hand_mem.read(args[1], 2));
    let out = if a != b { 1u8 } else { 0u8 };
    hand_mem.write(args[2], 1, &out.to_le_bytes());
    None
  });
  cpu!("neqi32", |args, hand_mem, _| {
    let a = LittleEndian::read_i32(hand_mem.read(args[0], 4));
    let b = LittleEndian::read_i32(hand_mem.read(args[1], 4));
    let out = if a != b { 1u8 } else { 0u8 };
    hand_mem.write(args[2], 1, &out.to_le_bytes());
    None
  });
  cpu!("neqi64", |args, hand_mem, _| {
    let a = LittleEndian::read_i64(hand_mem.read(args[0], 8));
    let b = LittleEndian::read_i64(hand_mem.read(args[1], 8));
    let out = if a != b { 1u8 } else { 0u8 };
    hand_mem.write(args[2], 1, &out.to_le_bytes());
    None
  });
  cpu!("neqf32", |args, hand_mem, _| {
    let a = LittleEndian::read_f32(hand_mem.read(args[0], 4));
    let b = LittleEndian::read_f32(hand_mem.read(args[1], 4));
    let out = if a != b { 1u8 } else { 0u8 };
    hand_mem.write(args[2], 1, &out.to_le_bytes());
    None
  });
  cpu!("neqf64", |args, hand_mem, _| {
    let a = LittleEndian::read_f64(hand_mem.read(args[0], 8));
    let b = LittleEndian::read_f64(hand_mem.read(args[1], 8));
    let out = if a != b { 1u8 } else { 0u8 };
    hand_mem.write(args[2], 1, &out.to_le_bytes());
    None
  });
  cpu!("neqstr", |args, hand_mem, _| {
    let a_pascal_string = hand_mem.read(args[0], 0);
    let a_size = LittleEndian::read_u64(&a_pascal_string[0..8]) as usize;
    let a_str = str::from_utf8(&a_pascal_string[8..a_size + 8]).unwrap();
    let b_pascal_string = hand_mem.read(args[1], 0);
    let b_size = LittleEndian::read_u64(&b_pascal_string[0..8]) as usize;
    let b_str = str::from_utf8(&b_pascal_string[8..b_size + 8]).unwrap();
    let out = if a_pascal_string != b_pascal_string { 1u8 } else { 0u8 };
    hand_mem.write(args[2], 1, &out.to_le_bytes());
    None
  });
  cpu!("neqbool", |args, hand_mem, _| {
    let a = hand_mem.read(args[0], 1)[0];
    let b = hand_mem.read(args[1], 1)[0];
    let out = if a != b { 1u8 } else { 0u8 };
    hand_mem.write(args[2], 1, &out.to_le_bytes());
    None
  });

  cpu!("lti8", |args, hand_mem, _| {
    let a = i8::from_le_bytes(hand_mem.read(args[0], 1).try_into().unwrap());
    let b = i8::from_le_bytes(hand_mem.read(args[1], 1).try_into().unwrap());
    let out = if a < b { 1u8 } else { 0u8 };
    hand_mem.write(args[2], 1, &out.to_le_bytes());
    None
  });
  cpu!("lti16", |args, hand_mem, _| {
    let a = LittleEndian::read_i16(hand_mem.read(args[0], 2));
    let b = LittleEndian::read_i16(hand_mem.read(args[1], 2));
    let out = if a < b { 1u8 } else { 0u8 };
    hand_mem.write(args[2], 1, &out.to_le_bytes());
    None
  });
  cpu!("lti32", |args, hand_mem, _| {
    let a = LittleEndian::read_i32(hand_mem.read(args[0], 4));
    let b = LittleEndian::read_i32(hand_mem.read(args[1], 4));
    let out = if a < b { 1u8 } else { 0u8 };
    hand_mem.write(args[2], 1, &out.to_le_bytes());
    None
  });
  cpu!("lti64", |args, hand_mem, _| {
    let a = LittleEndian::read_i64(hand_mem.read(args[0], 8));
    let b = LittleEndian::read_i64(hand_mem.read(args[1], 8));
    let out = if a < b { 1u8 } else { 0u8 };
    hand_mem.write(args[2], 1, &out.to_le_bytes());
    None
  });
  cpu!("ltf32", |args, hand_mem, _| {
    let a = LittleEndian::read_f32(hand_mem.read(args[0], 4));
    let b = LittleEndian::read_f32(hand_mem.read(args[1], 4));
    let out = if a < b { 1u8 } else { 0u8 };
    hand_mem.write(args[2], 1, &out.to_le_bytes());
    None
  });
  cpu!("ltf64", |args, hand_mem, _| {
    let a = LittleEndian::read_f64(hand_mem.read(args[0], 8));
    let b = LittleEndian::read_f64(hand_mem.read(args[1], 8));
    let out = if a < b { 1u8 } else { 0u8 };
    hand_mem.write(args[2], 1, &out.to_le_bytes());
    None
  });
  cpu!("ltstr", |args, hand_mem, _| {
    let a_pascal_string = hand_mem.read(args[0], 0);
    let a_size = LittleEndian::read_u64(&a_pascal_string[0..8]) as usize;
    let a_str = str::from_utf8(&a_pascal_string[8..a_size + 8]).unwrap();
    let b_pascal_string = hand_mem.read(args[1], 0);
    let b_size = LittleEndian::read_u64(&b_pascal_string[0..8]) as usize;
    let b_str = str::from_utf8(&b_pascal_string[8..b_size + 8]).unwrap();
    let out = if a_pascal_string < b_pascal_string { 1u8 } else { 0u8 };
    hand_mem.write(args[2], 1, &out.to_le_bytes());
    None
  });

  cpu!("ltei8", |args, hand_mem, _| {
    let a = i8::from_le_bytes(hand_mem.read(args[0], 1).try_into().unwrap());
    let b = i8::from_le_bytes(hand_mem.read(args[1], 1).try_into().unwrap());
    let out = if a <= b { 1u8 } else { 0u8 };
    hand_mem.write(args[2], 1, &out.to_le_bytes());
    None
  });
  cpu!("ltei16", |args, hand_mem, _| {
    let a = LittleEndian::read_i16(hand_mem.read(args[0], 2));
    let b = LittleEndian::read_i16(hand_mem.read(args[1], 2));
    let out = if a <= b { 1u8 } else { 0u8 };
    hand_mem.write(args[2], 1, &out.to_le_bytes());
    None
  });
  cpu!("ltei32", |args, hand_mem, _| {
    let a = LittleEndian::read_i32(hand_mem.read(args[0], 4));
    let b = LittleEndian::read_i32(hand_mem.read(args[1], 4));
    let out = if a <= b { 1u8 } else { 0u8 };
    hand_mem.write(args[2], 1, &out.to_le_bytes());
    None
  });
  cpu!("ltei64", |args, hand_mem, _| {
    let a = LittleEndian::read_i64(hand_mem.read(args[0], 8));
    let b = LittleEndian::read_i64(hand_mem.read(args[1], 8));
    let out = if a <= b { 1u8 } else { 0u8 };
    hand_mem.write(args[2], 1, &out.to_le_bytes());
    None
  });
  cpu!("ltef32", |args, hand_mem, _| {
    let a = LittleEndian::read_f32(hand_mem.read(args[0], 4));
    let b = LittleEndian::read_f32(hand_mem.read(args[1], 4));
    let out = if a <= b { 1u8 } else { 0u8 };
    hand_mem.write(args[2], 1, &out.to_le_bytes());
    None
  });
  cpu!("ltef64", |args, hand_mem, _| {
    let a = LittleEndian::read_f64(hand_mem.read(args[0], 8));
    let b = LittleEndian::read_f64(hand_mem.read(args[1], 8));
    let out = if a <= b { 1u8 } else { 0u8 };
    hand_mem.write(args[2], 1, &out.to_le_bytes());
    None
  });
  cpu!("ltestr", |args, hand_mem, _| {
    let a_pascal_string = hand_mem.read(args[0], 0);
    let a_size = LittleEndian::read_u64(&a_pascal_string[0..8]) as usize;
    let a_str = str::from_utf8(&a_pascal_string[8..a_size + 8]).unwrap();
    let b_pascal_string = hand_mem.read(args[1], 0);
    let b_size = LittleEndian::read_u64(&b_pascal_string[0..8]) as usize;
    let b_str = str::from_utf8(&b_pascal_string[8..b_size + 8]).unwrap();
    let out = if a_pascal_string <= b_pascal_string { 1u8 } else { 0u8 };
    hand_mem.write(args[2], 1, &out.to_le_bytes());
    None
  });

  cpu!("gti8", |args, hand_mem, _| {
    let a = i8::from_le_bytes(hand_mem.read(args[0], 1).try_into().unwrap());
    let b = i8::from_le_bytes(hand_mem.read(args[1], 1).try_into().unwrap());
    let out = if a > b { 1u8 } else { 0u8 };
    hand_mem.write(args[2], 1, &out.to_le_bytes());
    None
  });
  cpu!("gti16", |args, hand_mem, _| {
    let a = LittleEndian::read_i16(hand_mem.read(args[0], 2));
    let b = LittleEndian::read_i16(hand_mem.read(args[1], 2));
    let out = if a > b { 1u8 } else { 0u8 };
    hand_mem.write(args[2], 1, &out.to_le_bytes());
    None
  });
  cpu!("gti32", |args, hand_mem, _| {
    let a = LittleEndian::read_i32(hand_mem.read(args[0], 4));
    let b = LittleEndian::read_i32(hand_mem.read(args[1], 4));
    let out = if a > b { 1u8 } else { 0u8 };
    hand_mem.write(args[2], 1, &out.to_le_bytes());
    None
  });
  cpu!("gti64", |args, hand_mem, _| {
    let a = LittleEndian::read_i64(hand_mem.read(args[0], 8));
    let b = LittleEndian::read_i64(hand_mem.read(args[1], 8));
    let out = if a > b { 1u8 } else { 0u8 };
    hand_mem.write(args[2], 1, &out.to_le_bytes());
    None
  });
  cpu!("gtf32", |args, hand_mem, _| {
    let a = LittleEndian::read_f32(hand_mem.read(args[0], 4));
    let b = LittleEndian::read_f32(hand_mem.read(args[1], 4));
    let out = if a > b { 1u8 } else { 0u8 };
    hand_mem.write(args[2], 1, &out.to_le_bytes());
    None
  });
  cpu!("gtf64", |args, hand_mem, _| {
    let a = LittleEndian::read_f64(hand_mem.read(args[0], 8));
    let b = LittleEndian::read_f64(hand_mem.read(args[1], 8));
    let out = if a > b { 1u8 } else { 0u8 };
    hand_mem.write(args[2], 1, &out.to_le_bytes());
    None
  });
  cpu!("gtstr", |args, hand_mem, _| {
    let a_pascal_string = hand_mem.read(args[0], 0);
    let a_size = LittleEndian::read_u64(&a_pascal_string[0..8]) as usize;
    let a_str = str::from_utf8(&a_pascal_string[8..a_size + 8]).unwrap();
    let b_pascal_string = hand_mem.read(args[1], 0);
    let b_size = LittleEndian::read_u64(&b_pascal_string[0..8]) as usize;
    let b_str = str::from_utf8(&b_pascal_string[8..b_size + 8]).unwrap();
    let out = if a_pascal_string > b_pascal_string { 1u8 } else { 0u8 };
    hand_mem.write(args[2], 1, &out.to_le_bytes());
    None
  });

  cpu!("gtei8", |args, hand_mem, _| {
    let a = i8::from_le_bytes(hand_mem.read(args[0], 1).try_into().unwrap());
    let b = i8::from_le_bytes(hand_mem.read(args[1], 1).try_into().unwrap());
    let out = if a >= b { 1u8 } else { 0u8 };
    hand_mem.write(args[2], 1, &out.to_le_bytes());
    None
  });
  cpu!("gtei16", |args, hand_mem, _| {
    let a = LittleEndian::read_i16(hand_mem.read(args[0], 2));
    let b = LittleEndian::read_i16(hand_mem.read(args[1], 2));
    let out = if a >= b { 1u8 } else { 0u8 };
    hand_mem.write(args[2], 1, &out.to_le_bytes());
    None
  });
  cpu!("gtei32", |args, hand_mem, _| {
    let a = LittleEndian::read_i32(hand_mem.read(args[0], 4));
    let b = LittleEndian::read_i32(hand_mem.read(args[1], 4));
    let out = if a >= b { 1u8 } else { 0u8 };
    hand_mem.write(args[2], 1, &out.to_le_bytes());
    None
  });
  cpu!("gtei64", |args, hand_mem, _| {
    let a = LittleEndian::read_i64(hand_mem.read(args[0], 8));
    let b = LittleEndian::read_i64(hand_mem.read(args[1], 8));
    let out = if a >= b { 1u8 } else { 0u8 };
    hand_mem.write(args[2], 1, &out.to_le_bytes());
    None
  });
  cpu!("gtef32", |args, hand_mem, _| {
    let a = LittleEndian::read_f32(hand_mem.read(args[0], 4));
    let b = LittleEndian::read_f32(hand_mem.read(args[1], 4));
    let out = if a >= b { 1u8 } else { 0u8 };
    hand_mem.write(args[2], 1, &out.to_le_bytes());
    None
  });
  cpu!("gtef64", |args, hand_mem, _| {
    let a = LittleEndian::read_f64(hand_mem.read(args[0], 8));
    let b = LittleEndian::read_f64(hand_mem.read(args[1], 8));
    let out = if a >= b { 1u8 } else { 0u8 };
    hand_mem.write(args[2], 1, &out.to_le_bytes());
    None
  });
  cpu!("gtestr", |args, hand_mem, _| {
    let a_pascal_string = hand_mem.read(args[0], 0);
    let a_size = LittleEndian::read_u64(&a_pascal_string[0..8]) as usize;
    let a_str = str::from_utf8(&a_pascal_string[8..a_size + 8]).unwrap();
    let b_pascal_string = hand_mem.read(args[1], 0);
    let b_size = LittleEndian::read_u64(&b_pascal_string[0..8]) as usize;
    let b_str = str::from_utf8(&b_pascal_string[8..b_size + 8]).unwrap();
    let out = if a_pascal_string >= b_pascal_string { 1u8 } else { 0u8 };
    hand_mem.write(args[2], 1, &out.to_le_bytes());
    None
  });

  // String opcodes
  cpu!("catstr", |args, hand_mem, _| {
    let a = hand_mem.read(args[0], 0);
    let b = hand_mem.read(args[1], 0);
    let a_size = LittleEndian::read_u64(&a[0..8]) as usize;
    let b_size = LittleEndian::read_u64(&b[0..8]) as usize;
    let a_str = str::from_utf8(&a[8..a_size + 8]).unwrap();
    let b_str = str::from_utf8(&b[8..b_size + 8]).unwrap();
    let out_str = format!("{}{}", a_str, b_str);
    let mut out = out_str.len().to_le_bytes().to_vec();
    out.append(&mut out_str.as_bytes().to_vec());
    hand_mem.write(args[2], 0, &out);
    None
  });
  cpu!("split", |args, hand_mem, _| {
    let a = hand_mem.read(args[0], 0);
    let b = hand_mem.read(args[1], 0);
    let a_size = LittleEndian::read_u64(&a[0..8]) as usize;
    let b_size = LittleEndian::read_u64(&b[0..8]) as usize;
    let a_str = str::from_utf8(&a[8..a_size + 8]).unwrap();
    let b_str = str::from_utf8(&b[8..b_size + 8]).unwrap();
    let outs: Vec<Vec<u8>> = a_str.split(b_str).map(|out_str| {
      let mut out = out_str.len().to_le_bytes().to_vec();
      out.append(&mut out_str.as_bytes().to_vec());
      return out;
    }).collect();
    hand_mem.new_arr(args[2]);
    for out in outs {
      hand_mem.push_arr(args[2], out, 0);
    }
    None
  });
  cpu!("repstr", |args, hand_mem, _| {
    let a_pascal_string = hand_mem.read(args[0], 0);
    let a_size = LittleEndian::read_u64(&a_pascal_string[0..8]) as usize;
    let a_str = str::from_utf8(&a_pascal_string[8..a_size + 8]).unwrap();
    let n = LittleEndian::read_i64(hand_mem.read(args[1], 8));
    let out_str = a_str.repeat(n as usize);
    let mut out = out_str.len().to_le_bytes().to_vec();
    out.append(&mut out_str.as_bytes().to_vec());
    hand_mem.write(args[2], 0, &out);
    None
  });
  cpu!("matches", |args, hand_mem, _| {
    let a_pascal_string = hand_mem.read(args[0], 0);
    let a_size = LittleEndian::read_u64(&a_pascal_string[0..8]) as usize;
    let a_str = str::from_utf8(&a_pascal_string[8..a_size + 8]).unwrap();
    let b_pascal_string = hand_mem.read(args[1], 0);
    let b_size = LittleEndian::read_u64(&b_pascal_string[0..8]) as usize;
    let b_str = str::from_utf8(&b_pascal_string[8..b_size + 8]).unwrap();
    let b_regex = Regex::new(b_str).unwrap();
    let out = if b_regex.is_match(a_str) { 1u8 } else { 0u8 };
    hand_mem.write(args[2], 1, &out.to_le_bytes());
    None
  });
  cpu!("indstr", |args, hand_mem, _| {
    let a_pascal_string = hand_mem.read(args[0], 0);
    let a_size = LittleEndian::read_u64(&a_pascal_string[0..8]) as usize;
    let a_str = str::from_utf8(&a_pascal_string[8..a_size + 8]).unwrap();
    let b_pascal_string = hand_mem.read(args[1], 0);
    let b_size = LittleEndian::read_u64(&b_pascal_string[0..8]) as usize;
    let b_str = str::from_utf8(&b_pascal_string[8..b_size + 8]).unwrap();
    let out_option = a_str.find(b_str);
    let out = if out_option.is_none()  { -1i64 } else { out_option.unwrap() as i64 };
    hand_mem.write(args[2], 8, &out.to_le_bytes());
    None
  });
  cpu!("lenstr", |args, hand_mem, _| {
    let a_pascal_string = hand_mem.read(args[0], 0);
    let a_size = LittleEndian::read_u64(&a_pascal_string[0..8]) as usize;
    let a_str = str::from_utf8(&a_pascal_string[8..a_size + 8]).unwrap();
    let out = a_str.len() as i64;
    hand_mem.write(args[2], 8, &out.to_le_bytes());
    None
  });
  cpu!("trim", |args, hand_mem, _| {
    let a_pascal_string = hand_mem.read(args[0], 0);
    let a_size = LittleEndian::read_u64(&a_pascal_string[0..8]) as usize;
    let a_str = str::from_utf8(&a_pascal_string[8..a_size + 8]).unwrap();
    let out_str = a_str.trim();
    let mut out = out_str.len().to_le_bytes().to_vec();
    out.append(&mut out_str.as_bytes().to_vec());
    hand_mem.write(args[2], 0, &out);
    None
  });

  // Array opcodes
  cpu!("register", |args, hand_mem, _| {
    // args[2] is the register address
    // args[0] point to an array in memory
    // args[1] is the address within the array to register
    let inner_addr = LittleEndian::read_i64(hand_mem.read(args[1], 8)) * 8;
    hand_mem.set_reg(args[2], args[0], inner_addr);
    None
  });
  cpu!("copyfrom", |args, hand_mem, _| {
    // args = [arr_addr, arr_idx_addr, outer_addr]
    // copy data from outer_addr to inner_addr of the array in reg_addr
    // The array index instead of inner address is provided to keep interaction with the js-runtime
    // sane.
    let inner_addr = LittleEndian::read_i64(hand_mem.read(args[1], 8)) * 8;
    hand_mem.copy_from(args[0], args[2], inner_addr);
    None
  });
  cpu!("copytof", |args, hand_mem, _| {
    // args = [arr_addr, outer_addr, inner_addr]
    // copy data from outer addr to inner_addr in arr_addr
    let inner = LittleEndian::read_i64(hand_mem.read(args[1], 8));
    hand_mem.copy_to(args[0], args[2], inner, 8);
    None
  });
  cpu!("copytov", |args, hand_mem, _| {
    // args = [arr_addr, outer_addr, inner_addr]
    // copy data from outer addr to inner_addr in arr_addr
    let inner = LittleEndian::read_i64(hand_mem.read(args[1], 8));
    hand_mem.copy_to(args[0], args[2], inner, 0);
    None
  });
  cpu!("lenarr", |args, hand_mem, _| {
    let arr = hand_mem.get_fractal(args[0]);
    let len = arr.len_as_arr() as i64;
    hand_mem.write(args[2], 8, &len.to_le_bytes());
    None
  });
  cpu!("indarrf", |args, hand_mem, _| {
    let val = LittleEndian::read_i64(hand_mem.read(args[1], 8));
    let mem = hand_mem.get_fractal(args[0]);
    let len = mem.len_as_arr() as i64;
    let mut idx = -1i64;
    for i in 0..len {
      let check = LittleEndian::read_i64(mem.read(i*8, 8));
      if val == check {
        idx = i;
        break
      }
    }
    hand_mem.write(args[2], 8, &idx.to_le_bytes());
    None
  });
  cpu!("indarrv", |args, hand_mem, _| {
    let val = hand_mem.read(args[1], 0);
    let mem = hand_mem.get_fractal(args[0]);
    let len = mem.len_as_arr() as i64;
    let mut idx = -1i64;
    for i in 0..len {
      let check = mem.read(i*8, 0);
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
        idx = i;
        break
      }
    }
    hand_mem.write(args[2], 8, &idx.to_le_bytes());
    None
  });
  cpu!("join", |args, hand_mem, _| {
    let sep_pascal_string = hand_mem.read(args[1], 0);
    let sep_size = LittleEndian::read_u64(&sep_pascal_string[0..8]) as usize;
    let sep_str = str::from_utf8(&sep_pascal_string[8..sep_size + 8]).unwrap();
    let mem = hand_mem.get_fractal(args[0]);
    let len = mem.len_as_arr() as i64;
    let mut strs: Vec<String> = Vec::new();
    for i in 0..len {
      let v_pascal_string = mem.read(i*8, 0);
      let v_size = LittleEndian::read_u64(&v_pascal_string[0..8]) as usize;
      let v_str = str::from_utf8(&v_pascal_string[8..v_size + 8]).unwrap().to_string();
      strs.push(v_str);
    }
    let out_str = strs.join(sep_str);
    let mut out = out_str.len().to_le_bytes().to_vec();
    out.append(&mut out_str.as_bytes().to_vec());
    hand_mem.write(args[2], 0, &out);
    None
  });
  cpu!("pusharr", |args, hand_mem, _| {
    let val_size = LittleEndian::read_i64(hand_mem.read(args[2], 8)) as u8;
    let val = hand_mem.read(args[1], val_size);
    let val_vec = val.to_vec();
    hand_mem.push_arr(args[0], val_vec, val_size);
    None
  });
  cpu!("poparr", |args, hand_mem, _| {
    let last = hand_mem.pop_arr(args[0]);
    hand_mem.write(args[1], last.len() as u8, last.as_slice());
    None
  });
  cpu!("newarr", |args, hand_mem, _| {
    hand_mem.new_arr(args[2]);
    None
  });
  // Map opcodes TODO after maps are implemented

  // Ternary opcodes
  // TODO: pair and condarr after arrays are implemented
  unpred_cpu!("condfn", |args, hand_mem, frag| {
    let cond = LittleEndian::read_i64(hand_mem.read(args[0], 8));
    let event_id = args[1];
    if cond == 1 {
      frag.insert_subhandler(event_id);
    }
    None
  });

  // Std opcodes
  unpred_cpu!("execop", |args, hand_mem, _| {
    let pascal_string = hand_mem.read(args[0], 0);
    let size = LittleEndian::read_u64(&pascal_string[0..8]) as usize;
    let full_cmd = str::from_utf8(&pascal_string[8..size + 8]).unwrap();
    let split_cmd: Vec<&str> = full_cmd.split(" ").collect();
    let output = Command::new(split_cmd[0]).args(&split_cmd[1..]).output();
    let result = match output {
      Err(e) => println!("Executing \"{}\" failed with: {}", full_cmd, e),
      Ok(out) => {
        io::stdout().write_all(&out.stdout).unwrap();
        io::stderr().write_all(&out.stderr).unwrap();
      },
    };
    None
  });

  // "Special" opcodes
  io!("waitop", |args, hand_mem, _| {
    let payload = hand_mem.read(args[0], 8);
    let ms = LittleEndian::read_i64(&payload[0..8]) as u64;
    return Box::pin(delay_for(Duration::from_millis(ms)));
  });
  cpu!("exitop", |args, hand_mem, _| {
    std::process::exit(LittleEndian::read_i32(hand_mem.read(args[0], 4)));
  });
  cpu!("stdoutp", |args, hand_mem, _| {
    let pascal_string = hand_mem.read(args[0], 0);
    let size = LittleEndian::read_u64(&pascal_string[0..8]) as usize;
    let out_str = str::from_utf8(&pascal_string[8..size + 8]).unwrap();
    print!("{}", out_str);
    None
  });
  // set opcodes use args[0] directly, since the relevant value directly
  // fits in i64, and write it to args[2]
  cpu!("seti64", |args, hand_mem, _| {
    let data = args[0];
    hand_mem.write(args[2], 8, &data.to_le_bytes());
    None
  });
  cpu!("seti32", |args, hand_mem, _| {
    let data = args[0] as i32;
    hand_mem.write(args[2], 4, &data.to_le_bytes());
    None
  });
  cpu!("seti16", |args, hand_mem, _| {
    let data = args[0] as i16;
    hand_mem.write(args[2], 2, &data.to_le_bytes());
    None
  });
  cpu!("seti8", |args, hand_mem, _| {
    let data = args[0] as i8;
    hand_mem.write(args[2], 1, &data.to_le_bytes());
    None
  });
  cpu!("setf64", |args, hand_mem, _| {
    let data = args[0] as f64;
    hand_mem.write(args[2], 8, &data.to_le_bytes());
    None
  });
  cpu!("setf32", |args, hand_mem, _| {
    let data = args[0] as f32;
    hand_mem.write(args[2], 4, &data.to_le_bytes());
    None
  });
  cpu!("setbool", |args, hand_mem, _| {
    let data = args[0] as u8;
    hand_mem.write(args[2], 1, &data.to_le_bytes());
    None
  });
  cpu!("setestr", |args, hand_mem, _| {
    let empty_str = 0i64.to_le_bytes().to_vec();
    hand_mem.write(args[2], 0, &empty_str);
    None
  });

  // copy opcodes used for let variable reassignments
  cpu!("copyi8", |args, hand_mem, _| {
    let val = hand_mem.read(args[0], 1).to_vec();
    hand_mem.write(args[2], 1, &val);
    None
  });
  cpu!("copyi16", |args, hand_mem, _| {
    let val = hand_mem.read(args[0], 2).to_vec();
    hand_mem.write(args[2], 2, &val);
    None
  });
  cpu!("copyi32", |args, hand_mem, _| {
    let val = hand_mem.read(args[0], 4).to_vec();
    hand_mem.write(args[2], 4, &val);
    None
  });
  cpu!("copyi64", |args, hand_mem, _| {
    let val = hand_mem.read(args[0], 8).to_vec();
    hand_mem.write(args[2], 8, &val);
    None
  });
  cpu!("copyf32", |args, hand_mem, _| {
    let val = hand_mem.read(args[0], 4).to_vec();
    hand_mem.write(args[2], 4, &val);
    None
  });
  cpu!("copyf64", |args, hand_mem, _| {
    let val = hand_mem.read(args[0], 8).to_vec();
    hand_mem.write(args[2], 8, &val);
    None
  });
  cpu!("copybool", |args, hand_mem, _| {
    let val = hand_mem.read(args[0], 1).to_vec();
    hand_mem.write(args[2], 1, &val);
    None
  });
  cpu!("copystr", |args, hand_mem, _| {
    let pascal_string = hand_mem.read(args[0], 0);
    let out = pascal_string.to_vec();
    hand_mem.write(args[2], 0, &out);
    None
  });
  cpu!("copyarr", |args, hand_mem, _| {
    // args = [in_addr, unused, out_addr]
    hand_mem.copy_arr(args[0], args[2]);
    None
  });

  cpu!("emit", |args, hand_mem, _| {
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
        panic!(format!("Illegal byte opcode {}", v));
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
