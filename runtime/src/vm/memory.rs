use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashMap, HashSet};

use uuid::Uuid;

use crate::vm::event::EventHandler;

/// Default starting size for a VM's memory
pub const DEFAULT_MEMORY_STARTING_SIZE: usize = 64;
/// Determines what factor we grow memory by on a resize
pub const GROWTH_FACTOR: f64 = 1.5;

/// Memory for handlers
pub struct VMMemory {
  /// Reference to global memory to keep things simpler
  gmem: &'static Vec<u8>,
  /// Memory counter that tracks the upper bound of allocated mem
  mc: usize,
  /// Memory of the program
  mem: Vec<u8>,
  /// Memory storage for variable-memory data types like strings segmented by handler
  var_mems: HashMap<Uuid, HashMap<i64, Vec<u8>>>,
  /// map of a uuid of each handler call to relevant metadata: start mem offset, end mem offset,
  /// payload_addr, and variable length memory storage (strings, etc)
  handler_records: HashMap<Uuid, (usize, usize, Option<i64>, HashMap<i64, Vec<u8>>)>,
  /// Lazily tracks the allocated start offsets using a min binary heap
  /// This allows us peek at an eventually consistent smallest/oldest offset in O(1)
  min_offset_heap: BinaryHeap<Reverse<usize>>,
  /// Tracks start offsets for handlers that have been deallocated
  /// but are still in the heap
  stale_offsets: HashSet<usize>,
}

/// Memory for each handler fragment
pub struct MemoryFragment {
  /// global memory reference
  gmem: &'static Vec<u8>,
  /// a slice of the memory for the fragment to work with. Unfortunately for now it's a copy
  mem_cpy: Vec<u8>,
  /// the separately allocated payload data. being separately allocated is probably good long term
  /// but this shouldn't have been exposed to the rest of the codebase, it's an optimization detail
  /// try to see if this can be hidden in the future
  pub payload_addr: Option<i64>,
  /// the memory storage for variable data types (strings, etc)
  var_mem: HashMap<i64, Vec<u8>>,
}

impl MemoryFragment {
  pub fn new(
    gmem: &'static Vec<u8>,
    mem_cpy: Vec<u8>,
    payload_addr: Option<i64>,
    var_mem: HashMap<i64, Vec<u8>>,
  ) -> MemoryFragment {
    return MemoryFragment {
      gmem,
      mem_cpy,
      payload_addr,
      var_mem,
    }
  }

  pub fn read(self: &MemoryFragment, addr: i64, size: u8) -> &[u8] {
    if addr < 0 {
      let a = (0 - addr - 1) as usize;
      return match size {
        0 => &self.gmem[a..],
        1 => &self.gmem[a..a + 1],
        2 => &self.gmem[a..a + 2],
        4 => &self.gmem[a..a + 4],
        8 => &self.gmem[a..a + 8],
        _ => panic!("Impossible size selection on global memory!"),
      }
    }
    let a = addr as usize;
    return match size {
      0 => &self.var_mem.get(&(a as i64)).unwrap(),
      1 => &self.mem_cpy[a..a + 1],
      2 => &self.mem_cpy[a..a + 2],
      4 => &self.mem_cpy[a..a + 4],
      8 => &self.mem_cpy[a..a + 8],
      _ => panic!("Impossible size selection on local memory!"),
    }
  }

  pub fn write(self: &mut MemoryFragment, addr: i64, size: u8, payload: &[u8]) {
    if addr < 0 {
      panic!("You can't write to global memory!");
    }
    let a = addr as usize;
    match size {
      0 => { self.var_mem.insert(addr, payload.to_vec()); },
      1 => self.mem_cpy[a] = payload[0],
      2 => {
        self.mem_cpy[a] = payload[0];
        self.mem_cpy[a + 1] = payload[1];
      },
      4 => {
        self.mem_cpy[a] = payload[0];
        self.mem_cpy[a + 1] = payload[1];
        self.mem_cpy[a + 2] = payload[2];
        self.mem_cpy[a + 3] = payload[3];
      },
      8 => {
        self.mem_cpy[a] = payload[0];
        self.mem_cpy[a + 1] = payload[1];
        self.mem_cpy[a + 2] = payload[2];
        self.mem_cpy[a + 3] = payload[3];
        self.mem_cpy[a + 4] = payload[4];
        self.mem_cpy[a + 5] = payload[5];
        self.mem_cpy[a + 6] = payload[6];
        self.mem_cpy[a + 7] = payload[7];
      },
      _ => panic!("Unexpected write of strange byte size!"),
    }
  }
}


impl VMMemory {
  pub fn new(gmem: &'static Vec<u8>) -> VMMemory {
    return VMMemory {
      gmem: gmem,
      mc: 0,
      mem: vec![0; DEFAULT_MEMORY_STARTING_SIZE],
      var_mems: HashMap::new(),
      handler_records: HashMap::new(),
      min_offset_heap: BinaryHeap::new(),
      stale_offsets: HashSet::new(),
    };
  }

