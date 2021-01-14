use std::convert::TryInto;
use std::str;
use std::sync::Arc;

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

/// Memory representation of a fractal memory block within HandlerMemory
#[derive(Clone, Debug)]
pub struct FractalMemory {
  // address in HandlerMemory which is not present for actual data or deeply nested fractals
  hm_addr: Option<i64>,
  // a memory block from HandlerMemory.mems
  block: Vec<(usize, i64)>,
  hm_id: usize,
  pub is_fractal: bool,
}

impl FractalMemory {
  pub fn new(block: Vec<(usize, i64)>) -> FractalMemory {
    return FractalMemory {
      hm_addr: None,
      hm_id: 0 as *const HandlerMemory as usize, // null ptr
      is_fractal: false,
      block,
    }
  }

  pub fn belongs(self: &FractalMemory, hm: &HandlerMemory) -> bool {
    return self.hm_id == 0 || self.hm_id == hm as *const HandlerMemory as usize;
  }

  /// Length of memory block
  pub fn len(self: &FractalMemory) -> usize {
    return self.block.len();
  }

  /// Compare the blocks at a given index between two FractalMemory
  pub fn compare_at(self: &FractalMemory, idx: usize, other: &FractalMemory) -> bool {
    return self.block[idx] == other.block[idx];
  }

  /// Reads fixed data from a given address.
  pub fn read_fixed(self: &FractalMemory, idx: usize) -> i64 {
    if self.block[idx].0 != usize::MAX {
      panic!("Trying to read raw data from memory when it is a pointer")
    }
    return self.block[idx].1;
  }
}

impl PartialEq for FractalMemory {
  fn eq(&self, other: &Self) -> bool {
    // ignore hm_addr
    self.block == other.block
  }
}

