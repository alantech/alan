use std::convert::TryInto;
use std::fmt;
use std::slice;
use std::str;

use regex::Regex;

use crate::vm::program::Program;

// -2^63
pub const CLOSURE_ARG_MEM_START: i64 = -9223372036854775808;
pub const CLOSURE_ARG_MEM_SIZE: usize = 4;
// Flags for the registers_ish vector. The normal address flag indicates that the data is stored
// normally in either the memory or fractal memory structures. The fixed pointer address flag
// indicates that the value in the memory structure is actually a pointer to an i64 value. The
// handlermemory pointer address flag indicates that the value in the memory structure is actually
// a pointer to a HandlerMemory object.
const NORMAL_ADDR: i8 = 0;
const FX_PTR_ADDR: i8 = 1;
const HM_PTR_ADDR: i8 = 2;

/// Memory representation of a handler call
#[derive(Clone)]
pub struct HandlerMemory {
  /// Global memory reference
  gmem: &'static Vec<u8>,
  /// Memory of the handler for fixed size data types
  pub mem: Vec<i64>,
  /// Fractal memory storage for variable-length data types to an instance of the same struct
  pub fractal_mem: Vec<HandlerMemory>,
  /// Helper for fractal_mem to lookup the actual location of the relevant data, since instantiating
  /// HandlerMemory instances for each memory usage need is expensive. -1 means the data is in mem,
  /// otherwise it's mapping to the index in fractal_mem that houses the relevant data.
  pub either_mem: Vec<i64>,
  /// Flag indicating the memory stored is actually a pointer
  registers_ish: Vec<i8>,
  /// Temporary hack
  pub is_fixed: bool,
  /// Memory space used for closure arguments
  closure_args: Vec<HandlerMemory>, // Option doesn't work for some reason
}

