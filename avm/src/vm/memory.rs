use std::convert::TryInto;
use std::str;

use crate::vm::program::Program;

// -2^63
pub const CLOSURE_ARG_MEM_START: i64 = -9223372036854775808;
pub const CLOSURE_ARG_MEM_SIZE: usize = 4;
pub const CLOSURE_ARG_MEM_END: i64 = CLOSURE_ARG_MEM_START + 3;
// Flags for the registers_ish vector. The normal address flag indicates that the data is stored
// normally in either the memory or fractal memory structures. The fixed pointer address flag
// indicates that the value in the memory structure is actually a pointer to an i64 value. The
// handlermemory pointer address flag indicates that the value in the memory structure is actually
// a pointer to a HandlerMemory object.
const NORMAL_ADDR: i8 = 0;
const GMEM_ADDR: i8 = 1;
const ARGS_ADDR: i8 = 2;

fn addr_type(addr: i64) -> i8 {
  return if addr >= 0 {
    NORMAL_ADDR
  } else if addr <= CLOSURE_ARG_MEM_END {
    ARGS_ADDR
  } else {
    GMEM_ADDR
  }
}

/// Memory representation of a handler call
#[derive(Clone)]
#[derive(Debug)]
pub struct HandlerMemory {
  /// The set of memory blocks. Each block consists of a tuple of two values. If the first value is
  /// zero then the second value is the actual data, if it is non-zero then the two values together
  /// represent a virtual pointer to another memory block and value (if the second value is zero,
  /// it represents a pointer to a nested array of data, otherwise a pointer to an explicit value)
  mems: Vec<Vec<(usize, i64)>>,
  /// The address spaces for the handler memory. The first  is the "normal" memory space, and the
  /// second is the args memory space.
  addr: (Vec<(usize, usize)>, Vec<(usize, usize)>),
  /// Specifies which memory block to push "normal" values into
  mem_addr: usize,
  /// Specifies which memory block to push "args" into
  args_addr: usize,
}

impl HandlerMemory {
  pub fn new(payload_mem: Option<HandlerMemory>, mem_req: i64) -> HandlerMemory {
    if payload_mem.is_none() {
      let mut hm = HandlerMemory {
        mems: [Program::global().gmem.clone(), Vec::new()].to_vec(),
        addr: (Vec::new(), Vec::new()),
        mem_addr: 1,
        args_addr: 1,
      };
      hm.mems[1].reserve(mem_req as usize);
      hm
    } else {
      let mut hm = payload_mem.unwrap();
      hm.mems[1].reserve(mem_req as usize);
      hm
    }
  }

  pub fn alloc_payload(
    event_id: i64,
    curr_addr: i64,
    curr_hand_mem: &HandlerMemory,
  ) -> Option<HandlerMemory> {
    let pls = Program::global().event_pls.get(&event_id).unwrap().clone();
    return if pls == 0 {
      // no payload, void event
      None
    } else if pls < 0 {
      // payload is a variable-length data type
      let mut hm = HandlerMemory::new(None, 1);
      hm.write_fractal(0, curr_hand_mem.read_fractal(curr_addr));
      Some(hm)
    } else {
      // payload is a fixed length data type which could be in global memory
      let mut hm = HandlerMemory::new(None, 1);
      hm.write_fixed(0, curr_hand_mem.read_fixed(curr_addr));
      Some(hm)
    }
  }

  pub fn addr_to_idxs(self: &HandlerMemory, addr: i64) -> (usize, usize) {
    return if addr >= 0 {
     self.addr.0[addr as usize]
    } else if addr <= CLOSURE_ARG_MEM_END {
      self.addr.1[(CLOSURE_ARG_MEM_START - addr) as usize]
    } else {
      (0, ((-1 * addr - 1) / 8) as usize)
    }
  }

  pub fn read_fixed(self: &HandlerMemory, addr: i64) -> i64 {
    let (a, b) = self.addr_to_idxs(addr);
    return if a == std::usize::MAX {
      b as i64
    } else {
      self.mems[a][b].1
    }
  }

  pub fn read_fractal(self: &HandlerMemory, addr: i64) -> &[(usize, i64)] {
    let (a, b) = self.addr_to_idxs(addr);
    if addr_type(addr) == GMEM_ADDR {
      // Special behavior to read strings out of global memory
      &self.mems[a][b..]
    } else {
      &self.mems[a][..]
    }
  }

  pub fn read_mut_fractal(self: &mut HandlerMemory, addr: i64) -> &mut Vec<(usize, i64)> {
    let (a, _) = self.addr_to_idxs(addr);
    &mut self.mems[a]
  }

  pub fn read_either(self: &HandlerMemory, addr: i64) -> (Vec<(usize, i64)>, bool) {
    let (a, b) = self.addr_to_idxs(addr);
    return if b < std::usize::MAX {
      (vec![self.mems[a][b].clone()], false)
    } else {
      (self.mems[a].clone(), true)
    }
  }