  fn derive_payload_addr(
    self: &mut VMMemory,
    uuid: &Uuid,
    handler: &EventHandler,
    payload: &Vec<u8>,
    gmem_addr: Option<i64>
  ) -> Option<i64> {
    // Make sure the handler's variable memory exists, or create it if not
    if !self.var_mems.contains_key(uuid) {
      self.var_mems.insert(*uuid, HashMap::new());
    }
    return if payload.len() > 0 {
      // Signal that this event actually takes a variable memory object and this should be
      // allocated accordingly
      if handler.mem_req < 0 {
        let mut var_mem = self.var_mems.get_mut(uuid).unwrap();
        var_mem.insert(0, payload.to_vec());
      } else {
        // allocate payload at beg of handler's memory
        self.mem.splice(self.mc..self.mc, payload.to_vec());
      }
      Some(0)
    } else if gmem_addr.is_some() {
      // provider gmem address which is negative
      gmem_addr
    } else {
      None
    };
  }

  /// Peeks in the min binary heap for the minimum offset. It will return 0 if the heap is empty
  /// or the result without removing it. Since the heap is lazily updated the value returned
  /// could be a stale offset, but everything from the start of mem to this
  /// offset is always guaranteed to be available.
  fn min_offset(self: &mut VMMemory) -> usize {
    return self.min_offset_heap.peek().unwrap_or(&Reverse(0)).0;
  }

  /// returns a new memory fragment and an optional payload address within it from the event emission
  pub fn alloc_handler(self: &mut VMMemory, handler: &EventHandler, uuid: Uuid, payload: &Vec<u8>, gmem_addr: Option<i64>) -> MemoryFragment {
    let payload_addr = self.derive_payload_addr(&uuid, handler, payload, gmem_addr);
    let mem_req = if handler.mem_req < 0 { 8 } else { handler.mem_req } as usize;
    // Allocate right behind smallest offset if possible
    let min_offset = self.min_offset();
    if min_offset > mem_req {
      let start = min_offset - mem_req;
      let end = min_offset;
      // update internal state
      self.min_offset_heap.push(Reverse(start));
      self.handler_records.insert(uuid, (start, end, payload_addr, HashMap::new()));
      // return empty vec if no payload address
      if payload_addr.is_none() {
        return MemoryFragment::new(
          self.gmem,
          vec![0; end - start],
          None,
          self.var_mems.get(&uuid).unwrap().clone(),
        );
      }
      return MemoryFragment::new(
        self.gmem,
        self.mem[start..end].to_vec(),
        payload_addr,
        self.var_mems.get(&uuid).unwrap().clone(),
      );
    }
    // Allocated at the end of memory so adjust memory counter
    let old_mc = self.mc;
    let new_mc = old_mc + mem_req;
    // resize mem if needed to allocate handler
    if new_mc > self.mem.len() {
      let new_len = (self.mem.len() as f64 * GROWTH_FACTOR) as usize;
      if new_len > new_mc {
        self.mem.resize(new_len, 0);
      } else {
        self.mem.resize(new_mc, 0);
      }
    }
    // update internal state
    self.mc = new_mc;
    self.min_offset_heap.push(Reverse(old_mc));
    self.handler_records.insert(uuid, (old_mc, new_mc, payload_addr, HashMap::new()));
    // return empty vec if no payload address
    if payload_addr.is_none() {
      return MemoryFragment::new(
        self.gmem,
        vec![0; mem_req],
        None,
        self.var_mems.get(&uuid).unwrap().clone(),
      );
    }
    return MemoryFragment::new(
      self.gmem,
      self.mem[old_mc..new_mc].to_vec(),
      payload_addr,
      self.var_mems.get(&uuid).unwrap().clone(), // TODO: Eliminate useless memory copying
    );
  }

  /// make a copy the handler's new memory
  pub fn update_handler(self: &mut VMMemory, uuid: Uuid, mem_frag: &MemoryFragment) {
    let new_mem = &mem_frag.mem_cpy;
    let (start, end, payload_addr, var_mem) = self.handler_records.get(&uuid).unwrap();
    // update mem
    assert_eq!(start + new_mem.len(), *end);
    self.mem.splice(start..end, new_mem.to_vec());
    self.var_mems.insert(uuid, mem_frag.var_mem.clone());
    self.handler_records.insert(uuid, (*start, *end, *payload_addr, mem_frag.var_mem.clone()));
  }

  pub fn dealloc_handler(self: &mut VMMemory, uuid: Uuid) {
    self.var_mems.remove(&uuid);
    let (start, end, _, _) = self.handler_records.remove(&uuid).unwrap();
    for i in start..end { self.mem[i] = 0 }
    let mut min_offset = self.min_offset();
    if start != min_offset {
      self.stale_offsets.insert(start);
      return;
    }
    // deallocating oldest handler so pop from heap
    self.min_offset_heap.pop();
    // rm as many stale offsets as possible
    min_offset = self.min_offset();
    while self.stale_offsets.contains(&min_offset) {
      self.min_offset_heap.pop();
      self.stale_offsets.remove(&min_offset);
      min_offset = self.min_offset();
    }
  }
}