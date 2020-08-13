use std::collections::HashMap;
use std::convert::TryInto;
use std::fmt;

use regex::Regex;

use crate::vm::program::Program;

// -2^63
pub const CLOSURE_ARG_MEM_START: i64 = -9223372036854775808;
pub const CLOSURE_ARG_MEM_SIZE: usize = 4;

/// Memory representation of a handler call
#[derive(Clone)]
pub struct HandlerMemory {
  /// Global memory reference
  gmem: &'static Vec<u8>,
  /// Memory of the handler for fixed size data types
  pub mem: Vec<i64>,
  /// Memory space used for closure arguments
  closure_args: Vec<i64>,
  fractal_closure_args: Vec<HandlerMemory>,
  /// Fractal memory storage for variable-length data types to an instance of the same struct
  pub fractal_mem: Vec<HandlerMemory>,
  /// Helper for fractal_mem or fractal_closure_args to lookup the actual location of the relevant data, since
  /// instantiating HandlerMemory instances for each memory usage need is expensive. -1 means the
  /// data is in mem, otherwise it's mapping to the index in fractal_mem that houses the relevant
  /// data.
  either_mem: Vec<i64>,
  pub either_closure_mem: Vec<i64>,
  /// Pointers to nested fractal HandlerMemory. Each is represented as a vector of up to 3 sequential addresses.
  /// These are not quite registers since they are not used by opcodes directly and they
  /// don't store the data itself, but an address to the data.
  registers_ish: HashMap<i64, Vec<i64>>,
  /// Temporary hack
  pub is_fixed: bool,
}

