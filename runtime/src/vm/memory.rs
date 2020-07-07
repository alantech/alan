use std::collections::HashMap;

use byteorder::{ByteOrder, LittleEndian};

use crate::vm::program::Program;

/// Memory representation of a handler call
#[derive(Clone)]
pub struct HandlerMemory {
  /// Global memory reference
  gmem: &'static Vec<u8>,
  /// Memory of the handler for fixed size data types
  mem: Vec<i64>,
  /// Fractal memory storage for variable-length data types to an instance of the same struct
  fractal_mem: HashMap<i64, HandlerMemory>,
  /// Pointers to nested fractal HandlerMemory. Each is represented as a vector of up to 3 sequential addresses.
  /// These are not quite registers since they are not used by opcodes directly and they
  /// don't store the data itself, but an address to the data.
  registers_ish: HashMap<i64, Vec<i64>>,
}

impl HandlerMemory {
  /// Allocates a payload for the given event id from the address within the HandlerMemory
  /// provided to a new HandlerMemory. Called by "emit to" opcode.
  pub fn alloc_payload(
    event_id: i64,
    curr_addr: i64,
    curr_hand_mem: &HandlerMemory,
  ) -> Option<HandlerMemory> {
    let pls = Program::global().event_pls.get(&event_id).unwrap().clone();
    if pls == 0 {
      // no payload, void event
      return None;
    }
    // the size of this array will be different for every handler so it will be resized later
    let mut mem = vec![];
    let mut fractal_mem = HashMap::new();
    if pls < 0 {
      // payload is a variable-length data type
      let payload: HandlerMemory = curr_hand_mem.fractal_mem.get(&curr_addr).unwrap().clone();
      fractal_mem.insert(0, payload.clone());
    } else {
      // payload is a fixed length data type which could be in global memory
      mem.push(curr_hand_mem.read_fixed(curr_addr));
    };
    return Some(HandlerMemory {
      mem,
      fractal_mem,
      gmem: curr_hand_mem.gmem,
      registers_ish: HashMap::new(),
    });
  }

  pub fn new(payload_mem: Option<HandlerMemory>,  mem_req: i64) -> HandlerMemory {
    let mut hand_mem = if payload_mem.is_none() {
      HandlerMemory {
        gmem: &Program::global().gmem,
        mem: vec![0; (mem_req / 8) as usize],
        fractal_mem: HashMap::new(),
        registers_ish: HashMap::new(),
      }
    } else {
      payload_mem.unwrap()
    };
    // hand_mem.mem.resize((mem_req / 8) as usize, 0);
    return hand_mem;
  }

  fn get_mut_arr(self: &mut HandlerMemory, addr: i64) -> &mut HandlerMemory {
    let arr_opt = self.fractal_mem.get_mut(&(addr / 8));
    if arr_opt.is_none() {
      panic!("Array at address {} does not exist.", addr);
    };
    return arr_opt.unwrap();
  }

  fn get_arr(self: &HandlerMemory, addr: i64) -> &HandlerMemory {
    let arr_opt = self.fractal_mem.get(&(addr / 8));
    if arr_opt.is_none() {
      panic!("Array at address {} does not exist.", addr);
    };
    return arr_opt.unwrap();
  }

  /// set registerish and return its address
  pub fn set_reg(self: &mut HandlerMemory, reg_addr: i64, arr_addr1: i64, arr_addr2: i64) {
    let arr_addrs = vec![arr_addr1 / 8, arr_addr2 / 8];
    self.registers_ish.insert(reg_addr / 8, arr_addrs);
  }

  /// The address provided can be a directly nested fractal or a registerish address that points to
  /// a fractal. Either returns a reference to a HandlerMemory
  pub fn get_fractal(self: &HandlerMemory, addr: i64) -> &HandlerMemory {
    let reg_opt = self.registers_ish.get(&(addr / 8));
    if reg_opt.is_none() {
      let arr_opt = self.fractal_mem.get(&(addr / 8));
      if arr_opt.is_some() {
        return arr_opt.unwrap();
      }
      panic!("Register at address {} does not exist.", addr);
    }
    let reg = reg_opt.unwrap().to_vec();
    let mut arr = self.get_arr(reg[0]);
    for (i, addr) in reg.iter().enumerate() {
      if i == 0 { continue };
      arr = arr.get_arr(addr / 8);
    }
    return arr;
  }

  /// The address provided can be a directly nested fractal or a registerish address that points to
  /// a fractal. Either returns a reference to a HandlerMemory
  pub fn get_mut_fractal(self: &mut HandlerMemory, addr: i64) -> &mut HandlerMemory {
    let reg_opt = self.registers_ish.get(&(addr / 8));
    if reg_opt.is_none() {
      let arr_opt = self.fractal_mem.get_mut(&(addr / 8));
      if arr_opt.is_some() {
        return arr_opt.unwrap();
      }
      panic!("Register at address {} does not exist.", addr);
    }
    let reg = reg_opt.unwrap().to_vec();
    let mut arr = self.get_mut_arr(reg[0]);
    for (i, addr) in reg.iter().enumerate() {
      if i == 0 { continue };
      arr = arr.get_mut_arr(*addr);
    }
    return arr;
  }

  /// copy data from outer address to inner address in array or registerish
  pub fn copy_to_fixed(self: &mut HandlerMemory, arr_addr: i64, outer_addr: i64, inner_addr: i64) {
    let data = self.read_fixed(outer_addr / 8);
    let arr = self.get_mut_fractal(arr_addr / 8);
    arr.write_fixed(inner_addr / 8, data);
  }

