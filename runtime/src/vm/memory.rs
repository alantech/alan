use std::collections::HashMap;

use crate::vm::program::Program;

/// Memory representation of a handler call
#[derive(Clone)]
pub struct HandlerMemory {
  /// Global memory reference
  gmem: &'static Vec<u8>,
  /// Memory of the handler for fixed size data types
  mem: Vec<u8>,
  /// Optional payload address. None when no payload, negative when payload in gmem, otherwise 0.
  payload_addr: Option<i64>,
  /// Fractal memory storage for variable-length data types to an instance of the same struct
  fractal_mem: HashMap<i64, Box<HandlerMemory>>,
  // Pointers to a nested fractal represented as a vector of (addr, size).
  // These are not quite registers since they are not used directly by most opcodes
  // registers_ish: HashMap<i64, Vec<(i64, i64)>>,
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
    if pls == 0 && curr_addr == 0 {
      // no payload, void event
      return None;
    }
    // the size of this array will be different for very handler so it will be resized later
    let mut mem = vec![];
    let mut fractal_mem = HashMap::new();
    let mut payload_addr = Some(0);
    if curr_addr < 0 {
      // payload in gmem
      payload_addr = Some(curr_addr);
    } else if pls < 0 {
      // payload is a variable-length data type
      let payload: HandlerMemory = *curr_hand_mem.fractal_mem.get(&curr_addr).unwrap().clone();
      fractal_mem.insert(0, Box::new(payload.clone()));
    } else {
      // payload is a fixed length data type
      mem = curr_hand_mem.read(curr_addr, pls as u8).to_vec();
    };
    return Some(HandlerMemory {
      mem,
      fractal_mem,
      payload_addr,
      gmem: curr_hand_mem.gmem,
    });
  }

  pub fn new(mem_req: i64) -> HandlerMemory {
    return HandlerMemory {
      payload_addr: None,
      gmem: &Program::global().gmem,
      mem: vec![0; mem_req as usize],
      fractal_mem: HashMap::new(),
    };
  }

  pub fn resize_mem_req(self: &mut HandlerMemory, mem_req: i64) {
    let new_size = if mem_req < 0 { 0 } else { mem_req as usize };
    self.mem.resize(new_size, 0);
  }

  pub fn read(self: &HandlerMemory, addr: i64, size: u8) -> &[u8] {
    let actual_addr = if addr == 0 && self.payload_addr.is_some() {
      self.payload_addr.unwrap()
    } else {
      addr
    };
    if actual_addr < 0 {
      let a = (0 - actual_addr - 1) as usize;
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
    let a = actual_addr as usize;
    let result = match size {
      0 => {
        // string as array u8
        let arr = self.fractal_mem.get(&actual_addr);
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
        // string as array u8
        let arr = HandlerMemory {
          payload_addr: None,
          mem: payload.to_vec(),
          fractal_mem: HashMap::new(),
          gmem: self.gmem,
        };
        self.fractal_mem.insert(addr, Box::new(arr));
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