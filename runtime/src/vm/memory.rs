use std::collections::HashMap;

use crate::vm::program::Program;

/// Memory representation of a handler call
#[derive(Clone)]
pub struct HandlerMemory {
  /// Global memory reference
  gmem: &'static Vec<u8>,
  /// Memory of the handler for fixed size data types
  mem: Vec<u8>,
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
      mem = curr_hand_mem.read(curr_addr, pls as u8).to_vec();
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
        mem: vec![],
        fractal_mem: HashMap::new(),
        registers_ish: HashMap::new(),
      }
    } else {
      payload_mem.unwrap()
    };
    hand_mem.mem.resize(mem_req as usize, 0);
    return hand_mem;
  }

  fn get_mut_arr(self: &mut HandlerMemory, addr: i64) -> &mut HandlerMemory {
    let arr_opt = self.fractal_mem.get_mut(&addr);
    if arr_opt.is_none() {
      panic!("Array at address {} does not exist.", addr);
    };
    return arr_opt.unwrap();
  }

  fn get_arr(self: &HandlerMemory, addr: i64) -> &HandlerMemory {
    let arr_opt = self.fractal_mem.get(&addr);
    if arr_opt.is_none() {
      panic!("Array at address {} does not exist.", addr);
    };
    return arr_opt.unwrap();
  }

  /// set registerish and return its address
  pub fn set_reg(self: &mut HandlerMemory, reg_addr: i64, arr_addr1: i64, arr_addr2: Option<i64>) {
    let mut arr_addrs = vec![arr_addr1];
    if arr_addr2.is_some() { arr_addrs.push(arr_addr2.unwrap()) };
    self.registers_ish.insert(reg_addr, arr_addrs);
  }

  /// returns the HandlerMemory the registerish references
  pub fn get_reg(self: &HandlerMemory, addr: i64) -> &HandlerMemory {
    let reg_opt = self.registers_ish.get(&addr);
    if reg_opt.is_none() {
      panic!("Register at address {} does not exist.", addr);
    };
    let reg = reg_opt.unwrap().to_vec();
    let mut arr = self.get_arr(reg[0]);
    for (i, addr) in reg.iter().enumerate() {
      if i == 0 { continue };
      arr = arr.get_arr(*addr);
    }
    return arr;
  }

  /// returns the mutable HandlerMemory the registerish references
  pub fn get_mut_reg(self: &mut HandlerMemory, addr: i64) -> &mut HandlerMemory {
    let reg_opt = self.registers_ish.get(&addr);
    if reg_opt.is_none() {
      panic!("Register at address {} does not exist.", addr);
    };
    let reg = reg_opt.unwrap().to_vec();
    let mut arr = self.get_mut_arr(reg[0]);
    for (i, addr) in reg.iter().enumerate() {
      if i == 0 { continue };
      arr = arr.get_mut_arr(*addr);
    }
    return arr;
  }

  /// copy data from outer address to inner address in array or registerish
  pub fn copy_to(self: &mut HandlerMemory, arr_addr: i64, outer_addr:i64, inner_addr: i64) {
    let data = self.read_and_copy_either(outer_addr);
    self.write_to_arr(arr_addr, inner_addr, data);
  }

  /// copy data from inner address in array to outer address. the array address can point to a
  /// registerish
  pub fn copy_from(self: &mut HandlerMemory, arr_addr:i64, outer_addr:i64, inner_addr: i64) {
    let data = self.read_and_copy_from_arr(arr_addr, inner_addr);
    let size = data.len() as u8;
    self.write(outer_addr, size, data.as_slice());
  }

  pub fn len_arr(self: &HandlerMemory, addr: i64) -> usize {
    let arr = self.get_arr(addr);
    // string as array of u8
    if arr.mem.len() > 0 {
      return arr.mem.len();
    }
    // array of types
    return arr.fractal_mem.len();
  }

  pub fn ind_arr(self: &HandlerMemory, addr: i64, val: &[u8]) -> i64 {
    let arr = self.get_arr(addr);
    for (key, el) in arr.fractal_mem.iter() {
      if el.mem.len() == val.len() && el.mem.iter().eq(val.iter()) {
        return key.clone();
      }
    }
    return -1;
  }

  pub fn new_arr(self: &mut HandlerMemory, addr: i64) {
    let curr = self.read(addr, 0);
    if curr.len() > 0 {
      panic!("Tried to create an array at address {}, but one already exists.", addr);
    }
    self.write(addr, 0, &[]);
  }

  pub fn push_arr(self: &mut HandlerMemory, addr: i64, val: Vec<u8>, val_size: u8) {
    let arr = self.get_mut_arr(addr);
    let idx = arr.fractal_mem.len();
    arr.write(idx as i64, val_size, val.as_slice());
  }

  /// removes the last value of the array in the address and returns it
  pub fn pop_arr(self: &mut HandlerMemory, addr: i64) -> Vec<u8> {
    let arr = self.get_mut_arr(addr);
    let idx = arr.fractal_mem.len() - 1;
    let last_adrr = idx as i64;
    let last = arr.fractal_mem.remove(&last_adrr).unwrap();
    return last.mem;
  }

  /// write data from outer address to inner address in array or registerish
  pub fn write_to_arr(self: &mut HandlerMemory, arr_addr:i64, inner_addr: i64, data: Vec<u8>) {
    let arr = self.fractal_mem.get_mut(&arr_addr);
    let size = data.len() as u8;
    if arr.is_none() {
      let reg = self.get_mut_reg(arr_addr);
      reg.write(inner_addr, size, &data);
    } else {
      let fractal = arr.unwrap();
      fractal.write(inner_addr, size, &data);
    }
  }
  /// read data from inner address in array to outer address. the array address can point to a registerish
  pub fn read_and_copy_from_arr(self: &HandlerMemory, arr_addr:i64, inner_addr: i64) -> Vec<u8>{
    let arr = self.fractal_mem.get(&arr_addr);
    return if arr.is_none() {
      let reg = self.get_reg(arr_addr);
      reg.read_and_copy_either(inner_addr)
    } else {
      let fractal = arr.unwrap();
      fractal.read_and_copy_either(inner_addr)
    }
  }

  /// read address of string or fixed length data type and return a reference
  pub fn read_either(self: &HandlerMemory, addr: i64) -> &[u8] {
    // test if the data read is itself a string/array
    let var = self.read(addr, 0);
    return if var.len() > 0 { var } else {
      self.read(addr, 8)
    }
  }

  /// read address of string or fixed length data type and return a copy
  pub fn read_and_copy_either(self: &HandlerMemory, addr: i64) -> Vec<u8> {
    return self.read_either(addr).to_vec();
  }

  pub fn read(self: &HandlerMemory, addr: i64, size: u8) -> &[u8] {
    if addr < 0 {
      let a = (0 - addr - 1) as usize;
      let result = match size {
        0 => &self.gmem[a..],
        1 => &self.gmem[a..a + 1],
        2 => &self.gmem[a..a + 2],
        4 => &self.gmem[a..a + 4],
        8 => &self.gmem[a..a + 8],
        _ => panic!("Impossible size selection on global memory!"),
      };
      return result;
    }
    let a = addr as usize;
    let result = match size {
      0 => {
        let arr = self.fractal_mem.get(&addr);
        let res = if arr.is_none() { &[] } else { arr.unwrap().mem.as_slice() };
        return res;
      },
      1 => &self.mem[a..a + 1],
      2 => &self.mem[a..a + 2],
      4 => &self.mem[a..a + 4],
      8 => &self.mem[a..a + 8],
      _ => panic!("Impossible size selection on local memory!"),
    };
    return result;
  }

  pub fn write(self: &mut HandlerMemory, addr: i64, size: u8, payload: &[u8]) {
    if addr < 0 {
      panic!("You can't write to global memory!");
    }
    let a = addr as usize;
    match size {
      0 => {
        let arr = HandlerMemory {
          mem: payload.to_vec(),
          gmem: self.gmem,
          fractal_mem: HashMap::new(),
          registers_ish: HashMap::new()
        };
        self.fractal_mem.insert(addr, arr);
      },
      1 => self.mem[a] = payload[0],
      2 => {
        self.mem[a] = payload[0];
        self.mem[a + 1] = payload[1];
      },
      4 => {
        self.mem[a] = payload[0];
        self.mem[a + 1] = payload[1];
        self.mem[a + 2] = payload[2];
        self.mem[a + 3] = payload[3];
      },
      8 => {
        self.mem[a] = payload[0];
        self.mem[a + 1] = payload[1];
        self.mem[a + 2] = payload[2];
        self.mem[a + 3] = payload[3];
        self.mem[a + 4] = payload[4];
        self.mem[a + 5] = payload[5];
        self.mem[a + 6] = payload[6];
        self.mem[a + 7] = payload[7];
      },
      _ => panic!("Unexpected write of strange byte size!"),
    }
  }
}