  pub fn copy_to_arr(self: &mut HandlerMemory, arr_addr: i64, outer_addr: i64, inner_addr: i64) {
    let data_copy = self.read_arr(outer_addr / 8).to_vec();
    let arr = self.get_mut_fractal(arr_addr / 8);
    arr.write_arr(inner_addr / 8, data_copy.as_slice());
  }

  /// copy data from inner address in array to outer address. the array address can point to a
  /// registerish
  pub fn copy_from(self: &mut HandlerMemory, arr_addr:i64, outer_addr:i64, inner_addr: i64) {
    let arr = self.get_fractal(arr_addr / 8);
    let (data, size) = arr.read_either(inner_addr / 8);
    let data_copy = data.to_vec();
    if data_copy.len() > 1 {
      self.write_arr(outer_addr / 8, &data_copy);
    } else {
      self.write_fixed(outer_addr / 8, data_copy[0]);
    }
  }

  pub fn copy_arr(self: &mut HandlerMemory, in_addr: i64, out_addr: i64) {
    let arr = self.get_mut_arr(in_addr);
    let new_arr = arr.clone();
    self.fractal_mem.insert(out_addr / 8, new_arr);
  }

  pub fn len_as_arr(self: &HandlerMemory) -> usize {
    if self.mem.len() > 0 {
      // denormalizing for Array<any> implementation defined in push_arr
      // since the length of mem is used to track the length of fractal_mem
      return self.mem.len();
    }
    // array of types
    return self.fractal_mem.len();
  }

  pub fn new_arr(self: &mut HandlerMemory, addr: i64) {
    let curr = self.read_arr(addr / 8);
    if curr.len() > 0 {
      panic!("Tried to create an array at address {}, but one already exists.", addr);
    }
    self.write_arr(addr / 8, &[]);
  }

  pub fn push_arr(self: &mut HandlerMemory, addr: i64, val: i64) {
    // This implementation uses the `mem` vector as a way to keep track of the total length of the
    // array, as well. It's simple but wastes space when the inserted value is variable-length
    // (such as other strings or other arrays), however it greatly simplifies addressing and
    // lookup, particularly for `Array<any>`, which is also what user-defined types are transformed
    // into. In the future we could have an address translation layer and pack the data as tightly
    // as we can, assuming that doesn't impose a large performance penalty, while this simple
    // solution only adds an extra key's worth of space usage, but does have memory copy issues due
    // to the constant resizing.
    let arr = self.get_mut_arr(addr);
    let idx = arr.mem.len();
    arr.mem.resize(idx + 1, 0);
    arr.write_fixed(idx as i64, val);
  }

  /// removes the last value of the array in the address and returns it
  pub fn pop_arr(self: &mut HandlerMemory, addr: i64) -> Vec<i64> {
    let arr = self.get_mut_arr(addr);
    let idx = arr.fractal_mem.len() - 1;
    let last_addr = idx as i64;
    let last = arr.fractal_mem.remove(&last_addr).unwrap();
    return last.mem;
  }

  /// read address of string or fixed length data type and
  /// return a reference to the data and its size
  /// WARNING fails on reads to global memory, make sure it is not possible to pass this globals
  pub fn read_either(self: &HandlerMemory, addr: i64) -> (Vec<i64>, u8) {
    if addr < 0 {
      panic!("Reads of undefined size do not work on global memory");
    }
    // test if the data read is itself a string/array
    let var = self.read_arr(addr);
    return if var.len() > 0 {
      (var.to_vec(), 0)
    } else {
      // Nope, it's fixed data. We can safely read 8 bytes for all of the fixed types
      (vec![self.read_fixed(addr)], 8)
    };
  }

  pub fn read_fixed(self: &HandlerMemory, addr: i64) -> i64 {
    if addr < 0 {
      let a = (0 - addr - 1) as usize;
      let result = LittleEndian::read_i64(&self.gmem[a..a + 8]);
      return result;
    }
    let a = (addr / 8) as usize;
    let result = self.mem[a];
    return result;
  }

  pub fn read_arr(self: &HandlerMemory, addr: i64) -> Vec<i64> {
    if addr < 0 {
      let a = (0 - addr - 1) as usize;
      let result = &self.gmem[a..];
      let mut out: Vec<i64> = Vec::new();
      for i in 0..(result.len() / 8) {
        let num = LittleEndian::read_i64(&result[8*i..8*i+8]);
        out.push(num);
      }
      return out;
    }
    let a = (addr / 8) as usize;
    let arr = self.fractal_mem.get(&addr);
    let res = if arr.is_none() { Vec::new() } else { arr.unwrap().mem.as_slice().to_vec() };
    return res;
  }

  pub fn write_fixed(self: &mut HandlerMemory, addr: i64, payload: i64) {
    if addr < 0 {
      panic!("You can't write to global memory!");
    }
    let a = (addr / 8) as usize;
    self.mem[a] = payload;
  }

  pub fn write_arr(self: &mut HandlerMemory, addr: i64, payload: &[i64]) {
    if addr < 0 {
      panic!("You can't write to global memory!");
    }
    let a = (addr / 8) as usize;
    let arr = HandlerMemory {
      mem: payload.to_vec(),
      gmem: self.gmem,
      fractal_mem: HashMap::new(),
      registers_ish: HashMap::new()
    };
    self.fractal_mem.insert(addr, arr);
  }
}