use std::convert::TryInto;
use std::str;

use crate::vm::program::Program;

// -2^63
pub const CLOSURE_ARG_MEM_START: i64 = -9223372036854775808;
// The closure arg memory end has been extended to handle disambiguating nested closure arguments
// being used deep in the scope hierarchy. The quickest solution was to just increase that memory
// space to a large constant range, but the proper solution is to make this no longer a constant
// and determine the range based on the side of the global memory.
pub const CLOSURE_ARG_MEM_END: i64 = CLOSURE_ARG_MEM_START + 9001; // TODO: IT'S OVER 9000!
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
  /// The set of memory blocks. The first (zeroth) block hosts global memory and all blocks
  /// afterwards host memory created by the handler. Each block consists of tuples of two values,
  /// representing either a virtual pointer or raw data, with three classes of values:
  /// 1. `(usize::MAX, any value)` - The first value indicates that the second value is actual raw
  ///    data
  /// 2. `(< usize::MAX, usize::MAX)` - The first value indicates that this is a virtual pointer to
  ///    nested memory. The second value indicates that the pointer is to an entire block of
  ///    memory, not an explicit value.
  /// 3. `(< usize::MAX, < usize::MAX)` - The first value indicates that this is a virtual pointer
  ///    to nested memory. The second value indicates that the pointer is to an explicit value
  ///    within that block of memory.
  /// Virtual pointers are simply the indexes into the `mems` field.
  mems: Vec<Vec<(usize, i64)>>,
  /// The address spaces for the handler memory that the handler can mutate. The first is the
  /// "normal" memory space, and the second is the args memory space. Global addresses are fixed
  /// for the application and do not need a mutable vector to parse.
  addr: (Vec<(usize, usize)>, Vec<(usize, usize)>),
  /// Specifies which memory block to push "normal" values into.
  mem_addr: usize,
  /// Specifies which memory block to push "args" into.
  args_addr: usize,
}

impl HandlerMemory {
  /// Constructs a new HandlerMemory. If given another HandlerMemory it simply adjusts it to the
  /// expected memory needs, otherwise constructs a new one with said memory requirements.
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

  /// Because of constraints in Tokio's RWLock, this method moves the contents of `self` into the
  /// `target`. The `self` is destroyed in the process
  pub fn replace(self: HandlerMemory, target: &mut HandlerMemory) {
    target.mems = self.mems;
    target.addr = self.addr;
    target.mem_addr = self.mem_addr;
    target.args_addr = self.args_addr;
  }

  /// Grabs the relevant data for the event and constructs a new HandlerMemory with that value in
  /// address 0, or returns no HandlerMemory if it is a void event.
  pub fn alloc_payload(
    event_id: i64,
    curr_addr: i64,
    curr_hand_mem: &HandlerMemory,
  ) -> Option<HandlerMemory> {
    let pls = Program::global().event_pls.get(&event_id).unwrap().clone();
    return if pls == 0 {
      // no payload, void event
      None
    } else {
      let mut hm = HandlerMemory::new(None, 1);
      HandlerMemory::transfer(curr_hand_mem, curr_addr, &mut hm, 0);
      Some(hm)
    }
  }

  /// Takes a given address and looks up the `mems` indexes relevant to it.
  pub fn addr_to_idxs(self: &HandlerMemory, addr: i64) -> (usize, usize) {
    return if addr >= 0 {
     self.addr.0[addr as usize]
    } else if addr <= CLOSURE_ARG_MEM_END {
      self.addr.1[(addr - CLOSURE_ARG_MEM_START) as usize]
    } else {
      (0, ((-1 * addr - 1) / 8) as usize)
    }
  }

  /// Reads fixed data from a given address.
  pub fn read_fixed(self: &HandlerMemory, addr: i64) -> i64 {
    let (a, b) = self.addr_to_idxs(addr);
    return if a == std::usize::MAX {
      b as i64
    } else {
      self.mems[a][b].1
    }
  }

  /// Reads an array of data from the given address.
  pub fn read_fractal(self: &HandlerMemory, addr: i64) -> &[(usize, i64)] {
    let (a, b) = self.addr_to_idxs(addr);
    if addr_type(addr) == GMEM_ADDR {
      // Special behavior to read strings out of global memory
      &self.mems[a][b..]
    } else {
      &self.mems[a][..]
    }
  }

