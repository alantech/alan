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
  fractal_mem: Vec<HandlerMemory>,
  /// Helper for fractal_mem to lookup the actual location of the relevant data, since
  /// instantiating HandlerMemory instances for each memory usage need is expensive
  either_mem: Vec<i64>,
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
    let mut fractal_mem = vec![];
    let mut either_mem = vec![];
    if pls < 0 {
      // payload is a variable-length data type
      let payload: HandlerMemory = curr_hand_mem.fractal_mem[curr_addr as usize].clone();
      fractal_mem.push(payload);
      mem.push(0);
      either_mem.push(0);
    } else {
      // payload is a fixed length data type which could be in global memory
      mem.push(curr_hand_mem.read_fixed(curr_addr));
      either_mem.push(-1);
    };
    return Some(HandlerMemory {
      mem,
      fractal_mem,
      either_mem,
      gmem: curr_hand_mem.gmem,
      registers_ish: HashMap::new(),
    });
  }

  pub fn new(payload_mem: Option<HandlerMemory>,  mem_req: i64) -> HandlerMemory {
    let hand_mem = if payload_mem.is_none() {
      HandlerMemory {
        gmem: &Program::global().gmem,
        mem: vec![0; mem_req as usize],
        fractal_mem: vec![],
        either_mem: vec![-1; mem_req as usize],
        registers_ish: HashMap::new(),
      }
    } else {
      payload_mem.unwrap()
    };
    //hand_mem.mem.resize(mem_req as usize, 0);
    //hand_mem.fractal_mem.resize(mem_req as usize, HandlerMemory::new(None, 0));
    return hand_mem;
  }

  fn get_mut_arr(self: &mut HandlerMemory, addr: i64) -> &mut HandlerMemory {
    let arr = &mut self.fractal_mem[self.either_mem[addr as usize] as usize];
    return arr;
  }

  fn get_arr(self: &HandlerMemory, addr: i64) -> &HandlerMemory {
    let arr = &self.fractal_mem[self.either_mem[addr as usize] as usize];
    return arr;
  }

  /// set registerish and return its address
  pub fn set_reg(self: &mut HandlerMemory, reg_addr: i64, arr_addr1: i64, arr_addr2: i64) {
    let arr_addrs = vec![arr_addr1, arr_addr2];
    self.registers_ish.insert(reg_addr, arr_addrs);
  }

  /// The address provided can be a directly nested fractal or a registerish address that points to
  /// a fractal. Either returns a reference to a HandlerMemory
  pub fn get_fractal(self: &HandlerMemory, addr: i64) -> &HandlerMemory {
    let reg_opt = self.registers_ish.get(&addr);
    if reg_opt.is_none() {
      let arr = &self.fractal_mem[self.either_mem[addr as usize] as usize];
      return arr;
    }
    let reg = reg_opt.unwrap().to_vec();
    let mut arr = self.get_arr(reg[0]);
    for (i, addr) in reg.iter().enumerate() {
      if i == 0 { continue };
      arr = arr.get_arr(*addr);
    }
    return arr;
  }

  /// The address provided can be a directly nested fractal or a registerish address that points to
  /// a fractal. Either returns a reference to a HandlerMemory
  pub fn get_mut_fractal(self: &mut HandlerMemory, addr: i64) -> &mut HandlerMemory {
    let reg_opt = self.registers_ish.get(&addr);
    if reg_opt.is_none() {
      let arr = &mut self.fractal_mem[self.either_mem[addr as usize] as usize];
      return arr;
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
    let data = self.read_fixed(outer_addr);
    let arr = self.get_mut_fractal(arr_addr);
    arr.write_fixed(inner_addr, data);
  }

  pub fn copy_to_arr(self: &mut HandlerMemory, arr_addr: i64, outer_addr: i64, inner_addr: i64) {
    let data_copy = self.read_arr(outer_addr).to_vec();
    let arr = self.get_mut_fractal(arr_addr);
    arr.write_arr(inner_addr, data_copy.as_slice());
  }

  /// copy data from inner address in array to outer address. the array address can point to a
  /// registerish
  pub fn copy_from(self: &mut HandlerMemory, arr_addr: i64, outer_addr: i64, inner_addr: i64) {
    let arr = self.get_fractal(arr_addr);
    let (data, _size) = arr.read_either(inner_addr);
    if data.len() > 1 {
      self.write_arr(outer_addr, &data);
    } else {
      self.write_fixed(outer_addr, data[0]);
    }
  }

  pub fn copy_arr(self: &mut HandlerMemory, in_addr: i64, out_addr: i64) {
    let arr = self.get_mut_arr(in_addr);
    let new_arr = arr.clone();
    self.fractal_mem[self.either_mem[out_addr as usize] as usize] = new_arr;
  }

  pub fn len_as_arr(self: &HandlerMemory) -> usize {
    return self.mem.len();
  }

  pub fn new_arr(self: &mut HandlerMemory, addr: i64) {
    if self.either_mem[addr as usize] > 0 {
      panic!("Tried to create an array at address {}, but one already exists.", addr);
    }
    self.write_arr(addr, &[]);
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
    arr.either_mem.resize(idx + 1, -1);
  }

  pub fn push_arr_arr(self: &mut HandlerMemory, addr: i64, val: Vec<i64>) {
    let arr = self.get_mut_arr(addr);
    let idx = arr.mem.len();
    arr.mem.resize(idx + 1, 0);
    arr.write_arr(idx as i64, &val);
    arr.either_mem.resize(idx + 1, idx as i64);
  }

  pub fn push_arr_fractal(self: &mut HandlerMemory, addr: i64, val: HandlerMemory) {
    let arr = self.get_mut_arr(addr);
    let idx = arr.mem.len() as i64;
    arr.fractal_mem.push(val);
    arr.either_mem.push(idx);
  }

  /// removes the last value of the array in the address and returns it
  pub fn pop_arr(self: &mut HandlerMemory, addr: i64) -> Result<i64, HandlerMemory> {
    let arr = self.get_mut_arr(addr);
    let decision = arr.either_mem.pop().unwrap();
    if decision < 0 {
      return Ok(arr.mem.pop().unwrap());
    } else {
      arr.mem.pop();
      return Err(arr.fractal_mem.pop().unwrap());
    }
  }

  /// read address of string or fixed length data type and
  /// return a reference to the data and its size
  /// WARNING fails on reads to global memory, make sure it is not possible to pass this globals
  pub fn read_either(self: &HandlerMemory, addr: i64) -> (Vec<i64>, u8) {
    if addr < 0 {
      panic!("Reads of undefined size do not work on global memory");
    }
    // test if the data read is itself a string/array
    return if self.either_mem[addr as usize] > -1 {
      let var = self.read_arr(addr);
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
    unsafe {
      let mem_ptr = self.mem.as_ptr();
      return mem_ptr.add(addr as usize).read();
    }
  }

  pub fn read_arr(self: &HandlerMemory, addr: i64) -> Vec<i64> {
    if addr < 0 {
      let a = (0 - addr - 1) as usize;
      let result = &self.gmem[a..];
      let mut out: Vec<i64> = Vec::new();
      for i in 0..result.len() {
        let num = LittleEndian::read_i64(&result[8*i..8*i+8]);
        out.push(num);
      }
      return out;
    }
    let a = addr as usize;
    let arr = &self.fractal_mem[self.either_mem[a] as usize];
    let res = arr.mem.as_slice().to_vec();
    return res;
  }

  pub fn read_fractal(self: &HandlerMemory, addr: i64) -> HandlerMemory {
    if addr < 0 {
      panic!("Cannot treat global memory as fractal");
    }
    let a = addr as usize;
    let arr = self.fractal_mem[self.either_mem[a] as usize].clone();
    return arr;
  }

  pub fn write_fixed(self: &mut HandlerMemory, addr: i64, payload: i64) {
    // We can see a difference between the normal and unsafe forms of reading these integers in
    // benchmarking
    unsafe {
      let mem_ptr = self.mem.as_mut_ptr();
      *mem_ptr.add(addr as usize) = payload;
    }
  }

  pub fn write_arr(self: &mut HandlerMemory, addr: i64, payload: &[i64]) {
    let a = addr as usize;
    let arr = HandlerMemory {
      gmem: self.gmem,
      mem: payload.to_vec(),
      fractal_mem: vec![],
      either_mem: vec![-1; payload.len()],
      registers_ish: HashMap::new()
    };
    let idx = self.fractal_mem.len() as i64;
    self.either_mem[a] = idx;
    self.fractal_mem.push(arr);
  }

  pub fn write_fractal(self: &mut HandlerMemory, addr: i64, payload: HandlerMemory) {
    let idx = self.fractal_mem.len() as i64;
    self.either_mem[addr as usize] = idx;
    self.fractal_mem.push(payload);
  }
}