/// Memory representation of a handler call
#[derive(Clone, Debug)]
pub struct HandlerMemory {
  /// Optional parent pointer present in forked HandlerMemories
  parent: Option<Arc<HandlerMemory>>,
  /// The set of memory blocks. Each representing a fractal. The first (zeroth) block hosts global memory and all blocks
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
  /// The virtual address spaces to an (optional) index pointing to mems that the handler can mutate.
  /// If the index is not defined in the current Handler Memory, it is in the parent Handler Memory.
  /// The first is the "normal" memory space, and the second is the args memory space.
  /// Global addresses are fixed for the application and do not need a mutable vector to parse.
  addr: (Vec<Option<(usize, usize)>>, Vec<Option<(usize, usize)>>),
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
        parent: None,
      };
      hm.mems[1].reserve(mem_req as usize);
      hm
    } else {
      let mut hm = payload_mem.unwrap();
      hm.mems[1].reserve(mem_req as usize);
      hm
    }
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

  pub fn drop_parent(self: &mut HandlerMemory) {
    if self.parent.is_some() {
      let arc = self.parent.take();
      drop(arc);
    }
  }

  fn is_idx_defined(self: &HandlerMemory, a: usize, b: usize) -> bool {
    // println!("a: {}, self.mem_addr: {}", a, self.mem_addr);
    // println!("{:?}", self);
    let is_raw = a == std::usize::MAX;
    let safe_mem_space = self.mem_addr == 1 || a >= self.mem_addr; // account for init_fractal
    let is_fixed = safe_mem_space && self.mems.len() > a && self.mems[a].len() > b;
    let is_fractal = safe_mem_space && self.mems.len() > a && b == std::usize::MAX;
    return is_raw || is_fixed || is_fractal;
  }

  // returns None if the idxs belong to self
  fn hm_for_idxs(self: &HandlerMemory, a: usize, b: usize) -> Option<&Arc<HandlerMemory>> {
    //println!("self mems.len: {}, self.mem_addr: {}, a,b: {},{}", self.mems.len(),self.mem_addr, a, b);
    if self.is_idx_defined(a,b) {
      return None;
    }
    //println!("parent mems.len: {}, a,b: {},{}", self.parent.as_ref().unwrap().mems.len(), a, b);
    let res = self.parent.as_ref().unwrap().hm_for_idxs(a, b);
    return if res.is_none() { self.parent.as_ref() } else { res };
  }

  fn addr_to_idxs_opt(self: &HandlerMemory, addr: i64) -> Option<(usize, usize)> {
    return if addr >= 0 {
      *self.addr.0.get(addr as usize).unwrap_or(&None)
    } else if addr <= CLOSURE_ARG_MEM_END {
      *self.addr.1.get((addr - CLOSURE_ARG_MEM_START) as usize).unwrap_or(&None)
    } else {
      Some((0, ((-1 * addr - 1) / 8) as usize))
    };
  }

  /// Takes a given address and looks up the fractal location and
  /// `mems` indexes relevant to it. It also returns an Option that is
  /// none if the address is in self or a ptr to the ancestor
  fn addr_to_idxs(self: &HandlerMemory, addr: i64, ) -> ((usize, usize), Option<&Arc<HandlerMemory>>) {
    let idxs = if addr >= 0 {
      *self.addr.0.get(addr as usize).unwrap_or(&None)
    } else if addr <= CLOSURE_ARG_MEM_END {
      *self.addr.1.get((addr - CLOSURE_ARG_MEM_START) as usize).unwrap_or(&None)
    } else {
      Some((0, ((-1 * addr - 1) / 8) as usize))
    };
    //println!("idxs: {:?}", idxs);
    return if idxs.is_none() {
      // fail if no parent
      if self.parent.is_none() {
        panic!("Memory address referenced in parent, but no parent pointer defined");
      }
      // println!("recurse");
      let res = self.parent.as_ref().unwrap().addr_to_idxs(addr);
      let hm = if res.1.is_none() { self.parent.as_ref() } else { res.1 };
      (res.0, hm)
    } else {
      let res = idxs.unwrap();
      // if addr == -9223372036854775808 {
      //   println!("addr: {}, a: {}, b:{}, self.mem_addr: {}, self.mems.len: {}", addr, res.0, res.1, self.mem_addr, self.mems.len());
      // }
      (res, self.hm_for_idxs(res.0, res.1))
    };
  }

  /// Reads fixed data from a given address.
  pub fn read_fixed(self: &HandlerMemory, addr: i64) -> i64 {
    let ((a, b), hm_opt) = self.addr_to_idxs(addr);
    let hm = if hm_opt.is_none() { self } else { hm_opt.unwrap() };
    return if a == std::usize::MAX {
      b as i64
    } else {
      hm.mems[a][b].1
    }
  }

  /// Reads an array of data from the given address.
  pub fn read_fractal(self: &HandlerMemory, addr: i64) -> FractalMemory {
    let ((a, b), hm_opt) = self.addr_to_idxs(addr);
    //eprintln!("addr: {}, self?: {}, (a,b): ({},{})", addr, hm_opt.is_none(), a, b);
    let hm = if hm_opt.is_none() { self } else { hm_opt.unwrap() };
    return if addr_type(addr) == GMEM_ADDR {
      // Special behavior to read strings out of global memory
      FractalMemory {
        hm_addr: Some(addr),
        block: hm.mems[a][b..].to_vec(),
        hm_id: hm as *const HandlerMemory as usize,
        is_fractal: true,
      }
    } else {
      //eprintln!("{:?}", &hm);
      FractalMemory {
        hm_addr: Some(addr),
        block: hm.mems[a][..].to_vec(),
        hm_id: hm as *const HandlerMemory as usize,
        is_fractal: true,
      }
    };
  }

  /// Provides a mutable array of data from the given address.
  fn read_mut_fractal(self: &mut HandlerMemory, addr: i64) -> &mut Vec<(usize, i64)> {
    let ((a, _), hm_opt) = self.addr_to_idxs(addr);
    if hm_opt.is_some() {
      // copy necessary data from ancestor
      let hm = Arc::clone(hm_opt.unwrap());
      HandlerMemory::transfer(hm.as_ref(), addr, self, addr);
      drop(hm);
    }
    &mut self.mems[a]
  }

  /// For a given address, determines if the data is a single value or an array of values, and
  /// returns that value either as a vector or the singular value wrapped in a vector, and a
  /// boolean indicating if it was a fractal value or not.
  pub fn read_either(self: &HandlerMemory, addr: i64) -> (FractalMemory, bool) {
    let ((a, b), hm_opt) = self.addr_to_idxs(addr);
    // println!("READ_EITHER from self? {}", hm_opt.is_none());
    let hm = if hm_opt.is_none() { self } else { hm_opt.unwrap() };
    return if b < std::usize::MAX {
      (
        FractalMemory {
          hm_addr: Some(addr),
          block: vec![hm.mems[a][b].clone()],
          hm_id: hm as *const HandlerMemory as usize,
          is_fractal: false,
        }, false
      )
    } else {
      (
        FractalMemory {
          hm_addr: Some(addr),
          block: hm.mems[a].clone(),
          hm_id: hm as *const HandlerMemory as usize,
          is_fractal: true,
        }, true
      )
    }
  }

  /// Simply sets a given address to an explicit set of `mems` indexes. Simplifies pointer creation
  /// to deeply-nested data.
  fn set_addr(self: &mut HandlerMemory, addr: i64, a: usize, b: usize) {
    if addr_type(addr) == NORMAL_ADDR {
      let addru = addr as usize;
      if self.addr.0.len() <= addru {
        self.addr.0.resize(addru + 1, Some((std::usize::MAX, 0)));
      }
      self.addr.0[addru] = Some((a, b));
    } else {
      let addru = (addr - CLOSURE_ARG_MEM_START) as usize;
      if self.addr.1.len() <= addru {
        self.addr.1.resize(addru + 1, Some((std::usize::MAX, 0)));
      }
      self.addr.1[addru] = Some((a, b));
    }
  }

  /// For the memory block(s) starting at idx in Fractal, determines if the data is a single value or an array of
  /// values, and returns that value either as a vector or the singular value wrapped in a vector,
  /// and a boolean indicating if it is a fractal value or not.
  pub fn read_from_fractal(self: &HandlerMemory, fractal: &FractalMemory, idx: usize) -> (FractalMemory, bool) {
    let (a, b) = fractal.block[idx];
    let b_usize = b as usize;
    let hm_opt = self.hm_for_idxs(a, b as usize);
    let hm = if hm_opt.is_none() { self } else { hm_opt.unwrap() };
    return if a == std::usize::MAX {
      // The indexes are the actual data
      (
        FractalMemory::new(vec![(a, b)]),
        false
      )
    } else if b_usize < std::usize::MAX {
      // The indexes point to fixed data
      (
        FractalMemory {
          hm_addr: None,
          block: vec![hm.mems[a][b_usize].clone()],
          hm_id: hm as *const HandlerMemory as usize,
          is_fractal: false,
        }, false
      )
    } else {
      // The indexes point at nested data
      (
        FractalMemory {
          hm_addr: None,
          block: hm.mems[a].clone(),
          hm_id: hm as *const HandlerMemory as usize,
          is_fractal: true,
        }, true
      )
    }
  }

  /// Stores a nested fractal of data in a given address.
  pub fn write_fixed_in_fractal(self: &mut HandlerMemory, fractal: &mut FractalMemory, idx: usize, val: i64) {
    if !fractal.belongs(self) {
      panic!("Writing TO a forked/read-only FractalMemory is not allowed");
    }
    fractal.block[idx].1 = val;
    if fractal.hm_addr.is_some() {
      self.write_fractal(fractal.hm_addr.unwrap(), fractal);
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
  pub fn write_fractal(self: &mut HandlerMemory, addr: i64, fractal: &FractalMemory) {
    let a = self.mems.len();
    if !fractal.belongs(self) {
      if fractal.hm_addr.is_none() {
        panic!("Writing a forked/read-only FractalMemory that is also deeply-nested is not possible");
      }
      // copy fractal from ancestor
      let addr = fractal.hm_addr.as_ref().unwrap().clone();
      let (_, hm_opt) = self.addr_to_idxs(addr);
      let hm = Arc::clone(hm_opt.unwrap());
      HandlerMemory::transfer(hm.as_ref(), addr, self, addr);
      drop(hm);
    }
    self.mems.push(fractal.block.clone());
    self.set_addr(addr, a, std::usize::MAX);
  }

  /// Stores a nested empty fractal of data in a given address.
  pub fn init_fractal(self: &mut HandlerMemory, addr: i64) {
    let a = self.mems.len();
    self.mems.push(Vec::new());
    self.set_addr(addr, a, std::usize::MAX);
  }

  /// Pushes a fixed value into a fractal at a given address.
  pub fn push_fixed(self: &mut HandlerMemory, addr: i64, val: i64) {
    let mem = self.read_mut_fractal(addr);
    mem.push((std::usize::MAX, val));
  }

  /// Pushes a nested fractal value into a fractal at a given address.
  pub fn push_fractal(self: &mut HandlerMemory, addr: i64, val: FractalMemory) {
    let a = self.mems.len();
    let mem = self.read_mut_fractal(addr);
    mem.push((a, std::usize::MAX as i64));
    self.mems.push(val.block);
  }

  /// Pops a value off of the fractal. May be fixed data or a virtual pointer.
  pub fn pop(self: &mut HandlerMemory, addr: i64) -> Result<FractalMemory, String> {
    let mem = self.read_mut_fractal(addr);
    if mem.len() > 0 {
      return Ok(FractalMemory::new(vec![mem.pop().unwrap()]));
    } else {
      return Err("cannot pop empty array".to_string());
    }
  }

  /// Deletes a value off of the fractal at the given idx. May be fixed data or a virtual pointer.
  pub fn delete(self: &mut HandlerMemory, addr: i64, idx: usize) -> Result<FractalMemory, String> {
    let mem = self.read_mut_fractal(addr);
    if mem.len() > 0 && mem.len() > idx {
      return Ok(FractalMemory::new(vec![mem.remove(idx)]));
    } else {
      return Err(format!("cannot remove idx {} from array with length {}", idx, mem.len()));
    }
  }

  /* REGISTER MANIPULATION METHODS */

  /// Creates a pointer from `orig_addr` to `addr`
  pub fn register(self: &mut HandlerMemory, addr: i64, orig_addr: i64, is_variable: bool) {
    let ((a, b), _) = self.addr_to_idxs(orig_addr);
    if addr_type(orig_addr) == GMEM_ADDR && is_variable {
      // Special behavior to read strings out of global memory
      let string = HandlerMemory::fractal_to_string(FractalMemory::new(self.mems[a][b..].to_vec()));
      self.write_fractal(addr, &HandlerMemory::str_to_fractal(&string));
    } else {
      self.set_addr(addr, a, b);
    }
  }

  /// Pushes a pointer from `orig_addr` address into the fractal at `addr`.
  pub fn push_register(self: &mut HandlerMemory, addr: i64, orig_addr: i64) {
    let ((a, b), _) = self.addr_to_idxs(orig_addr);
    // Special path for strings in global memory which is the same for parent and self
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

  /// Creates a pointer from `orig_addr` to index/offset `offset_addr` of fractal in `fractal_addr`
  pub fn register_in(self: &mut HandlerMemory, orig_addr: i64, fractal_addr: i64, offset_addr: i64) {
    let ((a, b), _) = self.addr_to_idxs(orig_addr);
    let mem = self.read_mut_fractal(fractal_addr);
    mem[offset_addr as usize] = (a, b as i64);
  }

  /// Creates a pointer from index/offset `offset_addr` of fractal in `fractal_addr` to `out_addr`
  /// The inverse of `register_in`
  pub fn register_out(self: &mut HandlerMemory, fractal_addr: i64, offset_addr: usize, out_addr: i64) {
    let ((arr_a, _), _) = self.addr_to_idxs(fractal_addr);
    let fractal = self.read_fractal(fractal_addr);
    //println!("{:?}", fractal);
    let in_self = self as *const HandlerMemory as usize == fractal.hm_id;
    //println!("fractal_addr: {}, fractal_idx:{}, out_addr: {}, in_self: {}", fractal_addr, offset_addr, out_addr, in_self);
    let (a, b) = fractal.block[offset_addr];
    if a < std::usize::MAX {
      self.set_addr(out_addr, a, b as usize);
    } else {
      self.set_addr(out_addr, arr_a, offset_addr);
    }
  }

  /// Creates a pointer from index/offset `idx` in FractalMemory to `out_addr`
  /// Used for deeply nested fractals in which case `register_out` can't be used
  pub fn register_from_fractal(self: &mut HandlerMemory, out_addr: i64, fractal: &FractalMemory, idx: usize) {
    let (a, b) = fractal.block[idx];
    self.set_addr(out_addr, a, b as usize);
  }

  /// Pushes a pointer from index/offset `offset_addr` of FractalMemory to fractal at `out_addr`
  pub fn push_register_out(self: &mut HandlerMemory, out_addr: i64, fractal: &FractalMemory, offset_addr: usize) {
    let mem = self.read_mut_fractal(out_addr);
    mem.push(fractal.block[offset_addr]);
  }

  /* DATA TRANSFER, FORKING AND DUPLICATION METHODS */

  /// Migrates data from one HandlerMemory at a given address to another HandlerMemory at another
  /// address. Used by many things.
  pub fn transfer(origin: &HandlerMemory, orig_addr: i64, dest: &mut HandlerMemory, dest_addr: i64) {
    let ((a, b), hm_opt) = origin.addr_to_idxs(orig_addr);
    let orig = if hm_opt.is_none() { origin } else { hm_opt.unwrap() };
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
      let mut orig_arr_copies: Vec<Vec<(usize, i64)>> = vec![orig.read_fractal(orig_addr).block.clone()];
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
    let ((a, b), _) = self.addr_to_idxs(orig_addr);
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
      let mut orig_arr_copies: Vec<Vec<(usize, i64)>> = vec![self.read_fractal(orig_addr).block.clone()];
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
 
  /// Returns a new HandlerMemory with a read-only reference to the self HandlerMemory as the parent
  pub fn fork(parent: Arc<HandlerMemory>) -> HandlerMemory {
    let s = parent.mems.len();
    let mut hm = HandlerMemory::new(None, 1);
    hm.parent = Some(parent);
    hm.mems.resize(s + 1, Vec::new());
    hm.mem_addr = s;
    hm.args_addr = s;
    return hm;
  }

  /// Joins two HandlerMemory structs back together. Assumes that the passed in handler memory was
  /// generated by a fork call. This process moves over the records created in the forked HandlerMemory
  /// into the original and then "stitches up" the virtual memory pointers for anything pointing at
  /// newly-created data. This mechanism is faster but will keep around unreachable memory for longer.
  /// Whether or not this is the right trade-off will have to be seen by real-world usage.
  pub fn join(self: &mut HandlerMemory, hm: &mut HandlerMemory) {
    let s = hm.mem_addr; // The initial block that will be transferred (plus all following blocks)
    let s2 = self.mems.len(); // The new address of the initial block
    let offset = s2 - s; // Assuming it was made by `fork` this should be positive or zero
    if hm.addr.1.len() > 0 {
      let (a, b) = hm.addr_to_idxs_opt(CLOSURE_ARG_MEM_START).unwrap(); // The only address that can "escape"
      // println!("a: {}, b: {}, s: {}, in_fork: {}, in_parent: {}", a, b, s, hm.is_idx_defined(a, b), self.is_idx_defined(a, b));
      // println!("{:?}", hm);
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
    /*if hm.addr.0.len() > self.addr.0.len() {
      self.addr.0.resize(hm.addr.0.len(), (0, 0));
    }
    for i in 0..hm.addr.0.len() {
      let (a, b) = hm.addr.0[i];
      let (c, d) = self.addr.0[i];
      if a != c || b != d {
        if a + offset >= s && a != std::usize::MAX {
          self.addr.0[i] = (a + offset, b);
        } else {
          self.addr.0[i] = (a, b);
        }
      } else if a == c {
        self.addr.0[i] = (a, b);
      }
    }*/
  }

  /// Takes a UTF-8 string and converts it to fractal memory that can be stored inside of a
  /// HandlerMemory. Alan stores strings as Pascal strings with a 64-bit length prefix. There is no
  /// computer on the planet that has 64-bits worth of RAM, so this should work for quite a while
  /// into the future. :)
  pub fn str_to_fractal(s: &str) -> FractalMemory {
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
    FractalMemory::new(s_mem)
  }

  /// Takes a fractal memory and treats it like a UTF-8 encoded Pascal string, and the converts it
  /// to something Rust can work with. This function *may* crash if the underlying data is not a
  /// UTF-8 encoded Pascal string.
  pub fn fractal_to_string(f: FractalMemory) -> String {
    //println!("{:?}", f);
    let s_len = f.block[0].1 as usize;
    let mut s_bytes: Vec<u8> = Vec::new();
    for i in 1..f.block.len() {
      let mut b = f.block[i].1.to_ne_bytes().to_vec();
      s_bytes.append(&mut b);
    }
    let s = str::from_utf8(&s_bytes[0..s_len]).unwrap();
    s.to_string()
  }

  // pub fn fractal_to_string_2(self: &HandlerMemory, f: FractalMemory) -> String {
  //   let is_ptr = f.block[0].1 as usize == std::usize::MAX;
  //   let ((a, b), _) = self.addr_to_idxs(orig_addr);
  //   let s_len = f.block[0].1 as usize;
  //   let mut s_bytes: Vec<u8> = Vec::new();
  //   for i in 1..f.block.len() {
  //     let mut b = f.block[i].1.to_ne_bytes().to_vec();
  //     s_bytes.append(&mut b);
  //   }
  //   let s = str::from_utf8(&s_bytes[0..s_len]).unwrap();
  //   s.to_string()
  // }
}
