use std::collections::HashMap;
use std::convert::TryInto;
use std::future::Future;
use std::io::{self, Write};
use std::pin::Pin;
use std::process::Command;
use std::slice;
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

  macro_rules! unpred_cpu {
    ($name:expr, $func:expr) => {
      let id = opcode_id($name);
      let opcode = ByteOpcode {
        _id: id,
        _name: $name.to_string(),
        pred_exec: false,
        func: Some($func),
        async_func: None,
      };
      o.insert(id, opcode);
    };
  }

  // Type conversion opcodes
  cpu!("i8f64", |args, hand_mem, _| {
    let out = hand_mem.read_fixed(args[0]) as f64;
    hand_mem.write_fixed(args[2], i64::from_ne_bytes(out.to_ne_bytes()));
    None
  });
  cpu!("i16f64", |args, hand_mem, _| {
    let out = hand_mem.read_fixed(args[0]) as f64;
    hand_mem.write_fixed(args[2], i64::from_ne_bytes(out.to_ne_bytes()));
    None
  });
  cpu!("i32f64", |args, hand_mem, _| {
    let out = hand_mem.read_fixed(args[0]) as f64;
    hand_mem.write_fixed(args[2], i64::from_ne_bytes(out.to_ne_bytes()));
    None
  });
  cpu!("i64f64", |args, hand_mem, _| {
    let out = hand_mem.read_fixed(args[0]) as f64;
    hand_mem.write_fixed(args[2], i64::from_ne_bytes(out.to_ne_bytes()));
    None
  });
  cpu!("f32f64", |args, hand_mem, _| {
    let out = f32::from_ne_bytes((hand_mem.read_fixed(args[0]) as i32).to_ne_bytes());
    hand_mem.write_fixed(args[2], i32::from_ne_bytes(out.to_ne_bytes()) as i64);
    None
  });
  cpu!("strf64", |args, hand_mem, _| {
    unsafe {
      let pascal_string = hand_mem.read_fractal_mem(args[0]);
      let str_len = pascal_string[0] as usize;
      let pascal_string_u8 = slice::from_raw_parts(pascal_string[1..].as_ptr().cast::<u8>(), str_len*8);
      let out_str = str::from_utf8(&pascal_string_u8[0..str_len]).unwrap();
      let out: f64 = out_str.parse().unwrap();
      hand_mem.write_fixed(args[2], i64::from_ne_bytes(out.to_ne_bytes()));
    }
    None
  });
  cpu!("boolf64", |args, hand_mem, _| {
    let out = hand_mem.read_fixed(args[0]) as f64;
    hand_mem.write_fixed(args[2], i64::from_ne_bytes(out.to_ne_bytes()));
    None
  });

  cpu!("i8f32", |args, hand_mem, _| {
    let num = hand_mem.read_fixed(args[0]) as f32;
    let out = i32::from_ne_bytes(num.to_ne_bytes()) as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("i16f32", |args, hand_mem, _| {
    let num = hand_mem.read_fixed(args[0]) as f32;
    let out = i32::from_ne_bytes(num.to_ne_bytes()) as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("i32f32", |args, hand_mem, _| {
    let num = hand_mem.read_fixed(args[0]) as f32;
    let out = i32::from_ne_bytes(num.to_ne_bytes()) as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("i64f32", |args, hand_mem, _| {
    let num = hand_mem.read_fixed(args[0]) as f32;
    let out = i32::from_ne_bytes(num.to_ne_bytes()) as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("f64f32", |args, hand_mem, _| {
    let num = f64::from_ne_bytes(hand_mem.read_fixed(args[0]).to_ne_bytes()) as f32;
    let out = i32::from_ne_bytes(num.to_ne_bytes()) as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("strf32", |args, hand_mem, _| {
    unsafe {
      let pascal_string = hand_mem.read_fractal_mem(args[0]);
      let str_len = pascal_string[0] as usize;
      let pascal_string_u8 = slice::from_raw_parts(pascal_string[1..].as_ptr().cast::<u8>(), str_len*8);
      let out_str = str::from_utf8(&pascal_string_u8[0..str_len]).unwrap();
      let num: f32 = out_str.parse().unwrap();
      let out = i32::from_ne_bytes(num.to_ne_bytes()) as i64;
      hand_mem.write_fixed(args[2], out);
    }
    None
  });
  cpu!("boolf32", |args, hand_mem, _| {
    let num = hand_mem.read_fixed(args[0]) as f32;
    let out = i32::from_ne_bytes(num.to_ne_bytes()) as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });

  cpu!("i8i64", |args, hand_mem, _| {
    let out = hand_mem.read_fixed(args[0]);
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("i16i64", |args, hand_mem, _| {
    let out = hand_mem.read_fixed(args[0]);
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("i32i64", |args, hand_mem, _| {
    let out = hand_mem.read_fixed(args[0]);
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("f64i64", |args, hand_mem, _| {
    let out = f64::from_ne_bytes(hand_mem.read_fixed(args[0]).to_ne_bytes()) as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("f32i64", |args, hand_mem, _| {
    let out = f32::from_ne_bytes((hand_mem.read_fixed(args[0]) as i32).to_ne_bytes()) as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("stri64", |args, hand_mem, _| {
    unsafe {
      let pascal_string = hand_mem.read_fractal_mem(args[0]);
      let str_len = pascal_string[0] as usize;
      let pascal_string_u8 = slice::from_raw_parts(pascal_string[1..].as_ptr().cast::<u8>(), str_len*8);
      let out_str = str::from_utf8(&pascal_string_u8[0..str_len]).unwrap();
      let out: i64 = out_str.parse().unwrap();
      hand_mem.write_fixed(args[2], out);
    }
    None
  });
  cpu!("booli64", |args, hand_mem, _| {
    let out = hand_mem.read_fixed(args[0]);
    hand_mem.write_fixed(args[2], out);
    None
  });

  cpu!("i8i32", |args, hand_mem, _| {
    let out = hand_mem.read_fixed(args[0]);
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("i16i32", |args, hand_mem, _| {
    let out = hand_mem.read_fixed(args[0]);
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("i64i32", |args, hand_mem, _| {
    let out = (hand_mem.read_fixed(args[0]) as i32) as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("f64i32", |args, hand_mem, _| {
    let out = (f64::from_ne_bytes(hand_mem.read_fixed(args[0]).to_ne_bytes()) as i32) as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("f32i32", |args, hand_mem, _| {
    let out = (f32::from_ne_bytes((hand_mem.read_fixed(args[0]) as i32).to_ne_bytes()) as i32) as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("stri32", |args, hand_mem, _| {
    unsafe {
      let pascal_string = hand_mem.read_fractal_mem(args[0]);
      let str_len = pascal_string[0] as usize;
      let pascal_string_u8 = slice::from_raw_parts(pascal_string[1..].as_ptr().cast::<u8>(), str_len*8);
      let out_str = str::from_utf8(&pascal_string_u8[0..str_len]).unwrap();
      let num: i32 = out_str.parse().unwrap();
      let out = num as i64;
      hand_mem.write_fixed(args[2], out);
    }
    None
  });
  cpu!("booli32", |args, hand_mem, _| {
    let out = hand_mem.read_fixed(args[0]);
    hand_mem.write_fixed(args[2], out);
    None
  });

  cpu!("i8i16", |args, hand_mem, _| {
    let out = hand_mem.read_fixed(args[0]);
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("i32i16", |args, hand_mem, _| {
    let out = (hand_mem.read_fixed(args[0]) as i16) as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("i64i16", |args, hand_mem, _| {
    let out = (hand_mem.read_fixed(args[0]) as i16) as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("f64i16", |args, hand_mem, _| {
    let out = (f64::from_ne_bytes(hand_mem.read_fixed(args[0]).to_ne_bytes()) as i16) as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("f32i16", |args, hand_mem, _| {
    let out = (f32::from_ne_bytes((hand_mem.read_fixed(args[0]) as i32).to_ne_bytes()) as i16) as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("stri16", |args, hand_mem, _| {
    unsafe {
      let pascal_string = hand_mem.read_fractal_mem(args[0]);
      let str_len = pascal_string[0] as usize;
      let pascal_string_u8 = slice::from_raw_parts(pascal_string[1..].as_ptr().cast::<u8>(), str_len*8);
      let out_str = str::from_utf8(&pascal_string_u8[0..str_len]).unwrap();
      let num: i16 = out_str.parse().unwrap();
      let out = num as i64;
      hand_mem.write_fixed(args[2], out);
    }
    None
  });
  cpu!("booli16", |args, hand_mem, _| {
    let out = hand_mem.read_fixed(args[0]);
    hand_mem.write_fixed(args[2], out);
    None
  });

  cpu!("i16i8", |args, hand_mem, _| {
    let out = (hand_mem.read_fixed(args[0]) as i8) as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("i32i8", |args, hand_mem, _| {
    let out = (hand_mem.read_fixed(args[0]) as i8) as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("i64i8", |args, hand_mem, _| {
    let out = (hand_mem.read_fixed(args[0]) as i8) as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("f64i8", |args, hand_mem, _| {
    let out = (f64::from_ne_bytes(hand_mem.read_fixed(args[0]).to_ne_bytes()) as i8) as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("f32i8", |args, hand_mem, _| {
    let out = (f32::from_ne_bytes((hand_mem.read_fixed(args[0]) as i32).to_ne_bytes()) as i8) as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("stri8", |args, hand_mem, _| {
    unsafe {
      let pascal_string = hand_mem.read_fractal_mem(args[0]);
      let str_len = pascal_string[0] as usize;
      let pascal_string_u8 = slice::from_raw_parts(pascal_string[1..].as_ptr().cast::<u8>(), str_len*8);
      let out_str = str::from_utf8(&pascal_string_u8[0..str_len]).unwrap();
      let num: i8 = out_str.parse().unwrap();
      let out = num as i64;
      hand_mem.write_fixed(args[2], out);
    }
    None
  });
  cpu!("booli8", |args, hand_mem, _| {
    let out = (hand_mem.read_fixed(args[0]) as i8) as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });

  cpu!("i8bool", |args, hand_mem, _| {
    let a = hand_mem.read_fixed(args[0]) as i8;
    let out = if a != 0 { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("i16bool", |args, hand_mem, _| {
    let a = hand_mem.read_fixed(args[0]) as i16;
    let out = if a != 0 { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("i32bool", |args, hand_mem, _| {
    let a = hand_mem.read_fixed(args[0]) as i32;
    let out = if a != 0 { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("i64bool", |args, hand_mem, _| {
    let a = hand_mem.read_fixed(args[0]);
    let out = if a != 0 { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("f64bool", |args, hand_mem, _| {
    let a = f64::from_ne_bytes(hand_mem.read_fixed(args[0]).to_ne_bytes());
    let out = if a != 0.0 { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("f32bool", |args, hand_mem, _| {
    let a = f32::from_ne_bytes((hand_mem.read_fixed(args[0]) as i32).to_ne_bytes());
    let out = if a != 0.0 { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("strbool", |args, hand_mem, _| {
    unsafe {
      let pascal_string = hand_mem.read_fractal_mem(args[0]);
      let str_len = pascal_string[0] as usize;
      let pascal_string_u8 = slice::from_raw_parts(pascal_string[1..].as_ptr().cast::<u8>(), str_len*8);
      let out_str = str::from_utf8(&pascal_string_u8[0..str_len]).unwrap();
      let out = if out_str == "true" { 1i64 } else { 0i64 };
      hand_mem.write_fixed(args[2], out);
    }
    None
  });

  cpu!("i8str", |args, hand_mem, _| {
    let a = hand_mem.read_fixed(args[0]) as i8;
    let a_str = a.to_string();
    let mut out = vec![a_str.len() as i64];
    let mut a_str_bytes = a_str.as_bytes().to_vec();
    loop {
      if a_str_bytes.len() % 8 != 0 {
        a_str_bytes.push(0);
      } else {
        break
      }
    }
    let mut i = 0;
    loop {
      if i < a_str_bytes.len() {
        let str_slice = &a_str_bytes[i..i+8];
        out.push(i64::from_ne_bytes(str_slice.try_into().unwrap()));
        i = i + 8;
      } else {
        break
      }
    }
    hand_mem.write_fractal_mem(args[2], &out);
    None
  });
  cpu!("i16str", |args, hand_mem, _| {
    let a = hand_mem.read_fixed(args[0]) as i16;
    let a_str = a.to_string();
    let mut out = vec![a_str.len() as i64];
    let mut a_str_bytes = a_str.as_bytes().to_vec();
    loop {
      if a_str_bytes.len() % 8 != 0 {
        a_str_bytes.push(0);
      } else {
        break
      }
    }
    let mut i = 0;
    loop {
      if i < a_str_bytes.len() {
        let str_slice = &a_str_bytes[i..i+8];
        out.push(i64::from_ne_bytes(str_slice.try_into().unwrap()));
        i = i + 8;
      } else {
        break
      }
    }
    hand_mem.write_fractal_mem(args[2], &out);
    None
  });
  cpu!("i32str", |args, hand_mem, _| {
    let a = hand_mem.read_fixed(args[0]) as i32;
    let a_str = a.to_string();
    let mut out = vec![a_str.len() as i64];
    let mut a_str_bytes = a_str.as_bytes().to_vec();
    loop {
      if a_str_bytes.len() % 8 != 0 {
        a_str_bytes.push(0);
      } else {
        break
      }
    }
    let mut i = 0;
    loop {
      if i < a_str_bytes.len() {
        let str_slice = &a_str_bytes[i..i+8];
        out.push(i64::from_ne_bytes(str_slice.try_into().unwrap()));
        i = i + 8;
      } else {
        break
      }
    }
    hand_mem.write_fractal_mem(args[2], &out);
    None
  });
  cpu!("i64str", |args, hand_mem, _| {
    let a = hand_mem.read_fixed(args[0]);
    let a_str = a.to_string();
    let mut out = vec![a_str.len() as i64];
    let mut a_str_bytes = a_str.as_bytes().to_vec();
    loop {
      if a_str_bytes.len() % 8 != 0 {
        a_str_bytes.push(0);
      } else {
        break
      }
    }
    let mut i = 0;
    loop {
      if i < a_str_bytes.len() {
        let str_slice = &a_str_bytes[i..i+8];
        out.push(i64::from_ne_bytes(str_slice.try_into().unwrap()));
        i = i + 8;
      } else {
        break
      }
    }
    hand_mem.write_fractal_mem(args[2], &out);
    None
  });
  cpu!("f64str", |args, hand_mem, _| {
    let a = f64::from_ne_bytes(hand_mem.read_fixed(args[0]).to_ne_bytes());
    let a_str = a.to_string();
    let mut out = vec![a_str.len() as i64];
    let mut a_str_bytes = a_str.as_bytes().to_vec();
    loop {
      if a_str_bytes.len() % 8 != 0 {
        a_str_bytes.push(0);
      } else {
        break
      }
    }
    let mut i = 0;
    loop {
      if i < a_str_bytes.len() {
        let str_slice = &a_str_bytes[i..i+8];
        out.push(i64::from_ne_bytes(str_slice.try_into().unwrap()));
        i = i + 8;
      } else {
        break
      }
    }
    hand_mem.write_fractal_mem(args[2], &out);
    None
  });
  cpu!("f32str", |args, hand_mem, _| {
    let a = f32::from_ne_bytes((hand_mem.read_fixed(args[0]) as i32).to_ne_bytes());
    let a_str = a.to_string();
    let mut out = vec![a_str.len() as i64];
    let mut a_str_bytes = a_str.as_bytes().to_vec();
    loop {
      if a_str_bytes.len() % 8 != 0 {
        a_str_bytes.push(0);
      } else {
        break
      }
    }
    let mut i = 0;
    loop {
      if i < a_str_bytes.len() {
        let str_slice = &a_str_bytes[i..i+8];
        out.push(i64::from_ne_bytes(str_slice.try_into().unwrap()));
        i = i + 8;
      } else {
        break
      }
    }
    hand_mem.write_fractal_mem(args[2], &out);
    None
  });
  cpu!("boolstr", |args, hand_mem, _| {
    let a = hand_mem.read_fixed(args[0]);
    let a_str = if a == 1 { "true" } else { "false" };
    let mut out = vec![a_str.len() as i64];
    let mut a_str_bytes = a_str.as_bytes().to_vec();
    loop {
      if a_str_bytes.len() % 8 != 0 {
        a_str_bytes.push(0);
      } else {
        break
      }
    }
    let mut i = 0;
    loop {
      if i < a_str_bytes.len() {
        let str_slice = &a_str_bytes[i..i+8];
        out.push(i64::from_ne_bytes(str_slice.try_into().unwrap()));
        i = i + 8;
      } else {
        break
      }
    }
    hand_mem.write_fractal_mem(args[2], &out);
    None
  });

  // Arithmetic opcodes
  cpu!("addi8", |args, hand_mem, _| {
    let a = hand_mem.read_fixed(args[0]) as i8;
    let b = hand_mem.read_fixed(args[1]) as i8;
    let out = (a + b) as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("addi16", |args, hand_mem, _| {
    let a = hand_mem.read_fixed(args[0]) as i16;
    let b = hand_mem.read_fixed(args[1]) as i16;
    let out = (a + b) as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("addi32", |args, hand_mem, _| {
    let a = hand_mem.read_fixed(args[0]) as i32;
    let b = hand_mem.read_fixed(args[1]) as i32;
    let out = (a + b) as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("addi64", |args, hand_mem, _| {
    let a = hand_mem.read_fixed(args[0]);
    let b = hand_mem.read_fixed(args[1]);
    let out = a + b;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("addf32", |args, hand_mem, _| {
    let a = f32::from_ne_bytes((hand_mem.read_fixed(args[0]) as i32).to_ne_bytes());
    let b = f32::from_ne_bytes((hand_mem.read_fixed(args[1]) as i32).to_ne_bytes());
    let out = i32::from_ne_bytes((a + b).to_ne_bytes()) as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("addf64", |args, hand_mem, _| {
    let a = f64::from_ne_bytes(hand_mem.read_fixed(args[0]).to_ne_bytes());
    let b = f64::from_ne_bytes(hand_mem.read_fixed(args[1]).to_ne_bytes());
    let out = i64::from_ne_bytes((a + b).to_ne_bytes());
    hand_mem.write_fixed(args[2], out);
    None
  });

  cpu!("subi8", |args, hand_mem, _| {
    let a = hand_mem.read_fixed(args[0]) as i8;
    let b = hand_mem.read_fixed(args[1]) as i8;
    let out = (a - b) as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("subi16", |args, hand_mem, _| {
    let a = hand_mem.read_fixed(args[0]) as i16;
    let b = hand_mem.read_fixed(args[1]) as i16;
    let out = (a - b) as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("subi32", |args, hand_mem, _| {
    let a = hand_mem.read_fixed(args[0]) as i32;
    let b = hand_mem.read_fixed(args[1]) as i32;
    let out = (a - b) as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("subi64", |args, hand_mem, _| {
    let a = hand_mem.read_fixed(args[0]);
    let b = hand_mem.read_fixed(args[1]);
    let out = a - b;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("subf32", |args, hand_mem, _| {
    let a = f32::from_ne_bytes((hand_mem.read_fixed(args[0]) as i32).to_ne_bytes());
    let b = f32::from_ne_bytes((hand_mem.read_fixed(args[1]) as i32).to_ne_bytes());
    let out = i32::from_ne_bytes((a - b).to_ne_bytes()) as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("subf64", |args, hand_mem, _| {
    let a = f64::from_ne_bytes(hand_mem.read_fixed(args[0]).to_ne_bytes());
    let b = f64::from_ne_bytes(hand_mem.read_fixed(args[1]).to_ne_bytes());
    let out = i64::from_ne_bytes((a - b).to_ne_bytes());
    hand_mem.write_fixed(args[2], out);
    None
  });

  cpu!("negi8", |args, hand_mem, _| {
    let a = hand_mem.read_fixed(args[0]) as i8;
    let out = (0 - a) as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("negi16", |args, hand_mem, _| {
    let a = hand_mem.read_fixed(args[0]) as i16;
    let out = (0 - a) as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("negi32", |args, hand_mem, _| {
    let a = hand_mem.read_fixed(args[0]) as i32;
    let out = (0 - a) as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("negi64", |args, hand_mem, _| {
    let a = hand_mem.read_fixed(args[0]);
    let out = 0 - a;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("negf32", |args, hand_mem, _| {
    let a = f32::from_ne_bytes((hand_mem.read_fixed(args[0]) as i32).to_ne_bytes());
    let out = i32::from_ne_bytes((0.0 - a).to_ne_bytes()) as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("negf64", |args, hand_mem, _| {
    let a = f64::from_ne_bytes(hand_mem.read_fixed(args[0]).to_ne_bytes());
    let out = i64::from_ne_bytes((0.0 - a).to_ne_bytes());
    hand_mem.write_fixed(args[2], out);
    None
  });

  cpu!("muli8", |args, hand_mem, _| {
    let a = hand_mem.read_fixed(args[0]) as i8;
    let b = hand_mem.read_fixed(args[1]) as i8;
    let out = (a * b) as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("muli16", |args, hand_mem, _| {
    let a = hand_mem.read_fixed(args[0]) as i16;
    let b = hand_mem.read_fixed(args[1]) as i16;
    let out = (a * b) as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("muli32", |args, hand_mem, _| {
    let a = hand_mem.read_fixed(args[0]) as i32;
    let b = hand_mem.read_fixed(args[1]) as i32;
    let out = (a * b) as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("muli64", |args, hand_mem, _| {
    let a = hand_mem.read_fixed(args[0]);
    let b = hand_mem.read_fixed(args[1]);
    let out = a * b;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("mulf32", |args, hand_mem, _| {
    let a = f32::from_ne_bytes((hand_mem.read_fixed(args[0]) as i32).to_ne_bytes());
    let b = f32::from_ne_bytes((hand_mem.read_fixed(args[1]) as i32).to_ne_bytes());
    let out = i32::from_ne_bytes((a * b).to_ne_bytes()) as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("mulf64", |args, hand_mem, _| {
    let a = f64::from_ne_bytes(hand_mem.read_fixed(args[0]).to_ne_bytes());
    let b = f64::from_ne_bytes(hand_mem.read_fixed(args[1]).to_ne_bytes());
    let out = i64::from_ne_bytes((a * b).to_ne_bytes());
    hand_mem.write_fixed(args[2], out);
    None
  });

  cpu!("divi8", |args, hand_mem, _| {
    let a = hand_mem.read_fixed(args[0]) as i8;
    let b = hand_mem.read_fixed(args[1]) as i8;
    let out = (a / b) as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("divi16", |args, hand_mem, _| {
    let a = hand_mem.read_fixed(args[0]) as i16;
    let b = hand_mem.read_fixed(args[1]) as i16;
    let out = (a / b) as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("divi32", |args, hand_mem, _| {
    let a = hand_mem.read_fixed(args[0]) as i32;
    let b = hand_mem.read_fixed(args[1]) as i32;
    let out = (a / b) as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("divi64", |args, hand_mem, _| {
    let a = hand_mem.read_fixed(args[0]);
    let b = hand_mem.read_fixed(args[1]);
    let out = a / b;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("divf32", |args, hand_mem, _| {
    let a = f32::from_ne_bytes((hand_mem.read_fixed(args[0]) as i32).to_ne_bytes());
    let b = f32::from_ne_bytes((hand_mem.read_fixed(args[1]) as i32).to_ne_bytes());
    let out = i32::from_ne_bytes((a / b).to_ne_bytes()) as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("divf64", |args, hand_mem, _| {
    let a = f64::from_ne_bytes(hand_mem.read_fixed(args[0]).to_ne_bytes());
    let b = f64::from_ne_bytes(hand_mem.read_fixed(args[1]).to_ne_bytes());
    let out = i64::from_ne_bytes((a / b).to_ne_bytes());
    hand_mem.write_fixed(args[2], out);
    None
  });

  cpu!("modi8", |args, hand_mem, _| {
    let a = hand_mem.read_fixed(args[0]) as i8;
    let b = hand_mem.read_fixed(args[1]) as i8;
    let out = (a % b) as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("modi16", |args, hand_mem, _| {
    let a = hand_mem.read_fixed(args[0]) as i16;
    let b = hand_mem.read_fixed(args[1]) as i16;
    let out = (a % b) as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("modi32", |args, hand_mem, _| {
    let a = hand_mem.read_fixed(args[0]) as i32;
    let b = hand_mem.read_fixed(args[1]) as i32;
    let out = (a % b) as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("modi64", |args, hand_mem, _| {
    let a = hand_mem.read_fixed(args[0]);
    let b = hand_mem.read_fixed(args[1]);
    let out = a % b;
    hand_mem.write_fixed(args[2], out);
    None
  });

  cpu!("powi8", |args, hand_mem, _| {
    let a = hand_mem.read_fixed(args[0]) as i8;
    let b = hand_mem.read_fixed(args[1]) as i8;
    let out = if b < 0 { 0i64 } else { i8::pow(a, b as u32) as i64 };
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("powi16", |args, hand_mem, _| {
    let a = hand_mem.read_fixed(args[0]) as i16;
    let b = hand_mem.read_fixed(args[1]) as i16;
    let out = if b < 0 { 0i64 } else { i16::pow(a, b as u32) as i64 };
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("powi32", |args, hand_mem, _| {
    let a = hand_mem.read_fixed(args[0]) as i32;
    let b = hand_mem.read_fixed(args[1]) as i32;
    let out = if b < 0 { 0i64 } else { i32::pow(a, b as u32) as i64 };
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("powi64", |args, hand_mem, _| {
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
  cpu!("powf32", |args, hand_mem, _| {
    let a = f32::from_ne_bytes((hand_mem.read_fixed(args[0]) as i32).to_ne_bytes());
    let b = f32::from_ne_bytes((hand_mem.read_fixed(args[1]) as i32).to_ne_bytes());
    let out = i32::from_ne_bytes(f32::powf(a, b).to_ne_bytes()) as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("powf64", |args, hand_mem, _| {
    let a = f64::from_ne_bytes(hand_mem.read_fixed(args[0]).to_ne_bytes());
    let b = f64::from_ne_bytes(hand_mem.read_fixed(args[1]).to_ne_bytes());
    let out = i64::from_ne_bytes(f64::powf(a, b).to_ne_bytes());
    hand_mem.write_fixed(args[2], out);
    None
  });

  cpu!("sqrtf32", |args, hand_mem, _| {
    let a = f32::from_ne_bytes((hand_mem.read_fixed(args[0]) as i32).to_ne_bytes());
    let out = i32::from_ne_bytes(f32::sqrt(a).to_ne_bytes()) as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("sqrtf64", |args, hand_mem, _| {
    let a = f64::from_ne_bytes(hand_mem.read_fixed(args[0]).to_ne_bytes());
    let out = i64::from_ne_bytes(f64::sqrt(a).to_ne_bytes());
    hand_mem.write_fixed(args[2], out);
    None
  });

  // Boolean and bitwise opcodes
  cpu!("andi8", |args, hand_mem, _| {
    let a = hand_mem.read_fixed(args[0]) as i8;
    let b = hand_mem.read_fixed(args[1]) as i8;
    let out = (a & b) as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("andi16", |args, hand_mem, _| {
    let a = hand_mem.read_fixed(args[0]) as i16;
    let b = hand_mem.read_fixed(args[1]) as i16;
    let out = (a & b) as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("andi32", |args, hand_mem, _| {
    let a = hand_mem.read_fixed(args[0]) as i32;
    let b = hand_mem.read_fixed(args[1]) as i32;
    let out = (a & b) as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("andi64", |args, hand_mem, _| {
    let a = hand_mem.read_fixed(args[0]);
    let b = hand_mem.read_fixed(args[1]);
    let out = a & b;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("andbool", |args, hand_mem, _| {
    let a = hand_mem.read_fixed(args[0]);
    let b = hand_mem.read_fixed(args[1]);
    let a_bool = if a == 1 { true } else { false };
    let b_bool = if b == 1 { true } else { false };
    let out = if a_bool & b_bool { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out);
    None
  });

  cpu!("ori8", |args, hand_mem, _| {
    let a = hand_mem.read_fixed(args[0]) as i8;
    let b = hand_mem.read_fixed(args[1]) as i8;
    let out = (a | b) as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("ori16", |args, hand_mem, _| {
    let a = hand_mem.read_fixed(args[0]) as i16;
    let b = hand_mem.read_fixed(args[1]) as i16;
    let out = (a | b) as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("ori32", |args, hand_mem, _| {
    let a = hand_mem.read_fixed(args[0]) as i32;
    let b = hand_mem.read_fixed(args[1]) as i32;
    let out = (a | b) as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("ori64", |args, hand_mem, _| {
    let a = hand_mem.read_fixed(args[0]);
    let b = hand_mem.read_fixed(args[1]);
    let out = a | b;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("orbool", |args, hand_mem, _| {
    let a = hand_mem.read_fixed(args[0]);
    let b = hand_mem.read_fixed(args[1]);
    let a_bool = if a == 1 { true } else { false };
    let b_bool = if b == 1 { true } else { false };
    let out = if a_bool | b_bool { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out);
    None
  });

  cpu!("xori8", |args, hand_mem, _| {
    let a = hand_mem.read_fixed(args[0]) as i8;
    let b = hand_mem.read_fixed(args[1]) as i8;
    let out = (a ^ b) as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("xori16", |args, hand_mem, _| {
    let a = hand_mem.read_fixed(args[0]) as i16;
    let b = hand_mem.read_fixed(args[1]) as i16;
    let out = (a ^ b) as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("xori32", |args, hand_mem, _| {
    let a = hand_mem.read_fixed(args[0]) as i32;
    let b = hand_mem.read_fixed(args[1]) as i32;
    let out = (a ^ b) as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("xori64", |args, hand_mem, _| {
    let a = hand_mem.read_fixed(args[0]);
    let b = hand_mem.read_fixed(args[1]);
    let out = a ^ b;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("xorbool", |args, hand_mem, _| {
    let a = hand_mem.read_fixed(args[0]);
    let b = hand_mem.read_fixed(args[1]);
    let a_bool = if a == 1 { true } else { false };
    let b_bool = if b == 1 { true } else { false };
    let out = if a_bool ^ b_bool { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out);
    None
  });

  cpu!("noti8", |args, hand_mem, _| {
    let a = hand_mem.read_fixed(args[0]) as i8;
    let out = !a as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("noti16", |args, hand_mem, _| {
    let a = hand_mem.read_fixed(args[0]) as i16;
    let out = !a as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("noti32", |args, hand_mem, _| {
    let a = hand_mem.read_fixed(args[0]) as i32;
    let out = !a as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("noti64", |args, hand_mem, _| {
    let a = hand_mem.read_fixed(args[0]);
    let out = !a;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("notbool", |args, hand_mem, _| {
    let a = hand_mem.read_fixed(args[0]);
    let a_bool = if a == 1 { true } else { false };
    let out = if !a_bool { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out);
    None
  });

  cpu!("nandi8", |args, hand_mem, _| {
    let a = hand_mem.read_fixed(args[0]) as i8;
    let b = hand_mem.read_fixed(args[1]) as i8;
    let out = !(a & b) as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("nandi16", |args, hand_mem, _| {
    let a = hand_mem.read_fixed(args[0]) as i16;
    let b = hand_mem.read_fixed(args[1]) as i16;
    let out = !(a & b) as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("nandi32", |args, hand_mem, _| {
    let a = hand_mem.read_fixed(args[0]) as i32;
    let b = hand_mem.read_fixed(args[1]) as i32;
    let out = !(a & b) as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("nandi64", |args, hand_mem, _| {
    let a = hand_mem.read_fixed(args[0]);
    let b = hand_mem.read_fixed(args[1]);
    let out = !(a & b);
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("nandboo", |args, hand_mem, _| {
    let a = hand_mem.read_fixed(args[0]);
    let b = hand_mem.read_fixed(args[1]);
    let a_bool = if a == 1 { true } else { false };
    let b_bool = if b == 1 { true } else { false };
    let out = if !(a_bool & b_bool) { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out);
    None
  });

  cpu!("nori8", |args, hand_mem, _| {
    let a = hand_mem.read_fixed(args[0]) as i8;
    let b = hand_mem.read_fixed(args[1]) as i8;
    let out = !(a | b) as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("nori16", |args, hand_mem, _| {
    let a = hand_mem.read_fixed(args[0]) as i16;
    let b = hand_mem.read_fixed(args[1]) as i16;
    let out = !(a | b) as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("nori32", |args, hand_mem, _| {
    let a = hand_mem.read_fixed(args[0]) as i32;
    let b = hand_mem.read_fixed(args[1]) as i32;
    let out = !(a | b) as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("nori64", |args, hand_mem, _| {
    let a = hand_mem.read_fixed(args[0]);
    let b = hand_mem.read_fixed(args[1]);
    let out = !(a | b);
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("norbool", |args, hand_mem, _| {
    let a = hand_mem.read_fixed(args[0]);
    let b = hand_mem.read_fixed(args[1]);
    let a_bool = if a == 1 { true } else { false };
    let b_bool = if b == 1 { true } else { false };
    let out = if !(a_bool | b_bool) { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out);
    None
  });

  cpu!("xnori8", |args, hand_mem, _| {
    let a = hand_mem.read_fixed(args[0]) as i8;
    let b = hand_mem.read_fixed(args[1]) as i8;
    let out = !(a ^ b) as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("xnori16", |args, hand_mem, _| {
    let a = hand_mem.read_fixed(args[0]) as i16;
    let b = hand_mem.read_fixed(args[1]) as i16;
    let out = !(a ^ b) as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("xnori32", |args, hand_mem, _| {
    let a = hand_mem.read_fixed(args[0]) as i32;
    let b = hand_mem.read_fixed(args[1]) as i32;
    let out = !(a ^ b) as i64;
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("xnori64", |args, hand_mem, _| {
    let a = hand_mem.read_fixed(args[0]);
    let b = hand_mem.read_fixed(args[1]);
    let out = !(a ^ b);
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("xnorboo", |args, hand_mem, _| {
    let a = hand_mem.read_fixed(args[0]);
    let b = hand_mem.read_fixed(args[1]);
    let a_bool = if a == 1 { true } else { false };
    let b_bool = if b == 1 { true } else { false };
    let out = if !(a_bool ^ b_bool) { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out);
    None
  });

  // Equality and order opcodes
  cpu!("eqi8", |args, hand_mem, _| {
    let a = hand_mem.read_fixed(args[0]) as i8;
    let b = hand_mem.read_fixed(args[1]) as i8;
    let out = if a == b { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("eqi16", |args, hand_mem, _| {
    let a = hand_mem.read_fixed(args[0]) as i16;
    let b = hand_mem.read_fixed(args[1]) as i16;
    let out = if a == b { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("eqi32", |args, hand_mem, _| {
    let a = hand_mem.read_fixed(args[0]) as i32;
    let b = hand_mem.read_fixed(args[1]) as i32;
    let out = if a == b { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("eqi64", |args, hand_mem, _| {
    let a = hand_mem.read_fixed(args[0]);
    let b = hand_mem.read_fixed(args[1]);
    let out = if a == b { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("eqf32", |args, hand_mem, _| {
    let a = f32::from_ne_bytes((hand_mem.read_fixed(args[0]) as i32).to_ne_bytes());
    let b = f32::from_ne_bytes((hand_mem.read_fixed(args[1]) as i32).to_ne_bytes());
    let out = if a == b { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("eqf64", |args, hand_mem, _| {
    let a = f64::from_ne_bytes(hand_mem.read_fixed(args[0]).to_ne_bytes());
    let b = f64::from_ne_bytes(hand_mem.read_fixed(args[1]).to_ne_bytes());
    let out = if a == b { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("eqstr", |args, hand_mem, _| {
    let a_pascal_string = hand_mem.read_fractal_mem(args[0]);
    let b_pascal_string = hand_mem.read_fractal_mem(args[1]);
    let out = if a_pascal_string == b_pascal_string { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("eqbool", |args, hand_mem, _| {
    let a = hand_mem.read_fixed(args[0]) as i8;
    let b = hand_mem.read_fixed(args[1]) as i8;
    let out = if a == b { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out);
    None
  });

  cpu!("neqi8", |args, hand_mem, _| {
    let a = hand_mem.read_fixed(args[0]) as i8;
    let b = hand_mem.read_fixed(args[1]) as i8;
    let out = if a != b { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("neqi16", |args, hand_mem, _| {
    let a = hand_mem.read_fixed(args[0]) as i16;
    let b = hand_mem.read_fixed(args[1]) as i16;
    let out = if a != b { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("neqi32", |args, hand_mem, _| {
    let a = hand_mem.read_fixed(args[0]) as i32;
    let b = hand_mem.read_fixed(args[1]) as i32;
    let out = if a != b { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("neqi64", |args, hand_mem, _| {
    let a = hand_mem.read_fixed(args[0]);
    let b = hand_mem.read_fixed(args[1]);
    let out = if a != b { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("neqf32", |args, hand_mem, _| {
    let a = f32::from_ne_bytes((hand_mem.read_fixed(args[0]) as i32).to_ne_bytes());
    let b = f32::from_ne_bytes((hand_mem.read_fixed(args[1]) as i32).to_ne_bytes());
    let out = if a != b { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("neqf64", |args, hand_mem, _| {
    let a = f64::from_ne_bytes(hand_mem.read_fixed(args[0]).to_ne_bytes());
    let b = f64::from_ne_bytes(hand_mem.read_fixed(args[1]).to_ne_bytes());
    let out = if a != b { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("neqstr", |args, hand_mem, _| {
    let a_pascal_string = hand_mem.read_fractal_mem(args[0]);
    let b_pascal_string = hand_mem.read_fractal_mem(args[1]);
    let out = if a_pascal_string != b_pascal_string { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("neqbool", |args, hand_mem, _| {
    let a = hand_mem.read_fixed(args[0]) as i8;
    let b = hand_mem.read_fixed(args[1]) as i8;
    let out = if a != b { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out);
    None
  });

  cpu!("lti8", |args, hand_mem, _| {
    let a = hand_mem.read_fixed(args[0]) as i8;
    let b = hand_mem.read_fixed(args[1]) as i8;
    let out = if a < b { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("lti16", |args, hand_mem, _| {
    let a = hand_mem.read_fixed(args[0]) as i16;
    let b = hand_mem.read_fixed(args[1]) as i16;
    let out = if a < b { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("lti32", |args, hand_mem, _| {
    let a = hand_mem.read_fixed(args[0]) as i32;
    let b = hand_mem.read_fixed(args[1]) as i32;
    let out = if a < b { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("lti64", |args, hand_mem, _| {
    let a = hand_mem.read_fixed(args[0]);
    let b = hand_mem.read_fixed(args[1]);
    let out = if a < b { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("ltf32", |args, hand_mem, _| {
    let a = f32::from_ne_bytes((hand_mem.read_fixed(args[0]) as i32).to_ne_bytes());
    let b = f32::from_ne_bytes((hand_mem.read_fixed(args[1]) as i32).to_ne_bytes());
    let out = if a < b { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("ltf64", |args, hand_mem, _| {
    let a = f64::from_ne_bytes(hand_mem.read_fixed(args[0]).to_ne_bytes());
    let b = f64::from_ne_bytes(hand_mem.read_fixed(args[1]).to_ne_bytes());
    let out = if a < b { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("ltstr", |args, hand_mem, _| {
    unsafe {
      let a_pascal_string = hand_mem.read_fractal_mem(args[0]);
      let a_str_len = a_pascal_string[0] as usize;
      let a_pascal_string_u8 = slice::from_raw_parts(a_pascal_string[1..].as_ptr().cast::<u8>(), a_str_len*8);
      let a_str = str::from_utf8(&a_pascal_string_u8[0..a_str_len]).unwrap();
      let b_pascal_string = hand_mem.read_fractal_mem(args[1]);
      let b_str_len = b_pascal_string[0] as usize;
      let b_pascal_string_u8 = slice::from_raw_parts(b_pascal_string[1..].as_ptr().cast::<u8>(), b_str_len*8);
      let b_str = str::from_utf8(&b_pascal_string_u8[0..b_str_len]).unwrap();
      let out = if a_str < b_str { 1i64 } else { 0i64 };
      hand_mem.write_fixed(args[2], out);
    }
    None
  });

  cpu!("ltei8", |args, hand_mem, _| {
    let a = hand_mem.read_fixed(args[0]) as i8;
    let b = hand_mem.read_fixed(args[1]) as i8;
    let out = if a <= b { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("ltei16", |args, hand_mem, _| {
    let a = hand_mem.read_fixed(args[0]) as i16;
    let b = hand_mem.read_fixed(args[1]) as i16;
    let out = if a <= b { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("ltei32", |args, hand_mem, _| {
    let a = hand_mem.read_fixed(args[0]) as i32;
    let b = hand_mem.read_fixed(args[1]) as i32;
    let out = if a <= b { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("ltei64", |args, hand_mem, _| {
    let a = hand_mem.read_fixed(args[0]);
    let b = hand_mem.read_fixed(args[1]);
    let out = if a <= b { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("ltef32", |args, hand_mem, _| {
    let a = f32::from_ne_bytes((hand_mem.read_fixed(args[0]) as i32).to_ne_bytes());
    let b = f32::from_ne_bytes((hand_mem.read_fixed(args[1]) as i32).to_ne_bytes());
    let out = if a <= b { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("ltef64", |args, hand_mem, _| {
    let a = f64::from_ne_bytes(hand_mem.read_fixed(args[0]).to_ne_bytes());
    let b = f64::from_ne_bytes(hand_mem.read_fixed(args[1]).to_ne_bytes());
    let out = if a <= b { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("ltestr", |args, hand_mem, _| {
    unsafe {
      let a_pascal_string = hand_mem.read_fractal_mem(args[0]);
      let a_str_len = a_pascal_string[0] as usize;
      let a_pascal_string_u8 = slice::from_raw_parts(a_pascal_string[1..].as_ptr().cast::<u8>(), a_str_len*8);
      let a_str = str::from_utf8(&a_pascal_string_u8[0..a_str_len]).unwrap();
      let b_pascal_string = hand_mem.read_fractal_mem(args[1]);
      let b_str_len = b_pascal_string[0] as usize;
      let b_pascal_string_u8 = slice::from_raw_parts(b_pascal_string[1..].as_ptr().cast::<u8>(), b_str_len*8);
      let b_str = str::from_utf8(&b_pascal_string_u8[0..b_str_len]).unwrap();
      let out = if a_str <= b_str { 1i64 } else { 0i64 };
      hand_mem.write_fixed(args[2], out);
    }
    None
  });

  cpu!("gti8", |args, hand_mem, _| {
    let a = hand_mem.read_fixed(args[0]) as i8;
    let b = hand_mem.read_fixed(args[1]) as i8;
    let out = if a > b { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("gti16", |args, hand_mem, _| {
    let a = hand_mem.read_fixed(args[0]) as i16;
    let b = hand_mem.read_fixed(args[1]) as i16;
    let out = if a > b { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("gti32", |args, hand_mem, _| {
    let a = hand_mem.read_fixed(args[0]) as i32;
    let b = hand_mem.read_fixed(args[1]) as i32;
    let out = if a > b { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("gti64", |args, hand_mem, _| {
    let a = hand_mem.read_fixed(args[0]);
    let b = hand_mem.read_fixed(args[1]);
    let out = if a > b { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("gtf32", |args, hand_mem, _| {
    let a = f32::from_ne_bytes((hand_mem.read_fixed(args[0]) as i32).to_ne_bytes());
    let b = f32::from_ne_bytes((hand_mem.read_fixed(args[1]) as i32).to_ne_bytes());
    let out = if a > b { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("gtf64", |args, hand_mem, _| {
    let a = f64::from_ne_bytes(hand_mem.read_fixed(args[0]).to_ne_bytes());
    let b = f64::from_ne_bytes(hand_mem.read_fixed(args[1]).to_ne_bytes());
    let out = if a > b { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("gtstr", |args, hand_mem, _| {
    unsafe {
      let a_pascal_string = hand_mem.read_fractal_mem(args[0]);
      let a_str_len = a_pascal_string[0] as usize;
      let a_pascal_string_u8 = slice::from_raw_parts(a_pascal_string[1..].as_ptr().cast::<u8>(), a_str_len*8);
      let a_str = str::from_utf8(&a_pascal_string_u8[0..a_str_len]).unwrap();
      let b_pascal_string = hand_mem.read_fractal_mem(args[1]);
      let b_str_len = b_pascal_string[0] as usize;
      let b_pascal_string_u8 = slice::from_raw_parts(b_pascal_string[1..].as_ptr().cast::<u8>(), b_str_len*8);
      let b_str = str::from_utf8(&b_pascal_string_u8[0..b_str_len]).unwrap();
      let out = if a_str > b_str { 1i64 } else { 0i64 };
      hand_mem.write_fixed(args[2], out);
    }
    None
  });

  cpu!("gtei8", |args, hand_mem, _| {
    let a = hand_mem.read_fixed(args[0]) as i8;
    let b = hand_mem.read_fixed(args[1]) as i8;
    let out = if a >= b { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("gtei16", |args, hand_mem, _| {
    let a = hand_mem.read_fixed(args[0]) as i16;
    let b = hand_mem.read_fixed(args[1]) as i16;
    let out = if a >= b { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("gtei32", |args, hand_mem, _| {
    let a = hand_mem.read_fixed(args[0]) as i32;
    let b = hand_mem.read_fixed(args[1]) as i32;
    let out = if a >= b { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("gtei64", |args, hand_mem, _| {
    let a = hand_mem.read_fixed(args[0]);
    let b = hand_mem.read_fixed(args[1]);
    let out = if a >= b { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("gtef32", |args, hand_mem, _| {
    let a = f32::from_ne_bytes((hand_mem.read_fixed(args[0]) as i32).to_ne_bytes());
    let b = f32::from_ne_bytes((hand_mem.read_fixed(args[1]) as i32).to_ne_bytes());
    let out = if a >= b { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("gtef64", |args, hand_mem, _| {
    let a = f64::from_ne_bytes(hand_mem.read_fixed(args[0]).to_ne_bytes());
    let b = f64::from_ne_bytes(hand_mem.read_fixed(args[1]).to_ne_bytes());
    let out = if a >= b { 1i64 } else { 0i64 };
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("gtestr", |args, hand_mem, _| {
    unsafe {
      let a_pascal_string = hand_mem.read_fractal_mem(args[0]);
      let a_str_len = a_pascal_string[0] as usize;
      let a_pascal_string_u8 = slice::from_raw_parts(a_pascal_string[1..].as_ptr().cast::<u8>(), a_str_len*8);
      let a_str = str::from_utf8(&a_pascal_string_u8[0..a_str_len]).unwrap();
      let b_pascal_string = hand_mem.read_fractal_mem(args[1]);
      let b_str_len = b_pascal_string[0] as usize;
      let b_pascal_string_u8 = slice::from_raw_parts(b_pascal_string[1..].as_ptr().cast::<u8>(), b_str_len*8);
      let b_str = str::from_utf8(&b_pascal_string_u8[0..b_str_len]).unwrap();
      let out = if a_str >= b_str { 1i64 } else { 0i64 };
      hand_mem.write_fixed(args[2], out);
    }
    None
  });

  // String opcodes
  cpu!("catstr", |args, hand_mem, _| {
    unsafe {
      let a_pascal_string = hand_mem.read_fractal_mem(args[0]);
      let a_str_len = a_pascal_string[0] as usize;
      let a_pascal_string_u8 = slice::from_raw_parts(a_pascal_string[1..].as_ptr().cast::<u8>(), a_str_len*8);
      let a_str = str::from_utf8(&a_pascal_string_u8[0..a_str_len]).unwrap();
      let b_pascal_string = hand_mem.read_fractal_mem(args[1]);
      let b_str_len = b_pascal_string[0] as usize;
      let b_pascal_string_u8 = slice::from_raw_parts(b_pascal_string[1..].as_ptr().cast::<u8>(), b_str_len*8);
      let b_str = str::from_utf8(&b_pascal_string_u8[0..b_str_len]).unwrap();
      let out_str = format!("{}{}", a_str, b_str);
      let mut out = vec![out_str.len() as i64];
      let mut out_str_bytes = out_str.as_bytes().to_vec();
      loop {
        if out_str_bytes.len() % 8 != 0 {
          out_str_bytes.push(0);
        } else {
          break
        }
      }
      let mut i = 0;
      loop {
        if i < out_str_bytes.len() {
          let str_slice = &out_str_bytes[i..i+8];
          out.push(i64::from_ne_bytes(str_slice.try_into().unwrap()));
          i = i + 8;
        } else {
          break
        }
      }
      hand_mem.write_fractal_mem(args[2], &out);
    }
    None
  });
  cpu!("split", |args, hand_mem, _| {
    unsafe {
      let a_pascal_string = hand_mem.read_fractal_mem(args[0]);
      let a_str_len = a_pascal_string[0] as usize;
      let a_pascal_string_u8 = slice::from_raw_parts(a_pascal_string[1..].as_ptr().cast::<u8>(), a_str_len*8);
      let a_str = str::from_utf8(&a_pascal_string_u8[0..a_str_len]).unwrap();
      let b_pascal_string = hand_mem.read_fractal_mem(args[1]);
      let b_str_len = b_pascal_string[0] as usize;
      let b_pascal_string_u8 = slice::from_raw_parts(b_pascal_string[1..].as_ptr().cast::<u8>(), b_str_len*8);
      let b_str = str::from_utf8(&b_pascal_string_u8[0..b_str_len]).unwrap();
      let outs: Vec<Vec<i64>> = a_str.split(b_str).map(|out_str| {
        let mut out = vec![out_str.len() as i64];
        let mut out_str_bytes = out_str.as_bytes().to_vec();
        loop {
          if out_str_bytes.len() % 8 != 0 {
            out_str_bytes.push(0);
          } else {
            break
          }
        }
        let mut i = 0;
        loop {
          if i < out_str_bytes.len() {
            let str_slice = &out_str_bytes[i..i+8];
            out.push(i64::from_ne_bytes(str_slice.try_into().unwrap()));
            i = i + 8;
          } else {
            break
          }
        }
        return out;
      }).collect();
      hand_mem.new_fractal(args[2]);
      for out in outs {
        hand_mem.push_fractal_fractal_mem(args[2], out);
      }
    }
    None
  });
  cpu!("repstr", |args, hand_mem, _| {
    unsafe {
      let a_pascal_string = hand_mem.read_fractal_mem(args[0]);
      let a_str_len = a_pascal_string[0] as usize;
      let a_pascal_string_u8 = slice::from_raw_parts(a_pascal_string[1..].as_ptr().cast::<u8>(), a_str_len*8);
      let a_str = str::from_utf8(&a_pascal_string_u8[0..a_str_len]).unwrap();
      let n = hand_mem.read_fixed(args[1]);
      let out_str = a_str.repeat(n as usize);
      let mut out = vec![out_str.len() as i64];
      let mut out_str_bytes = out_str.as_bytes().to_vec();
      loop {
        if out_str_bytes.len() % 8 != 0 {
          out_str_bytes.push(0);
        } else {
          break
        }
      }
      let mut i = 0;
      loop {
        if i < out_str_bytes.len() {
          let str_slice = &out_str_bytes[i..i+8];
          out.push(i64::from_ne_bytes(str_slice.try_into().unwrap()));
          i = i + 8;
        } else {
          break
        }
      }
      hand_mem.write_fractal_mem(args[2], &out);
    }
    None
  });
  cpu!("matches", |args, hand_mem, _| {
    unsafe {
      let a_pascal_string = hand_mem.read_fractal_mem(args[0]);
      let a_str_len = a_pascal_string[0] as usize;
      let a_pascal_string_u8 = slice::from_raw_parts(a_pascal_string[1..].as_ptr().cast::<u8>(), a_str_len*8);
      let a_str = str::from_utf8(&a_pascal_string_u8[0..a_str_len]).unwrap();
      let b_pascal_string = hand_mem.read_fractal_mem(args[1]);
      let b_str_len = b_pascal_string[0] as usize;
      let b_pascal_string_u8 = slice::from_raw_parts(b_pascal_string[1..].as_ptr().cast::<u8>(), b_str_len*8);
      let b_str = str::from_utf8(&b_pascal_string_u8[0..b_str_len]).unwrap();
      let b_regex = Regex::new(b_str).unwrap();
      let out = if b_regex.is_match(a_str) { 1i64 } else { 0i64 };
      hand_mem.write_fixed(args[2], out);
    }
    None
  });
  cpu!("indstr", |args, hand_mem, _| {
    unsafe {
      let a_pascal_string = hand_mem.read_fractal_mem(args[0]);
      let a_str_len = a_pascal_string[0] as usize;
      let a_pascal_string_u8 = slice::from_raw_parts(a_pascal_string[1..].as_ptr().cast::<u8>(), a_str_len*8);
      let a_str = str::from_utf8(&a_pascal_string_u8[0..a_str_len]).unwrap();
      let b_pascal_string = hand_mem.read_fractal_mem(args[1]);
      let b_str_len = b_pascal_string[0] as usize;
      let b_pascal_string_u8 = slice::from_raw_parts(b_pascal_string[1..].as_ptr().cast::<u8>(), b_str_len*8);
      let b_str = str::from_utf8(&b_pascal_string_u8[0..b_str_len]).unwrap();
      let out_option = a_str.find(b_str);
      let out = if out_option.is_none()  { -1i64 } else { out_option.unwrap() as i64 };
      hand_mem.write_fixed(args[2], out);
    }
    None
  });
  cpu!("lenstr", |args, hand_mem, _| {
    let pascal_string = hand_mem.read_fractal_mem(args[0]);
    let out = pascal_string[0];
    hand_mem.write_fixed(args[2], out);
    None
  });
  cpu!("trim", |args, hand_mem, _| {
    unsafe {
      let pascal_string = hand_mem.read_fractal_mem(args[0]);
      let str_len = pascal_string[0] as usize;
      let pascal_string_u8 = slice::from_raw_parts(pascal_string[1..].as_ptr().cast::<u8>(), str_len*8);
      let out_str = str::from_utf8(&pascal_string_u8[0..str_len]).unwrap().trim();
      let mut out = vec![out_str.len() as i64];
      let mut out_str_bytes = out_str.as_bytes().to_vec();
      loop {
        if out_str_bytes.len() % 8 != 0 {
          out_str_bytes.push(0);
        } else {
          break
        }
      }
      let mut i = 0;
      loop {
        if i < out_str_bytes.len() {
          let str_slice = &out_str_bytes[i..i+8];
          out.push(i64::from_ne_bytes(str_slice.try_into().unwrap()));
          i = i + 8;
        } else {
          break
        }
      }
      hand_mem.write_fractal_mem(args[2], &out);
    }
    None
  });

  // Array opcodes
  cpu!("register", |args, hand_mem, _| {
    // args[2] is the register address
    // args[0] point to an array in memory
    // args[1] is the address within the array to register
    let inner_addr = hand_mem.read_fixed(args[1]);
    hand_mem.set_reg(args[2], args[0], inner_addr);
    None
  });
  cpu!("copyfrom", |args, hand_mem, _| {
    // args = [arr_addr, arr_idx_addr, outer_addr]
    // copy data from outer_addr to inner_addr of the array in reg_addr
    // The array index instead of inner address is provided to keep interaction with the js-runtime
    // sane.
    let inner_addr = hand_mem.read_fixed(args[1]);
    hand_mem.copy_from(args[0], args[2], inner_addr);
    None
  });
  cpu!("copytof", |args, hand_mem, _| {
    // args = [arr_addr, inner_addr, outer_addr]
    // copy data from outer_addr to inner_addr in arr_addr
    let inner = hand_mem.read_fixed(args[2]);
    hand_mem.copy_to_fixed(args[0], args[1], inner);
    None
  });
  cpu!("copytov", |args, hand_mem, _| {
    // args = [arr_addr, inner_addr, outer_addr]
    // copy data from outer_addr to inner_addr in arr_addr
    let inner = hand_mem.read_fixed(args[2]);
    hand_mem.copy_to_fractal_mem(args[0], args[1], inner);
    None
  });
  cpu!("lenarr", |args, hand_mem, _| {
    let arr = hand_mem.get_fractal(args[0]);
    let len = arr.len() as i64;
    hand_mem.write_fixed(args[2], len);
    None
  });
  cpu!("indarrf", |args, hand_mem, _| {
    let val = hand_mem.read_fixed(args[1]);
    let mem = hand_mem.get_fractal(args[0]);
    let len = mem.len() as i64;
    let mut idx = -1i64;
    for i in 0..len {
      let check = mem.read_fixed(i);
      if val == check {
        idx = i;
        break
      }
    }
    hand_mem.write_fixed(args[2], idx);
    None
  });
  cpu!("indarrv", |args, hand_mem, _| {
    let val = hand_mem.read_fractal_mem(args[1]);
    let mem = hand_mem.get_fractal(args[0]);
    let len = mem.len() as i64;
    let mut idx = -1i64;
    for i in 0..len {
      let check = mem.read_fractal_mem(i);
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
    hand_mem.write_fixed(args[2], idx);
    None
  });
  cpu!("join", |args, hand_mem, _| {
    unsafe {
      let sep_pascal_string = hand_mem.read_fractal_mem(args[1]);
      let sep_str_len = sep_pascal_string[0] as usize;
      let sep_pascal_string_u8 = slice::from_raw_parts(sep_pascal_string[1..].as_ptr().cast::<u8>(), sep_str_len*8);
      let sep_str = str::from_utf8(&sep_pascal_string_u8[0..sep_str_len]).unwrap();
      let mem = hand_mem.get_fractal(args[0]);
      let len = mem.len() as i64;
      let mut strs: Vec<String> = Vec::new();
      for i in 0..len {
        let v_pascal_string = mem.read_fractal_mem(i);
        let v_str_len = v_pascal_string[0] as usize;
        let v_pascal_string_u8 = slice::from_raw_parts(v_pascal_string[1..].as_ptr().cast::<u8>(), v_str_len*8);
        let v_str = str::from_utf8(&v_pascal_string_u8[0..v_str_len]).unwrap();
        strs.push(v_str.to_string());
      }
      let out_str = strs.join(sep_str);
      let mut out = vec![out_str.len() as i64];
      let mut out_str_bytes = out_str.as_bytes().to_vec();
      loop {
        if out_str_bytes.len() % 8 != 0 {
          out_str_bytes.push(0);
        } else {
          break
        }
      }
      let mut i = 0;
      loop {
        if i < out_str_bytes.len() {
          let str_slice = &out_str_bytes[i..i+8];
          out.push(i64::from_ne_bytes(str_slice.try_into().unwrap()));
          i = i + 8;
        } else {
          break
        }
      }
      hand_mem.write_fractal_mem(args[2], &out);
    }
    None
  });
  cpu!("pusharr", |args, hand_mem, _| {
    let val_size = hand_mem.read_fixed(args[2]);
    if val_size == 0 {
      let val = hand_mem.read_fractal(args[1]);
      hand_mem.push_fractal_fractal(args[0], val);
    } else {
      let val = hand_mem.read_fixed(args[1]);
      hand_mem.push_fractal_fixed(args[0], val);
    }
    None
  });
  cpu!("poparr", |args, hand_mem, _| {
    let last = hand_mem.pop_fractal(args[0]);
    if last.is_ok() {
      hand_mem.write_fixed(args[1], last.ok().unwrap());
    } else {
      hand_mem.write_fractal(args[1], last.err().unwrap());
    }
    None
  });
  cpu!("newarr", |args, hand_mem, _| {
    hand_mem.new_fractal(args[2]);
    None
  });
  // Map opcodes TODO after maps are implemented

  // Ternary opcodes
  unpred_cpu!("condfn", |args, hand_mem, frag| {
    let cond = hand_mem.read_fixed(args[0]);
    let event_id = args[1];
    if cond == 1 {
      frag.insert_subhandler(event_id);
    }
    None
  });

  // Std opcodes
  unpred_cpu!("execop", |args, hand_mem, _| {
    unsafe {
      let pascal_string = hand_mem.read_fractal_mem(args[0]);
      let str_len = pascal_string[0] as usize;
      let full_cmd = str::from_utf8(
        &std::mem::transmute::<&[i64], &[u8]>(
          &pascal_string[1..]
        )[0..str_len]
      ).unwrap();
      let split_cmd: Vec<&str> = full_cmd.split(" ").collect();
      let output = Command::new(split_cmd[0]).args(&split_cmd[1..]).output();
      match output {
        Err(e) => println!("Executing \"{}\" failed with: {}", full_cmd, e),
        Ok(out) => {
          io::stdout().write_all(&out.stdout).unwrap();
          io::stderr().write_all(&out.stderr).unwrap();
        },
      };
    }
    None
  });

  // "Special" opcodes
  io!("waitop", |args, hand_mem, _| {
    let ms = hand_mem.read_fixed(args[0]) as u64;
    return Box::pin(delay_for(Duration::from_millis(ms)));
  });
  cpu!("exitop", |args, hand_mem, _| {
    std::process::exit(hand_mem.read_fixed(args[0]) as i32);
  });
  cpu!("stdoutp", |args, hand_mem, _| {
    unsafe {
      let pascal_string = hand_mem.read_fractal_mem(args[0]);
      let str_len = pascal_string[0] as usize;
      let pascal_string_u8 = slice::from_raw_parts(pascal_string[1..].as_ptr().cast::<u8>(), str_len*8);
      let out_str = str::from_utf8(&pascal_string_u8[0..str_len]).unwrap();
      print!("{}", out_str);
    }
    None
  });
  // set opcodes use args[0] directly, since the relevant value directly
  // fits in i64, and write it to args[2]
  cpu!("seti64", |args, hand_mem, _| {
    let data = args[0];
    hand_mem.write_fixed(args[2], data);
    None
  });
  cpu!("seti32", |args, hand_mem, _| {
    let data = (args[0] as i32) as i64;
    hand_mem.write_fixed(args[2], data);
    None
  });
  cpu!("seti16", |args, hand_mem, _| {
    let data = (args[0] as i16) as i64;
    hand_mem.write_fixed(args[2], data);
    None
  });
  cpu!("seti8", |args, hand_mem, _| {
    let data = (args[0] as i8) as i64;
    hand_mem.write_fixed(args[2], data);
    None
  });
  cpu!("setf64", |args, hand_mem, _| {
    let data = i64::from_ne_bytes((args[0] as f64).to_ne_bytes());
    hand_mem.write_fixed(args[2], data);
    None
  });
  cpu!("setf32", |args, hand_mem, _| {
    let data = i32::from_ne_bytes((args[0] as f32).to_ne_bytes()) as i64;
    hand_mem.write_fixed(args[2], data);
    None
  });
  cpu!("setbool", |args, hand_mem, _| {
    let data = if args[0] == 0 { 0i64 } else { 1i64 };
    hand_mem.write_fixed(args[2], data);
    None
  });
  cpu!("setestr", |args, hand_mem, _| {
    let empty_str = vec![0];
    hand_mem.write_fractal_mem(args[2], &empty_str);
    None
  });

  // copy opcodes used for let variable reassignments
  cpu!("copyi8", |args, hand_mem, _| {
    let val = hand_mem.read_fixed(args[0]);
    hand_mem.write_fixed(args[2], val);
    None
  });
  cpu!("copyi16", |args, hand_mem, _| {
    let val = hand_mem.read_fixed(args[0]);
    hand_mem.write_fixed(args[2], val);
    None
  });
  cpu!("copyi32", |args, hand_mem, _| {
    let val = hand_mem.read_fixed(args[0]);
    hand_mem.write_fixed(args[2], val);
    None
  });
  cpu!("copyi64", |args, hand_mem, _| {
    let val = hand_mem.read_fixed(args[0]);
    hand_mem.write_fixed(args[2], val);
    None
  });
  cpu!("copyf32", |args, hand_mem, _| {
    let val = hand_mem.read_fixed(args[0]);
    hand_mem.write_fixed(args[2], val);
    None
  });
  cpu!("copyf64", |args, hand_mem, _| {
    let val = hand_mem.read_fixed(args[0]);
    hand_mem.write_fixed(args[2], val);
    None
  });
  cpu!("copybool", |args, hand_mem, _| {
    let val = hand_mem.read_fixed(args[0]);
    hand_mem.write_fixed(args[2], val);
    None
  });
  cpu!("copystr", |args, hand_mem, _| {
    let pascal_string = hand_mem.read_fractal_mem(args[0]);
    hand_mem.write_fractal_mem(args[2], &pascal_string);
    None
  });
  cpu!("copyarr", |args, hand_mem, _| {
    // args = [in_addr, unused, out_addr]
    hand_mem.copy_fractal(args[0], args[2]);
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