  pub fn read_either_idxs(self: &HandlerMemory, a: usize, b: usize) -> (Vec<(usize, i64)>, bool) {
    return if a == std::usize::MAX {
      // The indexes are the actual data
      (vec![(a, b as i64)], false)
    } else if b < std::usize::MAX {
      // The indexes point to fixed data
      (vec![self.mems[a][b].clone()], false)
    } else {
      // The indexes point at nested data
      (self.mems[a].clone(), true)
    }
  }


  fn set_addr(self: &mut HandlerMemory, addr: i64, a: usize, b: usize) {
    if addr_type(addr) == NORMAL_ADDR {
      let addru = addr as usize;
      if self.addr.0.len() <= addru {
        self.addr.0.resize(addru + 1, (std::usize::MAX, 0));
      }
      self.addr.0[addru] = (a, b);
    } else {
      let addru = (CLOSURE_ARG_MEM_START - addr) as usize;
      if self.addr.1.len() <= addru {
        self.addr.1.resize(addru + 1, (std::usize::MAX, 0));
      }
      self.addr.1[addru] = (a, b);
    }
  }

  pub fn write_fixed(self: &mut HandlerMemory, addr: i64, val: i64) {
    let a = if addr_type(addr) == NORMAL_ADDR {
      self.mem_addr
    } else {
      self.args_addr
    };
    let b = self.mems[a].len();
    self.mems[a].push((std::usize::MAX, val));
    self.set_addr(addr, a, b);
  }

  pub fn write_fractal(self: &mut HandlerMemory, addr: i64, val: &[(usize, i64)]) {
    let a = self.mems.len();
    self.mems.push(val.to_vec().clone());
    self.set_addr(addr, a, std::usize::MAX);
  }

  pub fn push_fixed(self: &mut HandlerMemory, addr: i64, val: i64) {
    let mem = self.read_mut_fractal(addr);
    mem.push((std::usize::MAX, val));
  }

  pub fn push_fractal(self: &mut HandlerMemory, addr: i64, val: &Vec<(usize, i64)>) {
    let a = self.mems.len();
    let mem = self.read_mut_fractal(addr);
    mem.push((a, std::usize::MAX as i64));
    self.mems.push(val.clone());
  }

  pub fn push_register(self: &mut HandlerMemory, addr: i64, other_addr: i64) {
    let (a, b) = self.addr_to_idxs(other_addr);
    // Special path for strings in global memory
    if a == 0 {
      let strmem = self.mems[0][b..].to_vec().clone();
      let new_a = self.mems.len();
      self.mems.push(strmem);
      let mem = self.read_mut_fractal(addr);
      mem.push((new_a, std::usize::MAX as i64));
    } else {
      let mem = self.read_mut_fractal(addr);
      mem.push((a, b as i64));
    }
  }

  pub fn push_idxs(self: &mut HandlerMemory, addr: i64, a: usize, b: usize) {
    let mem = self.read_mut_fractal(addr);
    mem.push((a, b as i64));
  }

  pub fn pop(self: &mut HandlerMemory, addr: i64) -> Result<(usize, i64), String> {
    let mem = self.read_mut_fractal(addr);
    if mem.len() > 0 {
      return Ok(mem.pop().unwrap());
    } else {
      return Err("cannot pop empty array".to_string());
    }
  }

  pub fn register(self: &mut HandlerMemory, addr: i64, orig_addr: i64) {
    let (a, b) = self.addr_to_idxs(orig_addr);
    self.set_addr(addr, a, b);
  }

  pub fn register_in(self: &mut HandlerMemory, addr: i64, fractal_addr: i64, offset_addr: i64) {
    let mem = self.read_fractal(fractal_addr);
    let (a, b) = mem[offset_addr as usize];
    if a < std::usize::MAX {
      self.set_addr(addr, a, b as usize);
    } else {
      let (a, _) = self.addr_to_idxs(fractal_addr);
      self.set_addr(addr, a, offset_addr as usize);
    }
  }

  pub fn register_out(self: &mut HandlerMemory, fractal_addr: i64, offset_addr: i64, orig_addr: i64) {
    let mem = self.read_mut_fractal(fractal_addr);
    let (a, b) = mem[offset_addr as usize];
    self.set_addr(orig_addr, a, b as usize);
  }