  /// Provides a mutable array of data from the given address.
  pub fn read_mut_fractal(self: &mut HandlerMemory, addr: i64) -> &mut Vec<(usize, i64)> {
    let (a, _) = self.addr_to_idxs(addr);
    &mut self.mems[a]
  }

  /// Provides a mutable array of data from the given indexes
  pub fn read_mut_fractal_idxs(self: &mut HandlerMemory, a: usize, _b: usize) -> &mut Vec<(usize, i64)> {
    &mut self.mems[a]
  }

  /// For a given address, determines if the data is a single value or an array of values, and
  /// returns that value either as a vector or the singular value wrapped in a vector, and a
  /// boolean indicating if it was a fractal value or not.
  pub fn read_either(self: &HandlerMemory, addr: i64) -> (Vec<(usize, i64)>, bool) {
    let (a, b) = self.addr_to_idxs(addr);
    return if b < std::usize::MAX {
      (vec![self.mems[a][b].clone()], false)
    } else {
      (self.mems[a].clone(), true)
    }
  }

  /// For a given set of `mems` indexes, determines if the data is a single value or an array of
  /// values, and returns that value either as a vector or the singular value wrapped in a vector,
  /// and a boolean indicating if it was a fractal value or not.
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

  /// Simply sets a given address to an explicit set of `mems` indexes. Simplifies pointer creation
  /// to deeply-nested data.
  pub fn set_addr(self: &mut HandlerMemory, addr: i64, a: usize, b: usize) {
    if addr_type(addr) == NORMAL_ADDR {
      let addru = addr as usize;
      if self.addr.0.len() <= addru {
        self.addr.0.resize(addru + 1, (std::usize::MAX, 0));
      }
      self.addr.0[addru] = (a, b);
    } else {
      let addru = (addr - CLOSURE_ARG_MEM_START) as usize;
      if self.addr.1.len() <= addru {
        self.addr.1.resize(addru + 1, (std::usize::MAX, 0));
      }
      self.addr.1[addru] = (a, b);
    }
  }

  /// Stores a fixed value in a given address. Determines where to place it based on the kind of
  /// address in question.
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

  /// Stores a nested fractal of data in a given address.
  pub fn write_fractal(self: &mut HandlerMemory, addr: i64, val: &[(usize, i64)]) {
    if addr >= 0 && self.addr.0.len() > (addr as usize) {
      let (a, b) = self.addr.0[addr as usize];
      if b == std::usize::MAX {
        let old_fractal = &self.mems[a];
        for i in 0..old_fractal.len() {
          if old_fractal[i].0 == self.mems.len() - 1 {
            drop(old_fractal);
            self.mems.pop();
            break;
          }
        }
        self.mems[a] = val.to_vec().clone();
        self.set_addr(addr, a, std::usize::MAX);
      } else {
        let a = self.mems.len();
        self.mems.push(val.to_vec().clone());
        self.set_addr(addr, a, std::usize::MAX);
      }
    } else if addr <= CLOSURE_ARG_MEM_END && self.addr.1.len() > ((addr - CLOSURE_ARG_MEM_START) as usize) {
      let (a, b) = self.addr.1[(addr - CLOSURE_ARG_MEM_START) as usize];
      if b == std::usize::MAX {
        let old_fractal = &self.mems[a];
        for i in 0..old_fractal.len() {
          if old_fractal[i].0 == self.mems.len() - 1 {
            drop(old_fractal);
            self.mems.pop();
            break;
          }
        }
        self.mems[a] = val.to_vec().clone();
        self.set_addr(addr, a, std::usize::MAX);
      } else {
        let a = self.mems.len();
        self.mems.push(val.to_vec().clone());
        self.set_addr(addr, a, std::usize::MAX);
      }
    } else {
      let a = self.mems.len();
      self.mems.push(val.to_vec().clone());
      self.set_addr(addr, a, std::usize::MAX);
    }
  }

  /// Pushes a fixed value into a fractal at a given address.
  pub fn push_fixed(self: &mut HandlerMemory, addr: i64, val: i64) {
    let mem = self.read_mut_fractal(addr);
    mem.push((std::usize::MAX, val));
  }

  /// Pushes a nested fractal value into a fractal at a given address.
  pub fn push_fractal(self: &mut HandlerMemory, addr: i64, val: &Vec<(usize, i64)>) {
    let a = self.mems.len();
    let mem = self.read_mut_fractal(addr);
    mem.push((a, std::usize::MAX as i64));
    self.mems.push(val.clone());
  }

  /// Pushes a pointer to an address into a fractal at a given address.
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

