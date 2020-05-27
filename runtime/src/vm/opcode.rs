use std::collections::HashMap;
use std::convert::TryInto;
use std::future::Future;
use std::pin::Pin;
use std::str;
use std::time::Duration;

use byteorder::{ByteOrder, LittleEndian};
use once_cell::sync::Lazy;
use tokio::time::delay_for;

use crate::vm::event::{EventEmit, HandlerFragment};
use crate::vm::memory::MemoryFragment;

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
  &mut MemoryFragment,
  &'static HashMap<i64, i64>,
  &mut HandlerFragment
) -> EmptyFuture;
/// Function pointer for cpu bound opcodes
type FnPtr = fn(
  &Vec<i64>,
  &mut MemoryFragment,
  &'static HashMap<i64, i64>,
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
        pred_exec: false,
        func: Some($func),
        async_func: None,
      };
      o.insert(id, opcode);
    };
  }

  // Type conversion opcodes
  cpu!("i8f64", |args, mem_frag, _, _| {
    let a = i8::from_le_bytes(mem_frag.read(args[0], 1).try_into().unwrap());
    let out = a as f64;
    mem_frag.write(args[2], 8, &out.to_le_bytes());
    None
  });
  cpu!("i16f64", |args, mem_frag, _, _| {
    let a = LittleEndian::read_i16(mem_frag.read(args[0], 2));
    let out = a as f64;
    mem_frag.write(args[2], 8, &out.to_le_bytes());
    None
  });
  cpu!("i32f64", |args, mem_frag, _, _| {
    let a = LittleEndian::read_i32(mem_frag.read(args[0], 4));
    let out = a as f64;
    mem_frag.write(args[2], 8, &out.to_le_bytes());
    None
  });
  cpu!("i64f64", |args, mem_frag, _, _| {
    let a = LittleEndian::read_i64(mem_frag.read(args[0], 8));
    let out = a as f64;
    mem_frag.write(args[2], 8, &out.to_le_bytes());
    None
  });
  cpu!("f32f64", |args, mem_frag, _, _| {
    let a = LittleEndian::read_f32(mem_frag.read(args[0], 4));
    let out = a as f64;
    mem_frag.write(args[2], 8, &out.to_le_bytes());
    None
  });
  cpu!("strf64", |args, mem_frag, _, _| {
    let pascal_string = mem_frag.read(args[0], 0);
    let size = LittleEndian::read_u64(&pascal_string[0..8]) as usize;
    let out_str = str::from_utf8(&pascal_string[8..size + 8]).unwrap();
    let out: f64 = out_str.parse().unwrap();
    mem_frag.write(args[2], 8, &out.to_le_bytes());
    None
  });
  cpu!("boolf64", |args, mem_frag, _, _| {
    let a = mem_frag.read(args[0], 1)[0];
    let out = a as f64; // This works because bools are 0 or 1 internally
    mem_frag.write(args[2], 8, &out.to_le_bytes());
    None
  });

  cpu!("i8f32", |args, mem_frag, _, _| {
    let a = i8::from_le_bytes(mem_frag.read(args[0], 1).try_into().unwrap());
    let out = a as f32;
    mem_frag.write(args[2], 4, &out.to_le_bytes());
    None
  });
  cpu!("i16f32", |args, mem_frag, _, _| {
    let a = LittleEndian::read_i16(mem_frag.read(args[0], 2));
    let out = a as f32;
    mem_frag.write(args[2], 4, &out.to_le_bytes());
    None
  });
  cpu!("i32f32", |args, mem_frag, _, _| {
    let a = LittleEndian::read_i32(mem_frag.read(args[0], 4));
    let out = a as f32;
    mem_frag.write(args[2], 4, &out.to_le_bytes());
    None
  });
  cpu!("i64f32", |args, mem_frag, _, _| {
    let a = LittleEndian::read_i64(mem_frag.read(args[0], 8));
    let out = a as f32;
    mem_frag.write(args[2], 4, &out.to_le_bytes());
    None
  });
  cpu!("f64f32", |args, mem_frag, _, _| {
    let a = LittleEndian::read_f64(mem_frag.read(args[0], 8));
    let out = a as f32;
    mem_frag.write(args[2], 4, &out.to_le_bytes());
    None
  });
  cpu!("strf32", |args, mem_frag, _, _| {
    let pascal_string = mem_frag.read(args[0], 0);
    let size = LittleEndian::read_u64(&pascal_string[0..8]) as usize;
    let out_str = str::from_utf8(&pascal_string[8..size + 8]).unwrap();
    let out: f32 = out_str.parse().unwrap();
    mem_frag.write(args[2], 4, &out.to_le_bytes());
    None
  });
  cpu!("boolf32", |args, mem_frag, _, _| {
    let a = mem_frag.read(args[0], 1)[0];
    let out = a as f32; // This works because bools are 0 or 1 internally
    mem_frag.write(args[2], 4, &out.to_le_bytes());
    None
  });

  cpu!("i8i64", |args, mem_frag, _, _| {
    let a = i8::from_le_bytes(mem_frag.read(args[0], 1).try_into().unwrap());
    let out = a as i64;
    mem_frag.write(args[2], 8, &out.to_le_bytes());
    None
  });
  cpu!("i16i64", |args, mem_frag, _, _| {
    let a = LittleEndian::read_i16(mem_frag.read(args[0], 2));
    let out = a as i64;
    mem_frag.write(args[2], 8, &out.to_le_bytes());
    None
  });
  cpu!("i32i64", |args, mem_frag, _, _| {
    let a = LittleEndian::read_i32(mem_frag.read(args[0], 4));
    let out = a as i64;
    mem_frag.write(args[2], 8, &out.to_le_bytes());
    None
  });
  cpu!("f64i64", |args, mem_frag, _, _| {
    let a = LittleEndian::read_f64(mem_frag.read(args[0], 8));
    let out = a as i64;
    mem_frag.write(args[2], 8, &out.to_le_bytes());
    None
  });
  cpu!("f32i64", |args, mem_frag, _, _| {
    let a = LittleEndian::read_f32(mem_frag.read(args[0], 4));
    let out = a as i64;
    mem_frag.write(args[2], 8, &out.to_le_bytes());
    None
  });
  cpu!("stri64", |args, mem_frag, _, _| {
    let pascal_string = mem_frag.read(args[0], 0);
    let size = LittleEndian::read_u64(&pascal_string[0..8]) as usize;
    let out_str = str::from_utf8(&pascal_string[8..size + 8]).unwrap();
    let out: i64 = out_str.parse().unwrap();
    mem_frag.write(args[2], 8, &out.to_le_bytes());
    None
  });
  cpu!("booli64", |args, mem_frag, _, _| {
    let a = mem_frag.read(args[0], 1)[0];
    let out = a as i64; // This works because bools are 0 or 1 internally
    mem_frag.write(args[2], 8, &out.to_le_bytes());
    None
  });

  cpu!("i8i32", |args, mem_frag, _, _| {
    let a = i8::from_le_bytes(mem_frag.read(args[0], 1).try_into().unwrap());
    let out = a as i32;
    mem_frag.write(args[2], 4, &out.to_le_bytes());
    None
  });
  cpu!("i16i32", |args, mem_frag, _, _| {
    let a = LittleEndian::read_i16(mem_frag.read(args[0], 2));
    let out = a as i32;
    mem_frag.write(args[2], 4, &out.to_le_bytes());
    None
  });
  cpu!("i64i32", |args, mem_frag, _, _| {
    let a = LittleEndian::read_i64(mem_frag.read(args[0], 8));
    let out = a as i32;
    mem_frag.write(args[2], 4, &out.to_le_bytes());
    None
  });
  cpu!("f64i32", |args, mem_frag, _, _| {
    let a = LittleEndian::read_f64(mem_frag.read(args[0], 8));
    let out = a as i32;
    mem_frag.write(args[2], 4, &out.to_le_bytes());
    None
  });
  cpu!("f32i32", |args, mem_frag, _, _| {
    let a = LittleEndian::read_f32(mem_frag.read(args[0], 4));
    let out = a as i32;
    mem_frag.write(args[2], 4, &out.to_le_bytes());
    None
  });
  cpu!("stri32", |args, mem_frag, _, _| {
    let pascal_string = mem_frag.read(args[0], 0);
    let size = LittleEndian::read_u64(&pascal_string[0..8]) as usize;
    let out_str = str::from_utf8(&pascal_string[8..size + 8]).unwrap();
    let out: i32 = out_str.parse().unwrap();
    mem_frag.write(args[2], 4, &out.to_le_bytes());
    None
  });
  cpu!("booli32", |args, mem_frag, _, _| {
    let a = mem_frag.read(args[0], 1)[0];
    let out = a as i32; // This works because bools are 0 or 1 internally
    mem_frag.write(args[2], 4, &out.to_le_bytes());
    None
  });

  cpu!("i8i16", |args, mem_frag, _, _| {
    let a = i8::from_le_bytes(mem_frag.read(args[0], 1).try_into().unwrap());
    let out = a as i16;
    mem_frag.write(args[2], 2, &out.to_le_bytes());
    None
  });
  cpu!("i32i16", |args, mem_frag, _, _| {
    let a = LittleEndian::read_i32(mem_frag.read(args[0], 4));
    let out = a as i16;
    mem_frag.write(args[2], 2, &out.to_le_bytes());
    None
  });
  cpu!("i64i16", |args, mem_frag, _, _| {
    let a = LittleEndian::read_i64(mem_frag.read(args[0], 8));
    let out = a as i16;
    mem_frag.write(args[2], 2, &out.to_le_bytes());
    None
  });
  cpu!("f64i16", |args, mem_frag, _, _| {
    let a = LittleEndian::read_f64(mem_frag.read(args[0], 8));
    let out = a as i16;
    mem_frag.write(args[2], 2, &out.to_le_bytes());
    None
  });
  cpu!("f32i16", |args, mem_frag, _, _| {
    let a = LittleEndian::read_f32(mem_frag.read(args[0], 4));
    let out = a as i16;
    mem_frag.write(args[2], 2, &out.to_le_bytes());
    None
  });
  cpu!("stri16", |args, mem_frag, _, _| {
    let pascal_string = mem_frag.read(args[0], 0);
    let size = LittleEndian::read_u64(&pascal_string[0..8]) as usize;
    let out_str = str::from_utf8(&pascal_string[8..size + 8]).unwrap();
    let out: i16 = out_str.parse().unwrap();
    mem_frag.write(args[2], 2, &out.to_le_bytes());
    None
  });
  cpu!("booli16", |args, mem_frag, _, _| {
    let a = mem_frag.read(args[0], 1)[0];
    let out = a as i16; // This works because bools are 0 or 1 internally
    mem_frag.write(args[2], 2, &out.to_le_bytes());
    None
  });

  cpu!("i16i8", |args, mem_frag, _, _| {
    let a = LittleEndian::read_i16(mem_frag.read(args[0], 2));
    let out = a as i8;
    mem_frag.write(args[2], 1, &out.to_le_bytes());
    None
  });
  cpu!("i32i8", |args, mem_frag, _, _| {
    let a = LittleEndian::read_i32(mem_frag.read(args[0], 4));
    let out = a as i8;
    mem_frag.write(args[2], 1, &out.to_le_bytes());
    None
  });
  cpu!("i64i8", |args, mem_frag, _, _| {
    let a = LittleEndian::read_i64(mem_frag.read(args[0], 8));
    let out = a as i8;
    mem_frag.write(args[2], 1, &out.to_le_bytes());
    None
  });
  cpu!("f64i8", |args, mem_frag, _, _| {
    let a = LittleEndian::read_f64(mem_frag.read(args[0], 8));
    let out = a as i8;
    mem_frag.write(args[2], 1, &out.to_le_bytes());
    None
  });
  cpu!("f32i8", |args, mem_frag, _, _| {
    let a = LittleEndian::read_f32(mem_frag.read(args[0], 4));
    let out = a as i8;
    mem_frag.write(args[2], 1, &out.to_le_bytes());
    None
  });
  cpu!("stri8", |args, mem_frag, _, _| {
    let pascal_string = mem_frag.read(args[0], 0);
    let size = LittleEndian::read_u64(&pascal_string[0..8]) as usize;
    let out_str = str::from_utf8(&pascal_string[8..size + 8]).unwrap();
    let out: i8 = out_str.parse().unwrap();
    mem_frag.write(args[2], 1, &out.to_le_bytes());
    None
  });
  cpu!("booli8", |args, mem_frag, _, _| {
    let a = mem_frag.read(args[0], 1)[0];
    let out = a as i8; // This works because bools are 0 or 1 internally
    mem_frag.write(args[2], 1, &out.to_le_bytes());
    None
  });

  cpu!("i8bool", |args, mem_frag, _, _| {
    let a = i8::from_le_bytes(mem_frag.read(args[0], 1).try_into().unwrap());
    let out = if a != 0 { 1u8 } else { 0u8 };
    mem_frag.write(args[2], 1, &out.to_le_bytes());
    None
  });
  cpu!("i16bool", |args, mem_frag, _, _| {
    let a = LittleEndian::read_i16(mem_frag.read(args[0], 2));
    let out = if a != 0 { 1u8 } else { 0u8 };
    mem_frag.write(args[2], 1, &out.to_le_bytes());
    None
  });
  cpu!("i32bool", |args, mem_frag, _, _| {
    let a = LittleEndian::read_i32(mem_frag.read(args[0], 4));
    let out = if a != 0 { 1u8 } else { 0u8 };
    mem_frag.write(args[2], 1, &out.to_le_bytes());
    None
  });
  cpu!("i64bool", |args, mem_frag, _, _| {
    let a = LittleEndian::read_i64(mem_frag.read(args[0], 8));
    let out = if a != 0 { 1u8 } else { 0u8 };
    mem_frag.write(args[2], 1, &out.to_le_bytes());
    None
  });
  cpu!("f64bool", |args, mem_frag, _, _| {
    let a = LittleEndian::read_f64(mem_frag.read(args[0], 8));
    let out = if a != 0.0 { 1u8 } else { 0u8 };
    mem_frag.write(args[2], 1, &out.to_le_bytes());
    None
  });
  cpu!("f32bool", |args, mem_frag, _, _| {
    let a = LittleEndian::read_f32(mem_frag.read(args[0], 4));
    let out = if a != 0.0 { 1u8 } else { 0u8 };
    mem_frag.write(args[2], 1, &out.to_le_bytes());
    None
  });
  cpu!("strbool", |args, mem_frag, _, _| {
    let pascal_string = mem_frag.read(args[0], 0);
    let size = LittleEndian::read_u64(&pascal_string[0..8]) as usize;
    let a_str = str::from_utf8(&pascal_string[8..size + 8]).unwrap();
    let out = if a_str == "true" { 1u8 } else { 0u8 };
    mem_frag.write(args[2], 1, &out.to_le_bytes());
    None
  });

  cpu!("i8str", |args, mem_frag, _, _| {
    let a = i8::from_le_bytes(mem_frag.read(args[0], 1).try_into().unwrap());
    let a_str = a.to_string();
    let mut out = (a_str.len() as u64).to_le_bytes().to_vec();
    out.append(&mut a_str.as_bytes().to_vec());
    mem_frag.write(args[2], 0, &out);
    None
  });
  cpu!("i16str", |args, mem_frag, _, _| {
    let a = LittleEndian::read_i16(mem_frag.read(args[0], 2));
    let a_str = a.to_string();
    let mut out = (a_str.len() as u64).to_le_bytes().to_vec();
    out.append(&mut a_str.as_bytes().to_vec());
    mem_frag.write(args[2], 0, &out);
    None
  });
  cpu!("i32str", |args, mem_frag, _, _| {
    let a = LittleEndian::read_i32(mem_frag.read(args[0], 4));
    let a_str = a.to_string();
    let mut out = (a_str.len() as u64).to_le_bytes().to_vec();
    out.append(&mut a_str.as_bytes().to_vec());
    mem_frag.write(args[2], 0, &out);
    None
  });
  cpu!("i64str", |args, mem_frag, _, _| {
    let a = LittleEndian::read_i64(mem_frag.read(args[0], 8));
    let a_str = a.to_string();
    let mut out = (a_str.len() as u64).to_le_bytes().to_vec();
    out.append(&mut a_str.as_bytes().to_vec());
    mem_frag.write(args[2], 0, &out);
    None
  });
  cpu!("f64str", |args, mem_frag, _, _| {
    let a = LittleEndian::read_f64(mem_frag.read(args[0], 8));
    let a_str = a.to_string();
    let mut out = (a_str.len() as u64).to_le_bytes().to_vec();
    out.append(&mut a_str.as_bytes().to_vec());
    mem_frag.write(args[2], 0, &out);
    None
  });
  cpu!("f32str", |args, mem_frag, _, _| {
    let a = LittleEndian::read_f32(mem_frag.read(args[0], 4));
    let a_str = a.to_string();
    let mut out = (a_str.len() as u64).to_le_bytes().to_vec();
    out.append(&mut a_str.as_bytes().to_vec());
    mem_frag.write(args[2], 0, &out);
    None
  });
  cpu!("boolstr", |args, mem_frag, _, _| {
    let a = mem_frag.read(args[0], 1)[0];
    let a_str = a.to_string();
    let mut out = (a_str.len() as u64).to_le_bytes().to_vec();
    out.append(&mut a_str.as_bytes().to_vec());
    mem_frag.write(args[2], 0, &out);
    None
  });

  // Arithmetic opcodes
  cpu!("addi8", |args, mem_frag, _, _| {
    let a = i8::from_le_bytes(mem_frag.read(args[0], 1).try_into().unwrap());
    let b = i8::from_le_bytes(mem_frag.read(args[1], 1).try_into().unwrap());
    let out = a + b;
    mem_frag.write(args[2], 1, &out.to_le_bytes());
    None
  });
  cpu!("addi16", |args, mem_frag, _, _| {
    let a = LittleEndian::read_i16(mem_frag.read(args[0], 2));
    let b = LittleEndian::read_i16(mem_frag.read(args[1], 2));
    let out = a + b;
    mem_frag.write(args[2], 2, &out.to_le_bytes());
    None
  });
  cpu!("addi32", |args, mem_frag, _, _| {
    let a = LittleEndian::read_i32(mem_frag.read(args[0], 4));
    let b = LittleEndian::read_i32(mem_frag.read(args[1], 4));
    let out = a + b;
    mem_frag.write(args[2], 4, &out.to_le_bytes());
    None
  });
  cpu!("addi64", |args, mem_frag, _, _| {
    let a = LittleEndian::read_i64(mem_frag.read(args[0], 8));
    let b = LittleEndian::read_i64(mem_frag.read(args[1], 8));
    let out = a + b;
    mem_frag.write(args[2], 8, &out.to_le_bytes());
    None
  });
  cpu!("addf32", |args, mem_frag, _, _| {
    let a = LittleEndian::read_f32(mem_frag.read(args[0], 4));
    let b = LittleEndian::read_f32(mem_frag.read(args[1], 4));
    let out = a + b;
    mem_frag.write(args[2], 4, &out.to_le_bytes());
    None
  });
  cpu!("addf64", |args, mem_frag, _, _| {
    let a = LittleEndian::read_f64(mem_frag.read(args[0], 8));
    let b = LittleEndian::read_f64(mem_frag.read(args[1], 8));
    let out = a + b;
    mem_frag.write(args[2], 8, &out.to_le_bytes());
    None
  });

  cpu!("subi8", |args, mem_frag, _, _| {
    let a = i8::from_le_bytes(mem_frag.read(args[0], 1).try_into().unwrap());
    let b = i8::from_le_bytes(mem_frag.read(args[1], 1).try_into().unwrap());
    let out = a - b;
    mem_frag.write(args[2], 1, &out.to_le_bytes());
    None
  });
  cpu!("subi16", |args, mem_frag, _, _| {
    let a = LittleEndian::read_i16(mem_frag.read(args[0], 2));
    let b = LittleEndian::read_i16(mem_frag.read(args[1], 2));
    let out = a - b;
    mem_frag.write(args[2], 2, &out.to_le_bytes());
    None
  });
  cpu!("subi32", |args, mem_frag, _, _| {
    let a = LittleEndian::read_i32(mem_frag.read(args[0], 4));
    let b = LittleEndian::read_i32(mem_frag.read(args[1], 4));
    let out = a - b;
    mem_frag.write(args[2], 4, &out.to_le_bytes());
    None
  });
  cpu!("subi64", |args, mem_frag, _, _| {
    let a = LittleEndian::read_i64(mem_frag.read(args[0], 8));
    let b = LittleEndian::read_i64(mem_frag.read(args[1], 8));
    let out = a - b;
    mem_frag.write(args[2], 8, &out.to_le_bytes());
    None
  });
  cpu!("subf32", |args, mem_frag, _, _| {
    let a = LittleEndian::read_f32(mem_frag.read(args[0], 4));
    let b = LittleEndian::read_f32(mem_frag.read(args[1], 4));
    let out = a - b;
    mem_frag.write(args[2], 4, &out.to_le_bytes());
    None
  });
  cpu!("subf64", |args, mem_frag, _, _| {
    let a = LittleEndian::read_f64(mem_frag.read(args[0], 8));
    let b = LittleEndian::read_f64(mem_frag.read(args[1], 8));
    let out = a - b;
    mem_frag.write(args[2], 8, &out.to_le_bytes());
    None
  });

  cpu!("negi8", |args, mem_frag, _, _| {
    let a = i8::from_le_bytes(mem_frag.read(args[0], 1).try_into().unwrap());
    let out = 0 - a;
    mem_frag.write(args[2], 1, &out.to_le_bytes());
    None
  });
  cpu!("negi16", |args, mem_frag, _, _| {
    let a = LittleEndian::read_i16(mem_frag.read(args[0], 2));
    let out = 0 - a;
    mem_frag.write(args[2], 2, &out.to_le_bytes());
    None
  });
  cpu!("negi32", |args, mem_frag, _, _| {
    let a = LittleEndian::read_i32(mem_frag.read(args[0], 4));
    let out = 0 - a;
    mem_frag.write(args[2], 4, &out.to_le_bytes());
    None
  });
  cpu!("negi64", |args, mem_frag, _, _| {
    let a = LittleEndian::read_i64(mem_frag.read(args[0], 8));
    let out = 0 - a;
    mem_frag.write(args[2], 8, &out.to_le_bytes());
    None
  });
  cpu!("negf32", |args, mem_frag, _, _| {
    let a = LittleEndian::read_f32(mem_frag.read(args[0], 4));
    let out = 0.0 - a;
    mem_frag.write(args[2], 4, &out.to_le_bytes());
    None
  });
  cpu!("negf64", |args, mem_frag, _, _| {
    let a = LittleEndian::read_f64(mem_frag.read(args[0], 8));
    let out = 0.0 - a;
    mem_frag.write(args[2], 8, &out.to_le_bytes());
    None
  });

  cpu!("muli8", |args, mem_frag, _, _| {
    let a = i8::from_le_bytes(mem_frag.read(args[0], 1).try_into().unwrap());
    let b = i8::from_le_bytes(mem_frag.read(args[1], 1).try_into().unwrap());
    let out = a * b;
    mem_frag.write(args[2], 1, &out.to_le_bytes());
    None
  });
  cpu!("muli16", |args, mem_frag, _, _| {
    let a = LittleEndian::read_i16(mem_frag.read(args[0], 2));
    let b = LittleEndian::read_i16(mem_frag.read(args[1], 2));
    let out = a * b;
    mem_frag.write(args[2], 2, &out.to_le_bytes());
    None
  });
  cpu!("muli32", |args, mem_frag, _, _| {
    let a = LittleEndian::read_i32(mem_frag.read(args[0], 4));
    let b = LittleEndian::read_i32(mem_frag.read(args[1], 4));
    let out = a * b;
    mem_frag.write(args[2], 4, &out.to_le_bytes());
    None
  });
  cpu!("muli64", |args, mem_frag, _, _| {
    let a = LittleEndian::read_i64(mem_frag.read(args[0], 8));
    let b = LittleEndian::read_i64(mem_frag.read(args[1], 8));
    let out = a * b;
    mem_frag.write(args[2], 8, &out.to_le_bytes());
    None
  });
  cpu!("mulf32", |args, mem_frag, _, _| {
    let a = LittleEndian::read_f32(mem_frag.read(args[0], 4));
    let b = LittleEndian::read_f32(mem_frag.read(args[1], 4));
    let out = a * b;
    mem_frag.write(args[2], 4, &out.to_le_bytes());
    None
  });
  cpu!("mulf64", |args, mem_frag, _, _| {
    let a = LittleEndian::read_f64(mem_frag.read(args[0], 8));
    let b = LittleEndian::read_f64(mem_frag.read(args[1], 8));
    let out = a * b;
    mem_frag.write(args[2], 8, &out.to_le_bytes());
    None
  });

  cpu!("divi8", |args, mem_frag, _, _| {
    let a = i8::from_le_bytes(mem_frag.read(args[0], 1).try_into().unwrap());
    let b = i8::from_le_bytes(mem_frag.read(args[1], 1).try_into().unwrap());
    let out = a / b;
    mem_frag.write(args[2], 1, &out.to_le_bytes());
    None
  });
  cpu!("divi16", |args, mem_frag, _, _| {
    let a = LittleEndian::read_i16(mem_frag.read(args[0], 2));
    let b = LittleEndian::read_i16(mem_frag.read(args[1], 2));
    let out = a / b;
    mem_frag.write(args[2], 2, &out.to_le_bytes());
    None
  });
  cpu!("divi32", |args, mem_frag, _, _| {
    let a = LittleEndian::read_i32(mem_frag.read(args[0], 4));
    let b = LittleEndian::read_i32(mem_frag.read(args[1], 4));
    let out = a / b;
    mem_frag.write(args[2], 4, &out.to_le_bytes());
    None
  });
  cpu!("divi64", |args, mem_frag, _, _| {
    let a = LittleEndian::read_i64(mem_frag.read(args[0], 8));
    let b = LittleEndian::read_i64(mem_frag.read(args[1], 8));
    let out = a / b;
    mem_frag.write(args[2], 8, &out.to_le_bytes());
    None
  });
  cpu!("divf32", |args, mem_frag, _, _| {
    let a = LittleEndian::read_f32(mem_frag.read(args[0], 4));
    let b = LittleEndian::read_f32(mem_frag.read(args[1], 4));
    let out = a / b;
    mem_frag.write(args[2], 4, &out.to_le_bytes());
    None
  });
  cpu!("divf64", |args, mem_frag, _, _| {
    let a = LittleEndian::read_f64(mem_frag.read(args[0], 8));
    let b = LittleEndian::read_f64(mem_frag.read(args[1], 8));
    let out = a / b;
    mem_frag.write(args[2], 8, &out.to_le_bytes());
    None
  });

  cpu!("modi8", |args, mem_frag, _, _| {
    let a = i8::from_le_bytes(mem_frag.read(args[0], 1).try_into().unwrap());
    let b = i8::from_le_bytes(mem_frag.read(args[1], 1).try_into().unwrap());
    let out = a % b;
    mem_frag.write(args[2], 1, &out.to_le_bytes());
    None
  });
  cpu!("modi16", |args, mem_frag, _, _| {
    let a = LittleEndian::read_i16(mem_frag.read(args[0], 2));
    let b = LittleEndian::read_i16(mem_frag.read(args[1], 2));
    let out = a % b;
    mem_frag.write(args[2], 2, &out.to_le_bytes());
    None
  });
  cpu!("modi32", |args, mem_frag, _, _| {
    let a = LittleEndian::read_i32(mem_frag.read(args[0], 4));
    let b = LittleEndian::read_i32(mem_frag.read(args[1], 4));
    let out = a % b;
    mem_frag.write(args[2], 4, &out.to_le_bytes());
    None
  });
  cpu!("modi64", |args, mem_frag, _, _| {
    let a = LittleEndian::read_i64(mem_frag.read(args[0], 8));
    let b = LittleEndian::read_i64(mem_frag.read(args[1], 8));
    let out = a % b;
    mem_frag.write(args[2], 8, &out.to_le_bytes());
    None
  });

  cpu!("powi8", |args, mem_frag, _, _| {
    let a = i8::from_le_bytes(mem_frag.read(args[0], 1).try_into().unwrap());
    let b = i8::from_le_bytes(mem_frag.read(args[1], 1).try_into().unwrap());
    let out = if b < 0 { 0i8 } else { i8::pow(a, b as u32) };
    mem_frag.write(args[2], 1, &out.to_le_bytes());
    None
  });
  cpu!("powi16", |args, mem_frag, _, _| {
    let a = LittleEndian::read_i16(mem_frag.read(args[0], 2));
    let b = LittleEndian::read_i16(mem_frag.read(args[1], 2));
    let out = if b < 0 { 0i16 } else { i16::pow(a, b as u32) };
    mem_frag.write(args[2], 2, &out.to_le_bytes());
    None
  });
  cpu!("powi32", |args, mem_frag, _, _| {
    let a = LittleEndian::read_i32(mem_frag.read(args[0], 4));
    let b = LittleEndian::read_i32(mem_frag.read(args[1], 4));
    let out = if b < 0 { 0i32 } else { i32::pow(a, b as u32) };
    mem_frag.write(args[2], 4, &out.to_le_bytes());
    None
  });
  cpu!("powi64", |args, mem_frag, _, _| {
    // The inputs may be from local memory or global
    let a = LittleEndian::read_i64(mem_frag.read(args[0], 8));
    let b = LittleEndian::read_i64(mem_frag.read(args[1], 8));
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
    mem_frag.write(args[2], 8, &out.to_le_bytes());
    None
  });
  cpu!("powf32", |args, mem_frag, _, _| {
    let a = LittleEndian::read_f32(mem_frag.read(args[0], 4));
    let b = LittleEndian::read_f32(mem_frag.read(args[1], 4));
    let out = f32::powf(a, b);
    mem_frag.write(args[2], 4, &out.to_le_bytes());
    None
  });
  cpu!("powf64", |args, mem_frag, _, _| {
    let a = LittleEndian::read_f64(mem_frag.read(args[0], 8));
    let b = LittleEndian::read_f64(mem_frag.read(args[1], 8));
    let out = f64::powf(a, b);
    mem_frag.write(args[2], 8, &out.to_le_bytes());
    None
  });

  cpu!("sqrtf32", |args, mem_frag, _, _| {
    let a = LittleEndian::read_f32(mem_frag.read(args[0], 4));
    let out = f32::sqrt(a);
    mem_frag.write(args[2], 4, &out.to_le_bytes());
    None
  });
  cpu!("sqrtf64", |args, mem_frag, _, _| {
    let a = LittleEndian::read_f64(mem_frag.read(args[0], 8));
    let out = f64::sqrt(a);
    mem_frag.write(args[2], 8, &out.to_le_bytes());
    None
  });

  // Boolean and bitwise opcodes
  cpu!("andi8", |args, mem_frag, _, _| {
    let a = i8::from_le_bytes(mem_frag.read(args[0], 1).try_into().unwrap());
    let b = i8::from_le_bytes(mem_frag.read(args[1], 1).try_into().unwrap());
    let out = a & b;
    mem_frag.write(args[2], 1, &out.to_le_bytes());
    None
  });
  cpu!("andi16", |args, mem_frag, _, _| {
    let a = LittleEndian::read_i16(mem_frag.read(args[0], 2));
    let b = LittleEndian::read_i16(mem_frag.read(args[1], 2));
    let out = a & b;
    mem_frag.write(args[2], 2, &out.to_le_bytes());
    None
  });
  cpu!("andi32", |args, mem_frag, _, _| {
    let a = LittleEndian::read_i32(mem_frag.read(args[0], 4));
    let b = LittleEndian::read_i32(mem_frag.read(args[1], 4));
    let out = a & b;
    mem_frag.write(args[2], 4, &out.to_le_bytes());
    None
  });
  cpu!("andi64", |args, mem_frag, _, _| {
    let a = LittleEndian::read_i64(mem_frag.read(args[0], 8));
    let b = LittleEndian::read_i64(mem_frag.read(args[1], 8));
    let out = a & b;
    mem_frag.write(args[2], 8, &out.to_le_bytes());
    None
  });
  cpu!("andbool", |args, mem_frag, _, _| {
    let a = mem_frag.read(args[0], 1)[0];
    let b = mem_frag.read(args[0], 1)[0];
    let out = if a & b == 0u8 { 0u8 } else { 1u8 };
    mem_frag.write(args[2], 8, &out.to_le_bytes());
    None
  });

  cpu!("ori8", |args, mem_frag, _, _| {
    let a = i8::from_le_bytes(mem_frag.read(args[0], 1).try_into().unwrap());
    let b = i8::from_le_bytes(mem_frag.read(args[1], 1).try_into().unwrap());
    let out = a | b;
    mem_frag.write(args[2], 1, &out.to_le_bytes());
    None
  });
  cpu!("ori16", |args, mem_frag, _, _| {
    let a = LittleEndian::read_i16(mem_frag.read(args[0], 2));
    let b = LittleEndian::read_i16(mem_frag.read(args[1], 2));
    let out = a | b;
    mem_frag.write(args[2], 2, &out.to_le_bytes());
    None
  });
  cpu!("ori32", |args, mem_frag, _, _| {
    let a = LittleEndian::read_i32(mem_frag.read(args[0], 4));
    let b = LittleEndian::read_i32(mem_frag.read(args[1], 4));
    let out = a | b;
    mem_frag.write(args[2], 4, &out.to_le_bytes());
    None
  });
  cpu!("ori64", |args, mem_frag, _, _| {
    let a = LittleEndian::read_i64(mem_frag.read(args[0], 8));
    let b = LittleEndian::read_i64(mem_frag.read(args[1], 8));
    let out = a | b;
    mem_frag.write(args[2], 8, &out.to_le_bytes());
    None
  });
  cpu!("orbool", |args, mem_frag, _, _| {
    let a = mem_frag.read(args[0], 1)[0];
    let b = mem_frag.read(args[0], 1)[0];
    let out = if a | b == 0u8 { 0u8 } else { 1u8 };
    mem_frag.write(args[2], 8, &out.to_le_bytes());
    None
  });

  cpu!("xori8", |args, mem_frag, _, _| {
    let a = i8::from_le_bytes(mem_frag.read(args[0], 1).try_into().unwrap());
    let b = i8::from_le_bytes(mem_frag.read(args[1], 1).try_into().unwrap());
    let out = a ^ b;
    mem_frag.write(args[2], 1, &out.to_le_bytes());
    None
  });
  cpu!("xori16", |args, mem_frag, _, _| {
    let a = LittleEndian::read_i16(mem_frag.read(args[0], 2));
    let b = LittleEndian::read_i16(mem_frag.read(args[1], 2));
    let out = a ^ b;
    mem_frag.write(args[2], 2, &out.to_le_bytes());
    None
  });
  cpu!("xori32", |args, mem_frag, _, _| {
    let a = LittleEndian::read_i32(mem_frag.read(args[0], 4));
    let b = LittleEndian::read_i32(mem_frag.read(args[1], 4));
    let out = a ^ b;
    mem_frag.write(args[2], 4, &out.to_le_bytes());
    None
  });
  cpu!("xori64", |args, mem_frag, _, _| {
    let a = LittleEndian::read_i64(mem_frag.read(args[0], 8));
    let b = LittleEndian::read_i64(mem_frag.read(args[1], 8));
    let out = a ^ b;
    mem_frag.write(args[2], 8, &out.to_le_bytes());
    None
  });
  cpu!("xorbool", |args, mem_frag, _, _| {
    let a = mem_frag.read(args[0], 1)[0];
    let b = mem_frag.read(args[0], 1)[0];
    let out = if a ^ b == 0u8 { 0u8 } else { 1u8 };
    mem_frag.write(args[2], 8, &out.to_le_bytes());
    None
  });

  cpu!("noti8", |args, mem_frag, _, _| {
    let a = i8::from_le_bytes(mem_frag.read(args[0], 1).try_into().unwrap());
    let out = !a;
    mem_frag.write(args[2], 1, &out.to_le_bytes());
    None
  });
  cpu!("noti16", |args, mem_frag, _, _| {
    let a = LittleEndian::read_i16(mem_frag.read(args[0], 2));
    let out = !a;
    mem_frag.write(args[2], 2, &out.to_le_bytes());
    None
  });
  cpu!("noti32", |args, mem_frag, _, _| {
    let a = LittleEndian::read_i32(mem_frag.read(args[0], 4));
    let out = !a;
    mem_frag.write(args[2], 4, &out.to_le_bytes());
    None
  });
  cpu!("noti64", |args, mem_frag, _, _| {
    let a = LittleEndian::read_i64(mem_frag.read(args[0], 8));
    let out = !a;
    mem_frag.write(args[2], 8, &out.to_le_bytes());
    None
  });
  cpu!("notbool", |args, mem_frag, _, _| {
    let a = mem_frag.read(args[0], 1)[0];
    let out = if a == 0u8 { 1u8 } else { 0u8 };
    mem_frag.write(args[2], 8, &out.to_le_bytes());
    None
  });

  cpu!("nandi8", |args, mem_frag, _, _| {
    let a = i8::from_le_bytes(mem_frag.read(args[0], 1).try_into().unwrap());
    let b = i8::from_le_bytes(mem_frag.read(args[1], 1).try_into().unwrap());
    let out = !(a & b);
    mem_frag.write(args[2], 1, &out.to_le_bytes());
    None
  });
  cpu!("nandi16", |args, mem_frag, _, _| {
    let a = LittleEndian::read_i16(mem_frag.read(args[0], 2));
    let b = LittleEndian::read_i16(mem_frag.read(args[1], 2));
    let out = !(a & b);
    mem_frag.write(args[2], 2, &out.to_le_bytes());
    None
  });
  cpu!("nandi32", |args, mem_frag, _, _| {
    let a = LittleEndian::read_i32(mem_frag.read(args[0], 4));
    let b = LittleEndian::read_i32(mem_frag.read(args[1], 4));
    let out = !(a & b);
    mem_frag.write(args[2], 4, &out.to_le_bytes());
    None
  });
  cpu!("nandi64", |args, mem_frag, _, _| {
    let a = LittleEndian::read_i64(mem_frag.read(args[0], 8));
    let b = LittleEndian::read_i64(mem_frag.read(args[1], 8));
    let out = !(a & b);
    mem_frag.write(args[2], 8, &out.to_le_bytes());
    None
  });
  cpu!("nandboo", |args, mem_frag, _, _| {
    let a = mem_frag.read(args[0], 1)[0];
    let b = mem_frag.read(args[0], 1)[0];
    let out = if !(a & b) == 0u8 { 0u8 } else { 1u8 };
    mem_frag.write(args[2], 8, &out.to_le_bytes());
    None
  });

  cpu!("nori8", |args, mem_frag, _, _| {
    let a = i8::from_le_bytes(mem_frag.read(args[0], 1).try_into().unwrap());
    let b = i8::from_le_bytes(mem_frag.read(args[1], 1).try_into().unwrap());
    let out = !(a | b);
    mem_frag.write(args[2], 1, &out.to_le_bytes());
    None
  });
  cpu!("nori16", |args, mem_frag, _, _| {
    let a = LittleEndian::read_i16(mem_frag.read(args[0], 2));
    let b = LittleEndian::read_i16(mem_frag.read(args[1], 2));
    let out = !(a | b);
    mem_frag.write(args[2], 2, &out.to_le_bytes());
    None
  });
  cpu!("nori32", |args, mem_frag, _, _| {
    let a = LittleEndian::read_i32(mem_frag.read(args[0], 4));
    let b = LittleEndian::read_i32(mem_frag.read(args[1], 4));
    let out = !(a | b);
    mem_frag.write(args[2], 4, &out.to_le_bytes());
    None
  });
  cpu!("nori64", |args, mem_frag, _, _| {
    let a = LittleEndian::read_i64(mem_frag.read(args[0], 8));
    let b = LittleEndian::read_i64(mem_frag.read(args[1], 8));
    let out = !(a | b);
    mem_frag.write(args[2], 8, &out.to_le_bytes());
    None
  });
  cpu!("norbool", |args, mem_frag, _, _| {
    let a = mem_frag.read(args[0], 1)[0];
    let b = mem_frag.read(args[0], 1)[0];
    let out = if !(a | b) == 0u8 { 0u8 } else { 1u8 };
    mem_frag.write(args[2], 8, &out.to_le_bytes());
    None
  });

  cpu!("xnori8", |args, mem_frag, _, _| {
    let a = i8::from_le_bytes(mem_frag.read(args[0], 1).try_into().unwrap());
    let b = i8::from_le_bytes(mem_frag.read(args[1], 1).try_into().unwrap());
    let out = !(a ^ b);
    mem_frag.write(args[2], 1, &out.to_le_bytes());
    None
  });
  cpu!("xnori16", |args, mem_frag, _, _| {
    let a = LittleEndian::read_i16(mem_frag.read(args[0], 2));
    let b = LittleEndian::read_i16(mem_frag.read(args[1], 2));
    let out = !(a ^ b);
    mem_frag.write(args[2], 2, &out.to_le_bytes());
    None
  });
  cpu!("xnori32", |args, mem_frag, _, _| {
    let a = LittleEndian::read_i32(mem_frag.read(args[0], 4));
    let b = LittleEndian::read_i32(mem_frag.read(args[1], 4));
    let out = !(a ^ b);
    mem_frag.write(args[2], 4, &out.to_le_bytes());
    None
  });
  cpu!("xnori64", |args, mem_frag, _, _| {
    let a = LittleEndian::read_i64(mem_frag.read(args[0], 8));
    let b = LittleEndian::read_i64(mem_frag.read(args[1], 8));
    let out = !(a ^ b);
    mem_frag.write(args[2], 8, &out.to_le_bytes());
    None
  });
  cpu!("xnorboo", |args, mem_frag, _, _| {
    let a = mem_frag.read(args[0], 1)[0];
    let b = mem_frag.read(args[0], 1)[0];
    let out = if !(a ^ b) == 0u8 { 0u8 } else { 1u8 };
    mem_frag.write(args[2], 8, &out.to_le_bytes());
    None
  });

  // Equality and order opcodes
  cpu!("eqi8", |args, mem_frag, _, _| {
    let a = i8::from_le_bytes(mem_frag.read(args[0], 1).try_into().unwrap());
    let b = i8::from_le_bytes(mem_frag.read(args[1], 1).try_into().unwrap());
    let out = if a == b { 1u8 } else { 0u8 };
    mem_frag.write(args[2], 1, &out.to_le_bytes());
    None
  });
  cpu!("eqi16", |args, mem_frag, _, _| {
    let a = LittleEndian::read_i16(mem_frag.read(args[0], 2));
    let b = LittleEndian::read_i16(mem_frag.read(args[1], 2));
    let out = if a == b { 1u8 } else { 0u8 };
    mem_frag.write(args[2], 1, &out.to_le_bytes());
    None
  });
  cpu!("eqi32", |args, mem_frag, _, _| {
    let a = LittleEndian::read_i32(mem_frag.read(args[0], 4));
    let b = LittleEndian::read_i32(mem_frag.read(args[1], 4));
    let out = if a == b { 1u8 } else { 0u8 };
    mem_frag.write(args[2], 1, &out.to_le_bytes());
    None
  });
  cpu!("eqi64", |args, mem_frag, _, _| {
    let a = LittleEndian::read_i64(mem_frag.read(args[0], 8));
    let b = LittleEndian::read_i64(mem_frag.read(args[1], 8));
    let out = if a == b { 1u8 } else { 0u8 };
    mem_frag.write(args[2], 1, &out.to_le_bytes());
    None
  });
  cpu!("eqf32", |args, mem_frag, _, _| {
    let a = LittleEndian::read_f32(mem_frag.read(args[0], 4));
    let b = LittleEndian::read_f32(mem_frag.read(args[1], 4));
    let out = if a == b { 1u8 } else { 0u8 };
    mem_frag.write(args[2], 1, &out.to_le_bytes());
    None
  });
  cpu!("eqf64", |args, mem_frag, _, _| {
    let a = LittleEndian::read_f64(mem_frag.read(args[0], 8));
    let b = LittleEndian::read_f64(mem_frag.read(args[1], 8));
    let out = if a == b { 1u8 } else { 0u8 };
    mem_frag.write(args[2], 1, &out.to_le_bytes());
    None
  });
  cpu!("eqstr", |args, mem_frag, _, _| {
    let a_pascal_string = mem_frag.read(args[0], 0);
    let a_size = LittleEndian::read_u64(&a_pascal_string[0..8]) as usize;
    let a_str = str::from_utf8(&a_pascal_string[8..a_size + 8]).unwrap();
    let b_pascal_string = mem_frag.read(args[1], 0);
    let b_size = LittleEndian::read_u64(&b_pascal_string[0..8]) as usize;
    let b_str = str::from_utf8(&b_pascal_string[8..b_size + 8]).unwrap();
    let out = if a_pascal_string == b_pascal_string { 1u8 } else { 0u8 };
    mem_frag.write(args[2], 1, &out.to_le_bytes());
    None
  });
  cpu!("eqbool", |args, mem_frag, _, _| {
    let a = mem_frag.read(args[0], 1)[0];
    let b = mem_frag.read(args[0], 1)[0];
    let out = if a == b { 1u8 } else { 0u8 };
    mem_frag.write(args[2], 1, &out.to_le_bytes());
    None
  });

  cpu!("neqi8", |args, mem_frag, _, _| {
    let a = i8::from_le_bytes(mem_frag.read(args[0], 1).try_into().unwrap());
    let b = i8::from_le_bytes(mem_frag.read(args[1], 1).try_into().unwrap());
    let out = if a != b { 1u8 } else { 0u8 };
    mem_frag.write(args[2], 1, &out.to_le_bytes());
    None
  });
  cpu!("neqi16", |args, mem_frag, _, _| {
    let a = LittleEndian::read_i16(mem_frag.read(args[0], 2));
    let b = LittleEndian::read_i16(mem_frag.read(args[1], 2));
    let out = if a != b { 1u8 } else { 0u8 };
    mem_frag.write(args[2], 1, &out.to_le_bytes());
    None
  });
  cpu!("neqi32", |args, mem_frag, _, _| {
    let a = LittleEndian::read_i32(mem_frag.read(args[0], 4));
    let b = LittleEndian::read_i32(mem_frag.read(args[1], 4));
    let out = if a != b { 1u8 } else { 0u8 };
    mem_frag.write(args[2], 1, &out.to_le_bytes());
    None
  });
  cpu!("neqi64", |args, mem_frag, _, _| {
    let a = LittleEndian::read_i64(mem_frag.read(args[0], 8));
    let b = LittleEndian::read_i64(mem_frag.read(args[1], 8));
    let out = if a != b { 1u8 } else { 0u8 };
    mem_frag.write(args[2], 1, &out.to_le_bytes());
    None
  });
  cpu!("neqf32", |args, mem_frag, _, _| {
    let a = LittleEndian::read_f32(mem_frag.read(args[0], 4));
    let b = LittleEndian::read_f32(mem_frag.read(args[1], 4));
    let out = if a != b { 1u8 } else { 0u8 };
    mem_frag.write(args[2], 1, &out.to_le_bytes());
    None
  });
  cpu!("neqf64", |args, mem_frag, _, _| {
    let a = LittleEndian::read_f64(mem_frag.read(args[0], 8));
    let b = LittleEndian::read_f64(mem_frag.read(args[1], 8));
    let out = if a != b { 1u8 } else { 0u8 };
    mem_frag.write(args[2], 1, &out.to_le_bytes());
    None
  });
  cpu!("neqstr", |args, mem_frag, _, _| {
    let a_pascal_string = mem_frag.read(args[0], 0);
    let a_size = LittleEndian::read_u64(&a_pascal_string[0..8]) as usize;
    let a_str = str::from_utf8(&a_pascal_string[8..a_size + 8]).unwrap();
    let b_pascal_string = mem_frag.read(args[1], 0);
    let b_size = LittleEndian::read_u64(&b_pascal_string[0..8]) as usize;
    let b_str = str::from_utf8(&b_pascal_string[8..b_size + 8]).unwrap();
    let out = if a_pascal_string != b_pascal_string { 1u8 } else { 0u8 };
    mem_frag.write(args[2], 1, &out.to_le_bytes());
    None
  });
  cpu!("neqbool", |args, mem_frag, _, _| {
    let a = mem_frag.read(args[0], 1)[0];
    let b = mem_frag.read(args[0], 1)[0];
    let out = if a != b { 1u8 } else { 0u8 };
    mem_frag.write(args[2], 1, &out.to_le_bytes());
    None
  });

  cpu!("lti8", |args, mem_frag, _, _| {
    let a = i8::from_le_bytes(mem_frag.read(args[0], 1).try_into().unwrap());
    let b = i8::from_le_bytes(mem_frag.read(args[1], 1).try_into().unwrap());
    let out = if a < b { 1u8 } else { 0u8 };
    mem_frag.write(args[2], 1, &out.to_le_bytes());
    None
  });
  cpu!("lti16", |args, mem_frag, _, _| {
    let a = LittleEndian::read_i16(mem_frag.read(args[0], 2));
    let b = LittleEndian::read_i16(mem_frag.read(args[1], 2));
    let out = if a < b { 1u8 } else { 0u8 };
    mem_frag.write(args[2], 1, &out.to_le_bytes());
    None
  });
  cpu!("lti32", |args, mem_frag, _, _| {
    let a = LittleEndian::read_i32(mem_frag.read(args[0], 4));
    let b = LittleEndian::read_i32(mem_frag.read(args[1], 4));
    let out = if a < b { 1u8 } else { 0u8 };
    mem_frag.write(args[2], 1, &out.to_le_bytes());
    None
  });
  cpu!("lti64", |args, mem_frag, _, _| {
    let a = LittleEndian::read_i64(mem_frag.read(args[0], 8));
    let b = LittleEndian::read_i64(mem_frag.read(args[1], 8));
    let out = if a < b { 1u8 } else { 0u8 };
    mem_frag.write(args[2], 1, &out.to_le_bytes());
    None
  });
  cpu!("ltf32", |args, mem_frag, _, _| {
    let a = LittleEndian::read_f32(mem_frag.read(args[0], 4));
    let b = LittleEndian::read_f32(mem_frag.read(args[1], 4));
    let out = if a < b { 1u8 } else { 0u8 };
    mem_frag.write(args[2], 1, &out.to_le_bytes());
    None
  });
  cpu!("ltf64", |args, mem_frag, _, _| {
    let a = LittleEndian::read_f64(mem_frag.read(args[0], 8));
    let b = LittleEndian::read_f64(mem_frag.read(args[1], 8));
    let out = if a < b { 1u8 } else { 0u8 };
    mem_frag.write(args[2], 1, &out.to_le_bytes());
    None
  });
  cpu!("ltstr", |args, mem_frag, _, _| {
    let a_pascal_string = mem_frag.read(args[0], 0);
    let a_size = LittleEndian::read_u64(&a_pascal_string[0..8]) as usize;
    let a_str = str::from_utf8(&a_pascal_string[8..a_size + 8]).unwrap();
    let b_pascal_string = mem_frag.read(args[1], 0);
    let b_size = LittleEndian::read_u64(&b_pascal_string[0..8]) as usize;
    let b_str = str::from_utf8(&b_pascal_string[8..b_size + 8]).unwrap();
    let out = if a_pascal_string < b_pascal_string { 1u8 } else { 0u8 };
    mem_frag.write(args[2], 1, &out.to_le_bytes());
    None
  });

  cpu!("ltei8", |args, mem_frag, _, _| {
    let a = i8::from_le_bytes(mem_frag.read(args[0], 1).try_into().unwrap());
    let b = i8::from_le_bytes(mem_frag.read(args[1], 1).try_into().unwrap());
    let out = if a <= b { 1u8 } else { 0u8 };
    mem_frag.write(args[2], 1, &out.to_le_bytes());
    None
  });
  cpu!("ltei16", |args, mem_frag, _, _| {
    let a = LittleEndian::read_i16(mem_frag.read(args[0], 2));
    let b = LittleEndian::read_i16(mem_frag.read(args[1], 2));
    let out = if a <= b { 1u8 } else { 0u8 };
    mem_frag.write(args[2], 1, &out.to_le_bytes());
    None
  });
  cpu!("ltei32", |args, mem_frag, _, _| {
    let a = LittleEndian::read_i32(mem_frag.read(args[0], 4));
    let b = LittleEndian::read_i32(mem_frag.read(args[1], 4));
    let out = if a <= b { 1u8 } else { 0u8 };
    mem_frag.write(args[2], 1, &out.to_le_bytes());
    None
  });
  cpu!("ltei64", |args, mem_frag, _, _| {
    let a = LittleEndian::read_i64(mem_frag.read(args[0], 8));
    let b = LittleEndian::read_i64(mem_frag.read(args[1], 8));
    let out = if a <= b { 1u8 } else { 0u8 };
    mem_frag.write(args[2], 1, &out.to_le_bytes());
    None
  });
  cpu!("ltef32", |args, mem_frag, _, _| {
    let a = LittleEndian::read_f32(mem_frag.read(args[0], 4));
    let b = LittleEndian::read_f32(mem_frag.read(args[1], 4));
    let out = if a <= b { 1u8 } else { 0u8 };
    mem_frag.write(args[2], 1, &out.to_le_bytes());
    None
  });
  cpu!("ltef64", |args, mem_frag, _, _| {
    let a = LittleEndian::read_f64(mem_frag.read(args[0], 8));
    let b = LittleEndian::read_f64(mem_frag.read(args[1], 8));
    let out = if a <= b { 1u8 } else { 0u8 };
    mem_frag.write(args[2], 1, &out.to_le_bytes());
    None
  });
  cpu!("ltestr", |args, mem_frag, _, _| {
    let a_pascal_string = mem_frag.read(args[0], 0);
    let a_size = LittleEndian::read_u64(&a_pascal_string[0..8]) as usize;
    let a_str = str::from_utf8(&a_pascal_string[8..a_size + 8]).unwrap();
    let b_pascal_string = mem_frag.read(args[1], 0);
    let b_size = LittleEndian::read_u64(&b_pascal_string[0..8]) as usize;
    let b_str = str::from_utf8(&b_pascal_string[8..b_size + 8]).unwrap();
    let out = if a_pascal_string <= b_pascal_string { 1u8 } else { 0u8 };
    mem_frag.write(args[2], 1, &out.to_le_bytes());
    None
  });

  cpu!("gti8", |args, mem_frag, _, _| {
    let a = i8::from_le_bytes(mem_frag.read(args[0], 1).try_into().unwrap());
    let b = i8::from_le_bytes(mem_frag.read(args[1], 1).try_into().unwrap());
    let out = if a > b { 1u8 } else { 0u8 };
    mem_frag.write(args[2], 1, &out.to_le_bytes());
    None
  });
  cpu!("gti16", |args, mem_frag, _, _| {
    let a = LittleEndian::read_i16(mem_frag.read(args[0], 2));
    let b = LittleEndian::read_i16(mem_frag.read(args[1], 2));
    let out = if a > b { 1u8 } else { 0u8 };
    mem_frag.write(args[2], 1, &out.to_le_bytes());
    None
  });
  cpu!("gti32", |args, mem_frag, _, _| {
    let a = LittleEndian::read_i32(mem_frag.read(args[0], 4));
    let b = LittleEndian::read_i32(mem_frag.read(args[1], 4));
    let out = if a > b { 1u8 } else { 0u8 };
    mem_frag.write(args[2], 1, &out.to_le_bytes());
    None
  });
  cpu!("gti64", |args, mem_frag, _, _| {
    let a = LittleEndian::read_i64(mem_frag.read(args[0], 8));
    let b = LittleEndian::read_i64(mem_frag.read(args[1], 8));
    let out = if a > b { 1u8 } else { 0u8 };
    mem_frag.write(args[2], 1, &out.to_le_bytes());
    None
  });
  cpu!("gtf32", |args, mem_frag, _, _| {
    let a = LittleEndian::read_f32(mem_frag.read(args[0], 4));
    let b = LittleEndian::read_f32(mem_frag.read(args[1], 4));
    let out = if a > b { 1u8 } else { 0u8 };
    mem_frag.write(args[2], 1, &out.to_le_bytes());
    None
  });
  cpu!("gtf64", |args, mem_frag, _, _| {
    let a = LittleEndian::read_f64(mem_frag.read(args[0], 8));
    let b = LittleEndian::read_f64(mem_frag.read(args[1], 8));
    let out = if a > b { 1u8 } else { 0u8 };
    mem_frag.write(args[2], 1, &out.to_le_bytes());
    None
  });
  cpu!("gtstr", |args, mem_frag, _, _| {
    let a_pascal_string = mem_frag.read(args[0], 0);
    let a_size = LittleEndian::read_u64(&a_pascal_string[0..8]) as usize;
    let a_str = str::from_utf8(&a_pascal_string[8..a_size + 8]).unwrap();
    let b_pascal_string = mem_frag.read(args[1], 0);
    let b_size = LittleEndian::read_u64(&b_pascal_string[0..8]) as usize;
    let b_str = str::from_utf8(&b_pascal_string[8..b_size + 8]).unwrap();
    let out = if a_pascal_string > b_pascal_string { 1u8 } else { 0u8 };
    mem_frag.write(args[2], 1, &out.to_le_bytes());
    None
  });

  cpu!("gtei8", |args, mem_frag, _, _| {
    let a = i8::from_le_bytes(mem_frag.read(args[0], 1).try_into().unwrap());
    let b = i8::from_le_bytes(mem_frag.read(args[1], 1).try_into().unwrap());
    let out = if a >= b { 1u8 } else { 0u8 };
    mem_frag.write(args[2], 1, &out.to_le_bytes());
    None
  });
  cpu!("gtei16", |args, mem_frag, _, _| {
    let a = LittleEndian::read_i16(mem_frag.read(args[0], 2));
    let b = LittleEndian::read_i16(mem_frag.read(args[1], 2));
    let out = if a >= b { 1u8 } else { 0u8 };
    mem_frag.write(args[2], 1, &out.to_le_bytes());
    None
  });
  cpu!("gtei32", |args, mem_frag, _, _| {
    let a = LittleEndian::read_i32(mem_frag.read(args[0], 4));
    let b = LittleEndian::read_i32(mem_frag.read(args[1], 4));
    let out = if a >= b { 1u8 } else { 0u8 };
    mem_frag.write(args[2], 1, &out.to_le_bytes());
    None
  });
  cpu!("gtei64", |args, mem_frag, _, _| {
    let a = LittleEndian::read_i64(mem_frag.read(args[0], 8));
    let b = LittleEndian::read_i64(mem_frag.read(args[1], 8));
    let out = if a >= b { 1u8 } else { 0u8 };
    mem_frag.write(args[2], 1, &out.to_le_bytes());
    None
  });
  cpu!("gtef32", |args, mem_frag, _, _| {
    let a = LittleEndian::read_f32(mem_frag.read(args[0], 4));
    let b = LittleEndian::read_f32(mem_frag.read(args[1], 4));
    let out = if a >= b { 1u8 } else { 0u8 };
    mem_frag.write(args[2], 1, &out.to_le_bytes());
    None
  });
  cpu!("gtef64", |args, mem_frag, _, _| {
    let a = LittleEndian::read_f64(mem_frag.read(args[0], 8));
    let b = LittleEndian::read_f64(mem_frag.read(args[1], 8));
    let out = if a >= b { 1u8 } else { 0u8 };
    mem_frag.write(args[2], 1, &out.to_le_bytes());
    None
  });
  cpu!("gtestr", |args, mem_frag, _, _| {
    let a_pascal_string = mem_frag.read(args[0], 0);
    let a_size = LittleEndian::read_u64(&a_pascal_string[0..8]) as usize;
    let a_str = str::from_utf8(&a_pascal_string[8..a_size + 8]).unwrap();
    let b_pascal_string = mem_frag.read(args[1], 0);
    let b_size = LittleEndian::read_u64(&b_pascal_string[0..8]) as usize;
    let b_str = str::from_utf8(&b_pascal_string[8..b_size + 8]).unwrap();
    let out = if a_pascal_string >= b_pascal_string { 1u8 } else { 0u8 };
    mem_frag.write(args[2], 1, &out.to_le_bytes());
    None
  });

  // String opcodes
  cpu!("catstr", |args, mem_frag, _, _| {
    let a = mem_frag.read(args[0], 0);
    let b = mem_frag.read(args[1], 0);
    let a_size = LittleEndian::read_u64(&a[0..8]) as usize;
    let b_size = LittleEndian::read_u64(&b[0..8]) as usize;
    let a_str = str::from_utf8(&a[8..a_size + 8]).unwrap();
    let b_str = str::from_utf8(&b[8..b_size + 8]).unwrap();
    let out_str = format!("{}{}", a_str, b_str);
    let mut out = out_str.len().to_le_bytes().to_vec();
    out.append(&mut out_str.as_bytes().to_vec());
    mem_frag.write(args[2], 0, &out);
    None
  });
  // TODO: `split` after Arrays work in the runtime
  cpu!("repstr", |args, mem_frag, _, _| {
    let a_pascal_string = mem_frag.read(args[0], 0);
    let a_size = LittleEndian::read_u64(&a_pascal_string[0..8]) as usize;
    let a_str = str::from_utf8(&a_pascal_string[8..a_size + 8]).unwrap();
    let n = LittleEndian::read_i64(mem_frag.read(args[0], 8));
    let out_str = a_str.repeat(n as usize);
    let mut out = out_str.len().to_le_bytes().to_vec();
    out.append(&mut out_str.as_bytes().to_vec());
    mem_frag.write(args[2], 0, &out);
    None
  });
  cpu!("matches", |args, mem_frag, _, _| {
    let a_pascal_string = mem_frag.read(args[0], 0);
    let a_size = LittleEndian::read_u64(&a_pascal_string[0..8]) as usize;
    let a_str = str::from_utf8(&a_pascal_string[8..a_size + 8]).unwrap();
    let b_pascal_string = mem_frag.read(args[1], 0);
    let b_size = LittleEndian::read_u64(&b_pascal_string[0..8]) as usize;
    let b_str = str::from_utf8(&b_pascal_string[8..b_size + 8]).unwrap();
    let out = if a_str.contains(b_str) { 1u8 } else { 0u8 };
    mem_frag.write(args[2], 1, &out.to_le_bytes());
    None
  });
  cpu!("indstr", |args, mem_frag, _, _| {
    let a_pascal_string = mem_frag.read(args[0], 0);
    let a_size = LittleEndian::read_u64(&a_pascal_string[0..8]) as usize;
    let a_str = str::from_utf8(&a_pascal_string[8..a_size + 8]).unwrap();
    let b_pascal_string = mem_frag.read(args[1], 0);
    let b_size = LittleEndian::read_u64(&b_pascal_string[0..8]) as usize;
    let b_str = str::from_utf8(&b_pascal_string[8..b_size + 8]).unwrap();
    let out_option = a_str.find(b_str);
    let out = if out_option.is_none()  { -1i64 } else { out_option.unwrap() as i64 };
    mem_frag.write(args[2], 8, &out.to_le_bytes());
    None
  });
  cpu!("lenstr", |args, mem_frag, _, _| {
    let a_pascal_string = mem_frag.read(args[0], 0);
    let a_size = LittleEndian::read_u64(&a_pascal_string[0..8]) as usize;
    let a_str = str::from_utf8(&a_pascal_string[8..a_size + 8]).unwrap();
    let out = a_str.len() as i64;
    mem_frag.write(args[2], 8, &out.to_le_bytes());
    None
  });
  cpu!("trim", |args, mem_frag, _, _| {
    let a_pascal_string = mem_frag.read(args[0], 0);
    let a_size = LittleEndian::read_u64(&a_pascal_string[0..8]) as usize;
    let a_str = str::from_utf8(&a_pascal_string[8..a_size + 8]).unwrap();
    let out_str = a_str.trim();
    let mut out = out_str.len().to_le_bytes().to_vec();
    out.append(&mut out_str.as_bytes().to_vec());
    mem_frag.write(args[2], 0, &out);
    None
  });

  // Array opcodes TODO after arrays are implemented

  // Map opcodes TODO after maps are implemented

  // Ternary opcodes
  // TODO: pair and condarr after arrays are implemented
  unpred_cpu!("condfn", |args, mem_frag, _, frag| {
    let cond = LittleEndian::read_i64(mem_frag.read(args[0], 8));
    let subfn = args[1];
    if cond == 1 {
      frag.insert_subhandler(subfn);
    }
    None
  });

  // "Special" opcodes
  io!("waitop", |_, mem_frag, _, _| {
    let payload = mem_frag.read(mem_frag.payload_addr.unwrap(), 1);
    let ms = LittleEndian::read_i64(&payload[0..8]) as u64;
    return Box::pin(delay_for(Duration::from_millis(ms)));
  });
  cpu!("exitop", |_, mem_frag, _, _| {
    std::process::exit(LittleEndian::read_i32(mem_frag.read(mem_frag.payload_addr.unwrap(), 4)));
  });
  cpu!("stdoutp", |_, mem_frag, _, _| {
    let pascal_string = mem_frag.read(mem_frag.payload_addr.unwrap(), 0);
    let size = LittleEndian::read_u64(&pascal_string[0..8]) as usize;
    let out_str = str::from_utf8(&pascal_string[8..size + 8]).unwrap();
    print!("{}", out_str);
    None
  });
  // TODO: Remove this opcode in the future
  cpu!("set i64", |args, mem_frag, _, _| {
    let data = mem_frag.read(args[1], 8).to_vec();
    mem_frag.write(args[1], 8, &data);
    None
  });
  cpu!("emit to:", |args, mem_frag, event_declrs, _| {
    let event = if args.len() <= 1 {
      EventEmit { id: args[0], payload: None, gmem_addr: None }
    } else {
      if args[1] < 0 {
        // payload addr is in global memory if negative
        EventEmit { id: args[0], payload: None, gmem_addr: Some(args[1]) }
      } else {
        let addr = args[1];
        let pls = event_declrs.get(&args[0]).unwrap().clone() as u8;
        let payload = Some(mem_frag.read(addr, pls).to_vec());
        EventEmit { id: args[0], payload: payload.clone(), gmem_addr: None }
      }
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