  pub fn transfer(orig: &HandlerMemory, orig_addr: i64, dest: &mut HandlerMemory, dest_addr: i64) {
    let (a_orig, b_orig) = orig.addr_to_idxs(orig_addr);
    let (a, b) = orig.mems[a_orig][b_orig];
    if a == std::usize::MAX {
      // It's direct fixed data, just copy it over
      dest.write_fixed(dest_addr, b);
    } else if a < std::usize::MAX && (b as usize) < std::usize::MAX {
      // All pointers are made shallow, so we know this is a pointer to a fixed value and just
      // grab it and de-reference it.
      let (_, b_nest) = orig.mems[a][b as usize];
      dest.write_fixed(dest_addr, b_nest);
    } else if a < std::usize::MAX && b as usize == std::usize::MAX {
      // It's a nested array of data. This may itself contain references to other nested arrays of
      // data and is relatively complex to transfer over. First create some lookup vectors and
      // populate them with the nested fractal, adding more and more as each fractal is checked
      // until no new ones are added
      let mut check_idx = 0;
      let mut orig_arr_addrs: Vec<usize> = vec![a];
      let mut orig_arr_copies: Vec<Vec<(usize, i64)>> = vec![orig.read_fractal(orig_addr).to_vec().clone()];
      while check_idx < orig_arr_addrs.len() {
        let arr = &orig_arr_copies[check_idx];
        let l = arr.len();
        drop(arr);
        for i in 0..l {
          let other_arr_idx = orig_arr_copies[check_idx][i].0.clone();
          if other_arr_idx < std::usize::MAX {
            if !orig_arr_addrs.contains(&other_arr_idx) {
              orig_arr_addrs.push(other_arr_idx);
              orig_arr_copies.push(orig.mems[other_arr_idx].clone());
            }
          }
        }
        check_idx = check_idx + 1;
      }
      // Next, get the current size of the destination mems vector to use as the offset to add to
      // the index of the copied arrays, updating their interior references, if any, in the process
      let dest_offset = dest.mems.len();
      for i in 0..orig_arr_copies.len() {
        let arr = &mut orig_arr_copies[i];
        for j in 0..arr.len() {
          let (a, b) = arr[j];
          if a < std::usize::MAX {
            for k in 0..orig_arr_addrs.len() {
              if orig_arr_addrs[k] == a {
                arr[j] = (dest_offset + k, b);
              }
            }
          }
        }
      }
      dest.mems.append(&mut orig_arr_copies);
      // Finally, set the destination address to point at the original, main nested array
      dest.set_addr(dest_addr, dest_offset, std::usize::MAX);
    }
  }

  pub fn dupe(self: &mut HandlerMemory, orig_addr: i64, dest_addr: i64) {
    // This *should be possible with something like this:
    // HandlerMemory::transfer(self, orig_addr, self, dest_addr);
    // But Rust's borrow checker doesn't like it, so we basically have to replicate the code here
    // despite the fact that it should work just fine...
    let (a, b) = self.addr_to_idxs(orig_addr);
    if a == std::usize::MAX {
      self.write_fixed(dest_addr, b as i64);
    } else if a < std::usize::MAX && (b as usize) < std::usize::MAX {
      // All pointers are made shallow, so we know this is a pointer to a fixed value and just
      // grab it and de-reference it.
      let (_, b_nest) = self.mems[a][b];
      self.write_fixed(dest_addr, b_nest);
    } else if a < std::usize::MAX && b as usize == std::usize::MAX {
      // It's a nested array of data. This may itself contain references to other nested arrays of
      // data and is relatively complex to transfer over. First create some lookup vectors and
      // populate them with the nested fractal, adding more and more as each fractal is checked
      // until no new ones are added
      let mut check_idx = 0;
      let mut orig_arr_addrs: Vec<usize> = vec![a];
      let mut orig_arr_copies: Vec<Vec<(usize, i64)>> = vec![self.read_fractal(orig_addr).to_vec().clone()];
      while check_idx < orig_arr_addrs.len() {
        let arr = &orig_arr_copies[check_idx];
        let l = arr.len();
        drop(arr);
        for i in 0..l {
          let other_arr_idx = orig_arr_copies[check_idx][i].0.clone();
          if other_arr_idx < std::usize::MAX {
            if !orig_arr_addrs.contains(&other_arr_idx) {
              orig_arr_addrs.push(other_arr_idx);
              orig_arr_copies.push(self.mems[other_arr_idx].clone());
            }
          }
        }
        check_idx = check_idx + 1;
      }
      // Next, get the current size of the destination mems vector to use as the offset to add to
      // the index of the copied arrays, updating their interior references, if any, in the process
      let dest_offset = self.mems.len();
      for i in 0..orig_arr_copies.len() {
        let arr = &mut orig_arr_copies[i];
        for j in 0..arr.len() {
          let (a, b) = arr[j];
          if a < std::usize::MAX {
            for k in 0..orig_arr_addrs.len() {
              if orig_arr_addrs[k] == a {
                arr[j] = (dest_offset + k, b);
              }
            }
          }
        }
      }
      self.mems.append(&mut orig_arr_copies);
      // Finally, set the destination address to point at the original, main nested array
      self.set_addr(dest_addr, dest_offset, std::usize::MAX);
    }
  }

  pub fn str_to_fractal(s: &str) -> Vec<(usize, i64)> {
    let mut s_mem = vec![(std::usize::MAX, s.len() as i64)];
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
        s_mem.push((std::usize::MAX, i64::from_ne_bytes(s_slice.try_into().unwrap())));
        i = i + 8;
      } else {
        break
      }
    }
    s_mem
  }

  pub fn fractal_to_string(f: &[(usize, i64)]) -> String {
    let s_len = f[0].1 as usize;
    let mut s_bytes: Vec<u8> = Vec::new();
    for i in 1..f.len() {
      let mut b = f[i].1.clone().to_ne_bytes().to_vec();
      s_bytes.append(&mut b);
    }
    let s = str::from_utf8(&s_bytes[0..s_len]).unwrap();
    s.to_string()
  }
}