  /// Pushes raw `mems` indexes into a fractal at a given address. Allows pointers between fractal
  /// data when no explicit address exists between them.
  pub fn push_idxs(self: &mut HandlerMemory, addr: i64, a: usize, b: usize) {
    let mem = self.read_mut_fractal(addr);
    mem.push((a, b as i64));
  }

  /// Pops a value off of the fractal. May be fixed data or a virtual pointer.
  pub fn pop(self: &mut HandlerMemory, addr: i64) -> Result<(usize, i64), String> {
    let mem = self.read_mut_fractal(addr);
    if mem.len() > 0 {
      return Ok(mem.pop().unwrap());
    } else {
      return Err("cannot pop empty array".to_string());
    }
  }

  /// Deletes a value off of the fractal at the given idx. May be fixed data or a virtual pointer.
  pub fn delete(self: &mut HandlerMemory, addr: i64, idx: usize) -> Result<(usize, i64), String> {
    let mem = self.read_mut_fractal(addr);
    if mem.len() > 0 && mem.len() > idx {
      return Ok(mem.remove(idx));
    } else {
      return Err(format!("cannot remove idx {} from array with length {}", idx, mem.len()));
    }
  }

  /// Creates an alias for data at one address in another address.
  pub fn register(self: &mut HandlerMemory, addr: i64, orig_addr: i64, is_variable: bool) {
    let (a, b) = self.addr_to_idxs(orig_addr);
    if addr_type(orig_addr) == GMEM_ADDR {
      if is_variable {
        // Special behavior to read strings out of global memory
        let string = HandlerMemory::fractal_to_string(&self.mems[a][b..]);
        self.write_fractal(addr, &HandlerMemory::str_to_fractal(&string));
      } else {
        self.set_addr(addr, a, b);
      }
    } else {
      self.set_addr(addr, a, b);
    }
  }

  /// Creates a pointer to a value stored in a fractal at the given address and offset and places
  /// it in the address space.
  pub fn register_in(self: &mut HandlerMemory, orig_addr: i64, fractal_addr: i64, offset_addr: i64) {
    let (a, b) = self.addr_to_idxs(orig_addr);
    let mem = self.read_mut_fractal(fractal_addr);
    mem[offset_addr as usize] = (a, b as i64);
  }

  /// Creates a pointer to a value in the address space inside of a fractal at the given address
  /// and offset. The inverse of `register_in`.
  pub fn register_out(self: &mut HandlerMemory, fractal_addr: i64, offset_addr: i64, out_addr: i64) {
    let (arr_a, _) = self.addr_to_idxs(fractal_addr);
    let mem = self.read_mut_fractal(fractal_addr);
    let (a, b) = mem[offset_addr as usize];
    if a < std::usize::MAX {
      self.set_addr(out_addr, a, b as usize);
    } else {
      self.set_addr(out_addr, arr_a, offset_addr as usize);
    }
  }