impl fmt::Display for HandlerMemory {
  fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
    let mut out_str = "".to_string();
    for i in 0..self.mem.len() {
      if self.either_mem[i] < 0 {
        if self.registers_ish[i] == 0 {
          out_str = out_str + &i.to_string() + ": " + &self.mem[i].to_string() + "\n"
        } else if self.registers_ish[i] == FX_PTR_ADDR {
          unsafe {
            let ptr = usize::from_ne_bytes(self.mem[i].to_ne_bytes()) as *const i64;
            let val = *ptr;
            out_str = out_str + &i.to_string() + ": *" + &val.to_string() + "\n"
          }
        } else {
          unsafe {
            let ptr = usize::from_ne_bytes(self.mem[i].to_ne_bytes()) as *const HandlerMemory;
            let hm = ptr.as_ref().unwrap();
            let nested_str = format!("{}", &hm);
            let re = Regex::new("\n").unwrap();
            let indented_str = re.replace_all(&nested_str, "\n  ");
            out_str = out_str + &i.to_string() + ": **" + &indented_str.to_string() + "\n"
          }
        }
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
    let mut registers_ish: Vec<i8> = vec![];
    if pls < 0 {
      // payload is a variable-length data type
      let mut mem_copy = curr_hand_mem.clone();
      mem_copy.copy_fractal(curr_addr, curr_addr);
      let payload = mem_copy.read_fractal(curr_addr);
      fractal_mem.push(payload);
      mem.push(0);
      either_mem.push(0);
      registers_ish.push(0);
    } else {
      // payload is a fixed length data type which could be in global memory
      mem.push(curr_hand_mem.read_fixed(curr_addr));
      either_mem.push(-1);
      registers_ish.push(0);
    };
    return Some(HandlerMemory {
      gmem: curr_hand_mem.gmem,
      mem,
      fractal_mem,
      either_mem,
      registers_ish,
      is_fixed: false,
      closure_args: Vec::new(),
    });
  }

  pub fn new(payload_mem: Option<HandlerMemory>,  mem_req: i64) -> HandlerMemory {
    let mem_size = mem_req as usize;
    let mut hand_mem = if payload_mem.is_none() {
      HandlerMemory {
        gmem: &Program::global().gmem,
        mem: vec![0; mem_size],
        fractal_mem: vec![],
        either_mem: vec![-1; mem_size],
        registers_ish: vec![0; mem_size],
        is_fixed: false,
        closure_args: Vec::new(),
      }
    } else {
      payload_mem.unwrap()
    };
    hand_mem.mem.resize(mem_size, 0);
    hand_mem.either_mem.resize(mem_size, -1);
    hand_mem.registers_ish.resize(mem_size, 0);
    hand_mem.fractal_mem.resize(mem_size, HandlerMemory {
      gmem: &hand_mem.gmem,
      mem: Vec::new(),
      fractal_mem: Vec::new(),
      either_mem: Vec::new(),
      registers_ish: Vec::new(),
      is_fixed: false,
      closure_args: Vec::new(),
    });
    return hand_mem;
  }

  /// allow an opcode to declare a HandlerMemory a closure
  pub fn make_closure(self: &mut HandlerMemory) {
    self.closure_args.push(HandlerMemory {
      gmem: &self.gmem,
      mem: vec![0; CLOSURE_ARG_MEM_SIZE],
      fractal_mem: vec![],
      either_mem: vec![-1; CLOSURE_ARG_MEM_SIZE],
      registers_ish: vec![0; CLOSURE_ARG_MEM_SIZE],
      is_fixed: false,
      closure_args: Vec::new(),
    });
  }

  /// set registerish and return its address
  pub fn set_reg(self: &mut HandlerMemory, reg_addr: i64, arr_addr1: i64, arr_addr2: i64) {
    unsafe {
      let ptr;
      let reg = reg_addr as usize;
      if reg_addr < 0 { // It's a closure arg
        let real_reg = (reg_addr - CLOSURE_ARG_MEM_START) as usize;
        let reg_type;
        let arr = self.get_fractal(arr_addr1);
        if arr.registers_ish[arr_addr2 as usize] > 0 {
          // It's a pointer, too return that
          let arr2 = arr.clone();
          let closure = &mut self.closure_args[0];
          closure.registers_ish[real_reg] = arr2.registers_ish[arr_addr2 as usize];
          closure.mem[real_reg] = arr2.mem[arr_addr2 as usize];
        } else {
          if arr.either_mem[arr_addr2 as usize] == -1 {
            ptr = arr.mem.as_ptr().add(arr_addr2 as usize) as i64;
            reg_type = FX_PTR_ADDR;
          } else {
            ptr = arr.fractal_mem.as_ptr().add(arr.either_mem[arr_addr2 as usize] as usize) as i64;
            reg_type = HM_PTR_ADDR;
          }
          let closure = &mut self.closure_args[0];
          closure.registers_ish[real_reg] = reg_type;
          closure.mem[real_reg] = ptr;
        }
      } else { 
        let arr = self.get_fractal(arr_addr1);
        if arr.registers_ish[arr_addr2 as usize] > 0 {
          // Just return the pointer
          let arr2 = arr.clone();
          self.registers_ish[reg] = arr2.registers_ish[arr_addr2 as usize];
          self.either_mem[reg] = -1;
          self.mem[reg] = arr2.mem[arr_addr2 as usize];
        } else {
          // Make a pointer to this data
          if arr.either_mem[arr_addr2 as usize] == -1 {
            ptr = arr.mem.as_ptr().add(arr_addr2 as usize) as i64;
            self.registers_ish[reg] = FX_PTR_ADDR;
            self.either_mem[reg] = -1;
          } else {
            ptr = arr.fractal_mem.as_ptr().add(arr.either_mem[arr_addr2 as usize] as usize) as i64;
            self.registers_ish[reg] = HM_PTR_ADDR;
            self.either_mem[reg] = -1;
          }
          self.mem[reg] = ptr;
        }
      }
    }
  }

  pub fn store_reg(self: &mut HandlerMemory, reg_addr: i64, other_addr: i64) {
    let ptr: *const HandlerMemory = self.get_fractal(other_addr);
    self.registers_ish[reg_addr as usize] = HM_PTR_ADDR;
    self.mem[reg_addr as usize] = ptr as i64;
  }

  pub fn push_reg(self: &mut HandlerMemory, push_addr: i64, other_addr: i64) {
    let ptr: *const HandlerMemory = self.get_fractal(other_addr);
    let push_mem = self.get_mut_fractal(push_addr);
    push_mem.mem.push(ptr as i64);
    push_mem.registers_ish.push(HM_PTR_ADDR);
    push_mem.either_mem.push(-1);
  }

  /// optionally create a registerish within a nested Result-like HandlerMemory, or leave an error
  /// message. This is here instead of in the opcodes because of the special access it needs to
  /// private fields
  pub fn res_from(self: &mut HandlerMemory, arr_addr: i64, arr_idx_arr: i64, outer_addr: i64) {
    self.new_fractal(outer_addr);
    self.push_fractal_fixed(outer_addr, 1i64);
    let inner_addr = self.read_fixed(arr_idx_arr);
    let arr = self.get_fractal(arr_addr);
    if inner_addr >= 0 && arr.len() as i64 > inner_addr {
      // Valid result
      unsafe {
        let ptr;
        let ptr_type;
        if arr.registers_ish[inner_addr as usize] == 0 {
          if arr.either_mem[inner_addr as usize] == -1 {
            ptr = arr.mem.as_ptr().add(inner_addr as usize) as i64;
            ptr_type = FX_PTR_ADDR;
          } else {
            ptr = arr.fractal_mem.as_ptr().add(arr.either_mem[inner_addr as usize] as usize) as i64;
            ptr_type = HM_PTR_ADDR;
          }
        } else {
          ptr = arr.mem[inner_addr as usize];
          ptr_type = arr.registers_ish[inner_addr as usize];
        }
        self.push_fractal_fixed(outer_addr, 0i64);
        let res_arr = self.get_mut_fractal(outer_addr);
        res_arr.registers_ish[1] = ptr_type;
        res_arr.mem[1] = ptr;
      }
    } else {
      // Out-of-bounds access
      let error_string = "out-of-bounds access";
      let mut out = vec![error_string.len() as i64];
      let mut out_str_bytes = error_string.as_bytes().to_vec();
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
      let arr = self.get_mut_fractal(outer_addr);
      arr.mem[0] = 0;
      self.push_nested_fractal_mem(outer_addr, out);
    }
  }

  /// The address provided can be a directly nested fractal or a registerish address that points to
  /// a fractal. Either returns a reference to a HandlerMemory
  pub fn get_fractal(self: &HandlerMemory, addr: i64) -> &HandlerMemory {
    if addr < 0 { // Assume it's a closure arg
      let closure = &self.closure_args[0];
      let arr = closure.get_fractal(addr - CLOSURE_ARG_MEM_START);
      return arr;
    } else {
      let reg = self.registers_ish[addr as usize];
      if reg == NORMAL_ADDR {
        if self.either_mem[addr as usize] < 0 {
          panic!("Trying to get a fractal from fixed memory");
        }
        let arr = &self.fractal_mem[self.either_mem[addr as usize] as usize];
        return arr;
      } else if reg == HM_PTR_ADDR {
        unsafe {
          let ptr = usize::from_ne_bytes(self.mem[addr as usize].to_ne_bytes()) as *const HandlerMemory;
          let hm = ptr.as_ref().unwrap();
          return hm;
        }
      } else {
        panic!("Trying to get a fractal from a fixed pointer");
      }
    }
  }

  /// The address provided can be a directly nested fractal or a registerish address that points to
  /// a fractal. Either returns a reference to a HandlerMemory
  pub fn get_mut_fractal(self: &mut HandlerMemory, addr: i64) -> &mut HandlerMemory {
    let reg = self.registers_ish[addr as usize];
    if reg == NORMAL_ADDR {
      let arr = &mut self.fractal_mem[self.either_mem[addr as usize] as usize];
      return arr;
    } else if reg == HM_PTR_ADDR {
      unsafe {
        let ptr = usize::from_ne_bytes(self.mem[addr as usize].to_ne_bytes()) as *mut HandlerMemory;
        let hm = ptr.as_mut().unwrap();
        return hm;
      }
    } else {
      panic!("Trying to get a fractal from a fixed pointer");
    }
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
    let mut arr = self.read_fractal(in_addr);
    let l = arr.mem.len() as i64;
    for i in 0..l {
      if arr.registers_ish[i as usize] == FX_PTR_ADDR {
        let val = arr.read_fixed(i);
        arr.write_fixed(i, val);
        arr.registers_ish[i as usize] = 0;
      } else if arr.registers_ish[i as usize] == HM_PTR_ADDR {
        arr.copy_fractal(i, i);
        arr.registers_ish[i as usize] = 0;
      }
    }
    if out_addr < 0 {
      // It's a closure arg
      let fractal_addr = self.closure_args[0].either_mem[(out_addr - CLOSURE_ARG_MEM_START) as usize];
      if fractal_addr > -1 {
        self.closure_args[0].fractal_mem[fractal_addr as usize] = arr;
      } else {
        let new_addr = self.closure_args[0].fractal_mem.len() as i64;
        self.closure_args[0].fractal_mem.push(arr);
        self.closure_args[0].either_mem[(out_addr - CLOSURE_ARG_MEM_START) as usize] = new_addr;
        self.closure_args[0].registers_ish.push(0);
      }
    } else {
      let fractal_addr = self.either_mem[out_addr as usize];
      if fractal_addr > -1 {
        self.fractal_mem[fractal_addr as usize] = arr;
      } else {
        // TODO: This shouldn't be happening, the compiler shouldn't be emitting `copyarr` for `void`
        // types. Once fixed the branching here can be removed.
        let new_addr = self.fractal_mem.len() as i64;
        self.fractal_mem.push(arr);
        self.either_mem[out_addr as usize] = new_addr;
        self.registers_ish.push(0);
      }
    }
  }

  pub fn len(self: &HandlerMemory) -> usize {
    return self.mem.len();
  }

  pub fn has_nested_fractals(self: &HandlerMemory) -> bool {
    return self.mem.len() > 0 && (self.fractal_mem.len() > 0 || self.registers_ish.contains(&1) ||
      self.registers_ish.contains(&2))
  }

  pub fn new_fractal(self: &mut HandlerMemory, addr: i64) {
    // TODO: See if this can be brought back safely with mutating array closure functions
    /*if self.either_mem[addr as usize] > 0 {
      panic!("Tried to create an array at address {}, but one already exists.", addr);
    }*/
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
    let arr = self.get_mut_fractal(addr);
    let idx = arr.mem.len();
    arr.mem.push(0);
    arr.either_mem.push(-1);
    arr.registers_ish.push(0);
    arr.write_fixed(idx as i64, val);
    //println!("push fixed: @{}[{}]: {}", addr, idx, val);
  }

  pub fn push_nested_fractal_mem(self: &mut HandlerMemory, addr: i64, val: Vec<i64>) {
    let arr = self.get_mut_fractal(addr);
    let idx = arr.mem.len() as i64;
    arr.mem.push(0);
    arr.either_mem.push(idx);
    arr.registers_ish.push(0);
    arr.write_fractal_mem(idx, &val);
    //println!("push nested mem: @{}[{}]: {}", addr, idx, val[0]);
  }

  pub fn push_nested_fractal(self: &mut HandlerMemory, addr: i64, val: HandlerMemory) {
    let arr = self.get_mut_fractal(addr);
    let idx = arr.fractal_mem.len() as i64;
    //println!("push nested: @{}[{}]: {}", addr, arr.mem.len(), val);
    arr.mem.push(0);
    arr.either_mem.push(idx);
    arr.fractal_mem.push(val);
    arr.registers_ish.push(0);
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
        frac.is_fixed = true;
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
      // assume it's fixed memory
      return (vec![self.read_fixed(addr)], 8);
    } else if addr < 0 { // It's closure memory
      return self.closure_args[0].read_either(addr - CLOSURE_ARG_MEM_START);
    }
    // first check if the data is stored in a pointer
    if self.registers_ish[addr as usize] > 0 {
      return if self.registers_ish[addr as usize] == HM_PTR_ADDR {
        let var = self.read_fractal_mem(addr);
        (var, 0)
      } else {
        (vec![self.read_fixed(addr)], 8)
      }
    }
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
      let closure = &self.closure_args[0];
      return closure.read_fixed(addr - CLOSURE_ARG_MEM_START);
    }
    let reg = self.registers_ish[addr as usize];
    if reg == NORMAL_ADDR {
      unsafe {
        return self.mem.as_ptr().add(addr as usize).read();
      }
    } else if reg == FX_PTR_ADDR {
      unsafe {
        let ptr = usize::from_ne_bytes(self.mem[addr as usize].to_ne_bytes()) as *const i64;
        let val = *ptr;
        return val;
      }
    } else {
      panic!("Trying to get fixed value from fractal pointer");
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
      let closure = &self.closure_args[0];
      return closure.read_fractal_mem(addr - CLOSURE_ARG_MEM_START);
    }
    let reg = self.registers_ish[addr as usize];
    if reg == NORMAL_ADDR {
      let a = addr as usize;
      let arr = &self.fractal_mem[self.either_mem[a] as usize];
      let res = arr.mem.as_slice();
      return res.to_vec();
    } else {
      let frac_mem = self.get_fractal(addr).mem.clone();
      return frac_mem;
    }
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
          fractal_mem: Vec::new(),
          either_mem: vec![-1; len],
          registers_ish: vec![0; len],
          is_fixed: false,
          closure_args: Vec::new(),
        }
      } else {
        // string from closure arguments memory
        let closure = &self.closure_args[0];
        return closure.read_fractal(addr - CLOSURE_ARG_MEM_START);
      }
    }
    let frac = self.get_fractal(addr);
    return frac.clone();
  }

  pub fn write_fixed(self: &mut HandlerMemory, addr: i64, payload: i64) {
    if addr < 0  {
      if self.is_neg_addr_gmem(addr) {
        panic!("Cannot write to global memory");
      }
      // closure arguments memory
      let closure = &mut self.closure_args[0];
      return closure.write_fixed(addr - CLOSURE_ARG_MEM_START, payload);
    } else if self.registers_ish[addr as usize] > 0 {
      // It's a pointer to fixed memory
      unsafe {
        let ptr = usize::from_ne_bytes(self.mem[addr as usize].to_ne_bytes()) as *mut i64;
        *ptr = payload;
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
      fractal_mem: vec![],
      either_mem: vec![-1; payload.len()],
      registers_ish: vec![0; payload.len()],
      is_fixed: false,
      closure_args: Vec::new(),
    };
    self.write_fractal(addr, arr);
  }

  pub fn write_fractal(self: &mut HandlerMemory, addr: i64, payload: HandlerMemory) {
    if addr < 0 && !self.is_neg_addr_gmem(addr) {
      let closure = &mut self.closure_args[0];
      return closure.write_fractal(addr - CLOSURE_ARG_MEM_START, payload);
    } else {
      let idx = self.fractal_mem.len() as i64;
      self.either_mem[addr as usize] = idx;
      self.fractal_mem.push(payload);
      self.registers_ish.push(0);
    }
  }

  pub fn str_to_hm(s: &str) -> HandlerMemory {
    let mut s_mem = vec![s.len() as i64];
    let mut s_bytes = s.as_bytes().to_vec();
    loop {
      if s_bytes.len() % 8 != 0 {
        s_bytes.push(0);
      } else {
        break
      }
    }
    let mut i = 0;
    loop {
      if i < s_bytes.len() {
        let s_slice = &s_bytes[i..i+8];
        s_mem.push(i64::from_ne_bytes(s_slice.try_into().unwrap()));
        i = i + 8;
      } else {
        break
      }
    }
    let mem_size = s_mem.len();
    HandlerMemory {
      gmem: &Program::global().gmem,
      mem: s_mem,
      fractal_mem: vec![],
      either_mem: vec![-1; mem_size],
      registers_ish: vec![0; mem_size],
      is_fixed: false,
      closure_args: Vec::new(),
    }
  }

  pub fn hm_to_string(self: &HandlerMemory) -> String {
    let s_len = self.mem[0] as usize;
    unsafe {
      let s_u8 = slice::from_raw_parts(self.mem[1..].as_ptr().cast::<u8>(), s_len*8);
      let s = str::from_utf8(&s_u8[0..s_len]).unwrap();
      s.to_string()
    }
  }
}