impl fmt::Display for HandlerMemory {
  fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
    let mut out_str = "".to_string();
    for i in 0..self.mem.len() {
      if self.either_mem[i] < 0 {
        out_str = out_str + &i.to_string() + ": " + &self.mem[i].to_string() + "\n"
      } else {
        let nested_str = format!("{}", &self.fractal_mem[self.either_mem[i] as usize]);
        let re = Regex::new("\n").unwrap();
        let indented_str = re.replace_all(&nested_str, "\n  ");
        out_str = out_str + &i.to_string() + ": " + &indented_str.to_string() + "\n"
      }
    }
    formatter.write_str(&out_str)
  }
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
      let payload: HandlerMemory = curr_hand_mem.fractal_mem[
        curr_hand_mem.either_mem[curr_addr as usize] as usize
      ].clone();
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
      closure_args: vec![0; CLOSURE_ARG_MEM_SIZE],
      fractal_closure_args: Vec::new(),
      either_closure_mem: vec![-1; CLOSURE_ARG_MEM_SIZE],
      gmem: curr_hand_mem.gmem,
      registers_ish: HashMap::new(),
      is_fixed: false,
    });
  }

  pub fn new(payload_mem: Option<HandlerMemory>,  mem_req: i64) -> HandlerMemory {
    let mem_size = mem_req as usize;
    let mut hand_mem = if payload_mem.is_none() {
      HandlerMemory {
        gmem: &Program::global().gmem,
        mem: vec![0; mem_size],
        fractal_mem: vec![],
        closure_args: vec![0; CLOSURE_ARG_MEM_SIZE],
        either_closure_mem: vec![-1; CLOSURE_ARG_MEM_SIZE],
        fractal_closure_args: Vec::new(),
        either_mem: vec![-1; mem_size],
        registers_ish: HashMap::new(),
        is_fixed: false,
      }
    } else {
      payload_mem.unwrap()
    };
    hand_mem.mem.resize(mem_size, 0);
    hand_mem.closure_args.resize(CLOSURE_ARG_MEM_SIZE, 0);
    hand_mem.either_closure_mem.resize(CLOSURE_ARG_MEM_SIZE, -1);
    hand_mem.either_mem.resize(mem_size, -1);
    hand_mem.fractal_mem.resize(mem_size, HandlerMemory {
      gmem: &hand_mem.gmem,
      mem: Vec::new(),
      fractal_mem: Vec::new(),
      either_mem: Vec::new(),
      closure_args: vec![0; CLOSURE_ARG_MEM_SIZE],
      either_closure_mem: vec![-1; CLOSURE_ARG_MEM_SIZE],
      fractal_closure_args: Vec::new(),
      registers_ish: HashMap::new(),
      is_fixed: false,
    });
    return hand_mem;
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
    let mut arr = &self.fractal_mem[self.either_mem[addr as usize] as usize];
    for (i, addr) in reg.iter().enumerate() {
      if i == 0 { continue };
      arr = &arr.fractal_mem[arr.either_mem[*addr as usize] as usize];
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
    let mut arr = &mut self.fractal_mem[self.either_mem[reg[0] as usize] as usize];
    for (i, addr) in reg.iter().enumerate() {
      if i == 0 { continue };
      arr = &mut arr.fractal_mem[arr.either_mem[*addr as usize] as usize];
    }
    return arr;
  }

  /// copy data from outer address to inner address in array or registerish
  pub fn copy_to_fixed(self: &mut HandlerMemory, arr_addr: i64, outer_addr: i64, inner_addr: i64) {
    let data = self.read_fixed(outer_addr);
    let arr = self.get_mut_fractal(arr_addr);
    arr.write_fixed(inner_addr, data);
  }

  pub fn copy_to_fractal_mem(self: &mut HandlerMemory, arr_addr: i64, outer_addr: i64, inner_addr: i64) {
    let data_copy = self.read_fractal_mem(outer_addr);
    let arr = self.get_mut_fractal(arr_addr);
    arr.write_fractal_mem(inner_addr, data_copy.as_slice());
  }

  /// copy data from inner address in array to outer address. the array address can point to a
  /// registerish
  pub fn copy_from(self: &mut HandlerMemory, arr_addr: i64, outer_addr: i64, inner_addr: i64) {
    let arr = self.get_fractal(arr_addr);
    let (data, size) = arr.read_either(inner_addr);
    if size == 0 {
      let inner_arr = arr.read_fractal(inner_addr);
      //println!("copy from fractal: @{}[{}] to {} val: {}", arr_addr, inner_addr, outer_addr, inner_arr.clone());
      self.write_fractal(outer_addr, inner_arr.clone());
    } else {
      //println!("copy from fixed: @{}[{}] to {} val {}", arr_addr, inner_addr, outer_addr, data[0]);
      self.write_fixed(outer_addr, data[0]);
    }
  }

  pub fn copy_fractal(self: &mut HandlerMemory, in_addr: i64, out_addr: i64) {
    let arr = &mut self.fractal_mem[self.either_mem[in_addr as usize] as usize];
    let new_arr = arr.clone();
    //println!("copy fractal from @{} to @{}: {}", in_addr, out_addr, new_arr);
    let fractal_addr = self.either_mem[out_addr as usize];
    if fractal_addr > -1 {
      self.fractal_mem[fractal_addr as usize] = new_arr;
    } else {
      // TODO: This shouldn't be happening, the compiler shouldn't be emitting `copyarr` for `void`
      // types. Once fixed the branching here can be removed.
      let new_addr = self.fractal_mem.len() as i64;
      self.fractal_mem.push(new_arr);
      self.either_mem[out_addr as usize] = new_addr;
    }
  }

  pub fn len(self: &HandlerMemory) -> usize {
    return self.mem.len();
  }

  pub fn has_nested_fractals(self: &HandlerMemory) -> bool {
    let mem_sum: i64 = self.mem.iter().sum();
    return mem_sum == 0;
  }

  pub fn new_fractal(self: &mut HandlerMemory, addr: i64) {
    if self.either_mem[addr as usize] > 0 {
      panic!("Tried to create an array at address {}, but one already exists.", addr);
    }
    //println!("create fractal: @{}", addr);
    self.write_fractal_mem(addr, &[]);
  }

  pub fn push_fractal_fixed(self: &mut HandlerMemory, addr: i64, val: i64) {
    // This implementation uses the `mem` vector as a way to keep track of the total length of the
    // array, as well. It's simple but wastes space when the inserted value is variable-length
    // (such as other strings or other arrays), however it greatly simplifies addressing and
    // lookup, particularly for `Array<any>`, which is also what user-defined types are transformed
    // into. In the future we could have an address translation layer and pack the data as tightly
    // as we can, assuming that doesn't impose a large performance penalty, while this simple
    // solution only adds an extra key's worth of space usage, but does have memory copy issues due
    // to the constant resizing.
    let arr = &mut self.fractal_mem[self.either_mem[addr as usize] as usize];
    let idx = arr.mem.len();
    arr.mem.push(0);
    arr.either_mem.push(-1);
    arr.write_fixed(idx as i64, val);
    //println!("push fixed: @{}[{}]: {}", addr, idx, val);
  }

  pub fn push_nested_fractal_mem(self: &mut HandlerMemory, addr: i64, val: Vec<i64>) {
    let arr = &mut self.fractal_mem[self.either_mem[addr as usize] as usize];
    let idx = arr.mem.len() as i64;
    arr.mem.push(0);
    arr.either_mem.push(idx);
    arr.write_fractal_mem(idx, &val);
    //println!("push nested mem: @{}[{}]: {}", addr, idx, val[0]);
  }

  pub fn push_nested_fractal(self: &mut HandlerMemory, addr: i64, val: HandlerMemory) {
    let arr = &mut self.fractal_mem[self.either_mem[addr as usize] as usize];
    let idx = arr.fractal_mem.len() as i64;
    //println!("push nested: @{}[{}]: {}", addr, arr.mem.len(), val);
    arr.mem.push(0);
    arr.either_mem.push(idx);
    arr.fractal_mem.push(val);
  }

  /// removes the last value of the array in the address and returns it
  pub fn pop_fractal(self: &mut HandlerMemory, addr: i64) -> Result<HandlerMemory, String> {
    // There's probably a more elegant way of doing this, but...
    let arr = &mut self.fractal_mem[self.either_mem[addr as usize] as usize];
    if arr.mem.len() == 0 {
      return Err("cannot pop empty array".to_string());
    } else {
      let decision = arr.either_mem.pop().unwrap();
      if decision < 0 {
        let mut frac = HandlerMemory::new(None, 1);
        frac.write_fixed(0, arr.mem.pop().unwrap());
        // This is a really shitty side-channel signal that it's fixed data
        frac.either_closure_mem[0] = 1;
        return Ok(frac);
      } else {
        arr.mem.pop();
        return Ok(arr.fractal_mem.pop().unwrap());
      }
    }
  }

  /// read address of string or fixed length data type and
  /// return a reference to the data and its size
  pub fn read_either(self: &HandlerMemory, addr: i64) -> (Vec<i64>, u8) {
    if addr < 0 && self.is_neg_addr_gmem(addr) {
      panic!("Reads of undefined size do not work on global memory");
    }
    /*if !self.is_neg_addr_gmem(addr) {
      return if self.either_closure_mem[(addr - CLOSURE_ARG_MEM_START) as usize] > -1 {
        let var = self.read_fractal_mem(addr);
        (var, 0)
      } else {
        // Nope, it's fixed data. We can safely read 8 bytes for all of the fixed types
        (vec![self.read_fixed(addr)], 8)
      };
    }*/
    // test if the data read is itself a string/array
    return if self.either_mem[addr as usize] > -1 {
      let var = self.read_fractal_mem(addr);
      (var, 0)
    } else {
      // Nope, it's fixed data. We can safely read 8 bytes for all of the fixed types
      (vec![self.read_fixed(addr)], 8)
    };
  }

  pub fn read_fixed(self: &HandlerMemory, addr: i64) -> i64 {
    if addr < 0 {
      // global memory
      if self.is_neg_addr_gmem(addr) {
        let a = (0 - addr - 1) as usize;
        let result = i64::from_ne_bytes((&self.gmem[a..a+8]).try_into().unwrap());
        return result;
      }
      // closure arguments memory
      let a = (addr - CLOSURE_ARG_MEM_START) as usize;
      unsafe {
        return self.closure_args.as_ptr().add(a).read();
      }
    }
    unsafe {
      return self.mem.as_ptr().add(addr as usize).read();
    }
  }

  // returns a copy
  pub fn read_fractal_mem(self: &HandlerMemory, addr: i64) -> Vec<i64> {
    if addr < 0 {
      // global memory
      if self.is_neg_addr_gmem(addr) {
        let a = (0 - addr - 1) as usize;
        let result = &self.gmem[a..];
        let mut out: Vec<i64> = Vec::new();
        for i in 0..(result.len() / 8) {
          let num = i64::from_ne_bytes((&result[8*i..8*i+8]).try_into().unwrap());
          out.push(num);
        }
        return out;
      }
      // closure arguments memory
      let a = (addr - CLOSURE_ARG_MEM_START) as usize;
      let arr = &self.fractal_closure_args[self.either_closure_mem[a] as usize];
      let res = arr.mem.as_slice();
      return res.to_vec();
    }
    let a = addr as usize;
    let arr = &self.fractal_mem[self.either_mem[a] as usize];
    let res = arr.mem.as_slice();
    return res.to_vec();
  }

  fn is_neg_addr_gmem(self: &HandlerMemory, addr: i64) -> bool {
    // avoids overflow on subtract
    return addr != CLOSURE_ARG_MEM_START && (0 - addr - 1) as usize <= self.gmem.len();
  }

  // returns a copy while get_fractal returns a reference
  pub fn read_fractal(self: &HandlerMemory, addr: i64) -> HandlerMemory {
    if addr < 0 {
      // string from global memory
      if self.is_neg_addr_gmem(addr) {
        let a = (0 - addr - 1) as usize;
        let result = &self.gmem[a..];
        let mut out: Vec<i64> = Vec::new();
        for i in 0..(result.len() / 8) {
          let num = i64::from_ne_bytes((&result[8*i..8*i+8]).try_into().unwrap());
          out.push(num);
        }
        let len = out.len();
        return HandlerMemory {
          gmem: &self.gmem,
          mem: out,
          closure_args: vec![0; CLOSURE_ARG_MEM_SIZE],
          either_closure_mem: vec![-1; CLOSURE_ARG_MEM_SIZE],
          fractal_closure_args: Vec::new(),
          fractal_mem: Vec::new(),
          either_mem: vec![-1; len],
          registers_ish: HashMap::new(),
          is_fixed: false,
        }
      } else {
        // string from closure arguments memory
        let a = (addr - CLOSURE_ARG_MEM_START) as usize;
        if self.either_closure_mem[a] > -1 {
          return self.fractal_closure_args[self.either_closure_mem[a] as usize].clone();
        } else {
          let out = self.closure_args[a..].to_vec();
          let len = out.len();
          return HandlerMemory {
            gmem: &self.gmem,
            mem: Vec::new(),
            closure_args: out,
            either_closure_mem: vec![-1; len],
            fractal_closure_args: Vec::new(),
            fractal_mem: Vec::new(),
            either_mem: Vec::new(),
            registers_ish: HashMap::new(),
            is_fixed: false,
          }
        }
      }
    }
    let a = addr as usize;
    let arr = self.fractal_mem[self.either_mem[a] as usize].clone();
    return arr;
  }

  pub fn write_fixed(self: &mut HandlerMemory, addr: i64, payload: i64) {
    if addr < 0  {
      if self.is_neg_addr_gmem(addr) {
        panic!("Cannot write to global memory");
      }
      // closure arguments memory
      let a = (addr - CLOSURE_ARG_MEM_START) as usize;
      unsafe {
        let mem_ptr = self.closure_args.as_mut_ptr();
        *mem_ptr.add(a) = payload;
      }
    } else {
      // We can see a difference between the normal and unsafe forms of reading these integers in
      // benchmarking
      unsafe {
        let mem_ptr = self.mem.as_mut_ptr();
        *mem_ptr.add(addr as usize) = payload;
      }
    }
  }

  // new fractal from mem
  pub fn write_fractal_mem(self: &mut HandlerMemory, addr: i64, payload: &[i64]) {
    let arr = HandlerMemory {
      gmem: self.gmem,
      mem: payload.to_vec(),
      closure_args: vec![0; CLOSURE_ARG_MEM_SIZE],
      either_closure_mem: vec![-1; CLOSURE_ARG_MEM_SIZE],
      fractal_closure_args: Vec::new(),
      fractal_mem: vec![],
      either_mem: vec![-1; payload.len()],
      registers_ish: HashMap::new(),
      is_fixed: false,
    };
    self.write_fractal(addr, arr);
  }

  pub fn write_fractal(self: &mut HandlerMemory, addr: i64, payload: HandlerMemory) {
    if addr < 0 && !self.is_neg_addr_gmem(addr) {
      let a = (addr - CLOSURE_ARG_MEM_START) as usize;
      let idx = self.fractal_closure_args.len() as i64;
      self.either_closure_mem[a] = idx;
      self.fractal_closure_args.push(payload);
    } else {
      let idx = self.fractal_mem.len() as i64;
      self.either_mem[addr as usize] = idx;
      self.fractal_mem.push(payload);
    }
  }
}