  /// Migrates data from one HandlerMemory at a given address to another HandlerMemory at another
  /// address. Used by many things.
  pub fn transfer(orig: &HandlerMemory, orig_addr: i64, dest: &mut HandlerMemory, dest_addr: i64) {
    let (a, b) = orig.addr_to_idxs(orig_addr);
    if addr_type(orig_addr) == GMEM_ADDR {
      // Special behavior for global memory transfers since it may be a single value or a string
      let mem_slice = &orig.mems[a][b..];
      // To make this distinction we're gonna do some tests on the memory and see if it evals as a
      // string or not. There is some ridiculously small possibility that this is going to make a
      // false positive though so TODO: either make global memory unambiguous or update all uses of
      // this function to provide a type hint.
      let len = mem_slice[0].1 as usize;
      if len == 0 { // Assume zero is not a string
        dest.write_fixed(dest_addr, mem_slice[0].1);
        return;
      }
      let mut s_bytes: Vec<u8> = Vec::new();
      for i in 1..mem_slice.len() {
        let mut b = mem_slice[i].1.clone().to_ne_bytes().to_vec();
        s_bytes.append(&mut b);
      }
      if len > s_bytes.len() {
        // Absolutely not correct
        dest.write_fixed(dest_addr, mem_slice[0].1);
        return;
      }
      let try_str = str::from_utf8(&s_bytes[0..len]);
      if try_str.is_err() {
        // Also not a string
        dest.write_fixed(dest_addr, mem_slice[0].1);
      } else {
        // Well, waddaya know!
        dest.write_fractal(dest_addr, &HandlerMemory::str_to_fractal(try_str.unwrap()));
        return;
      }
    }
    if a == std::usize::MAX {
      // It's direct fixed data, just copy it over
      dest.write_fixed(dest_addr, b as i64);
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

  /// Creates a duplicate of data at one address in the HandlerMemory in a new address. Makes the
  /// `clone` function in Alan possible.
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
 
  /// Creates a clone of the HandlerMemory but with a new vector to write all new addressed data
  /// into, useful for the eventual joining logic to prevent accidental duplication of shared
  /// references
  pub fn fork(self: &HandlerMemory) -> HandlerMemory {
    let mut hm = self.clone();
    let s = hm.mems.len();
    hm.mems.push(Vec::new());
    hm.mem_addr = s;
    hm.args_addr = s;
    return hm;
  }

  /// Joins two HandlerMemory structs back together. Assumes that the passed in handler memory was
  /// generated by a fork call. This process consumes the forked HandlerMemory, moving over the
  /// records created in the fork into the original and then "stitches up" the virtual memory
  /// pointers for anything pointing at newly-created data. This mechanism is faster but will keep
  /// around unreachable memory for longer. Whether or not this is the right trade-off will have to
  /// be seen by real-world usage.
  pub fn join(self: &mut HandlerMemory, mut hm: HandlerMemory) {
    let s = hm.mem_addr; // The initial block that will be transferred (plus all following blocks)
    let s2 = self.mems.len(); // The new address of the initial block
    let offset = s2 - s; // Assuming it was made by `fork` this should be positive or zero
    if hm.addr.1.len() > 0 {
      let (a, b) = hm.addr_to_idxs(CLOSURE_ARG_MEM_START); // The only address that can "escape"
      hm.mems.drain(..s); // Remove the irrelevant memory blocks
      self.mems.append(&mut hm.mems); // Append the relevant ones to the original HandlerMemory
      // Set the return address on the original HandlerMemory to the acquired indexes, potentially
      // offset if it is a pointer at new data
      if a < std::usize::MAX && a >= s {
        self.set_addr(CLOSURE_ARG_MEM_START, a + offset, b);
      } else {
        self.set_addr(CLOSURE_ARG_MEM_START, a, b);
      }
    } else {
      hm.mems.drain(..s); // Remove the irrelevant memory blocks
      self.mems.append(&mut hm.mems); // Append the relevant ones to the original HandlerMemory
    }
    // Similarly "stitch up" every pointer in the moved data with a pass-through scan and update
    let l = self.mems.len();
    for i in s2..l {
      let mem = &mut self.mems[i];
      for j in 0..mem.len() {
        let (a, b) = mem[j];
        if a < std::usize::MAX && a >= s {
          mem[j] = (a + offset, b);
        }
      }
    }
    // Finally pull any addresses added by the old object into the new with a similar stitching
    if hm.addr.0.len() > self.addr.0.len() {
      self.addr.0.resize(hm.addr.0.len(), (0, 0));
    }
    for i in 0..hm.addr.0.len() {
      let (a, b) = hm.addr.0[i];
      let (c, _d) = self.addr.0[i];
      if a != std::usize::MAX && (a >= c || c == std::usize::MAX) {
        if a + offset >= s && a != std::usize::MAX {
          self.addr.0[i] = (a + offset, b);
        } else {
          self.addr.0[i] = (a, b);
        }
      } else if a == c {
        self.addr.0[i] = (a, b);
      }
    }
  }

  /// Takes a UTF-8 string and converts it to fractal memory that can be stored inside of a
  /// HandlerMemory. Alan stores strings as Pascal strings with a 64-bit length prefix. There is no
  /// computer on the planet that has 64-bits worth of RAM, so this should work for quite a while
  /// into the future. :)
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

  /// Takes a fractal memory and treats it like a UTF-8 encoded Pascal string, and the converts it
  /// to something Rust can work with. This function *may* crash if the underlying data is not a
  /// UTF-8 encoded Pascal string.
  pub fn fractal_to_string(f: &[(usize, i64)]) -> String {
    let s_len = f[0].1 as usize;
    let mut s_bytes: Vec<u8> = Vec::new();
    for i in 1..f.len() {
      let mut b = f[i].1.to_ne_bytes().to_vec();
      s_bytes.append(&mut b);
    }
    let s = str::from_utf8(&s_bytes[0..s_len]).unwrap();
    s.to_string()
  }
}
