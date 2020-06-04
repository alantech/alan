use std::collections::HashMap;

use crate::vm::event::{EventHandler, EventEmit};
use crate::vm::program::Program;

/// Memory representation of a handler call
#[derive(Clone)]
pub struct HandlerMemory {
  /// global memory reference
  gmem: &'static Vec<u8>,
  /// memory of the handler for fixed size data types
  mem: Vec<u8>,
  /// Fractal memory storage for variable-length data types to an instance of the same struct
  fractal_mem: HashMap<i64, Box<HandlerMemory>>,
  /// Pointers to a nested fractal represented as a vector of (addr, size).
  /// These are not quite registers since they are not used directly by most opcodes
  registers_ish: HashMap<i64, Vec<(i64, i64)>>,
}

impl HandlerMemory {
  /// allocate a payload (new HandlerMemory) for the given event id from the
  /// address within the HandlerMemory provided. called by emit to opcode
  pub fn alloc(
    event_id: &i64,
    curr_addr: &i64,
    curr_hand_mem: &HandlerMemory,
  ) -> Option<HandlerMemory> {
    let pls = Program::global().event_pls.get(event_id).unwrap().clone();
    let mut mem = vec![];
    let mut fractal_mem = HashMap::new();
    // no payload
    if pls == 0 { return None }
    else if pls < 0 {
      // payload is a variable-length data type
      let payload: HandlerMemory = *curr_hand_mem.fractal_mem.get(curr_addr).unwrap().clone();
      fractal_mem.insert(0, Box::new(payload.clone()));
    } else {
      // payload is a fixed length data type
      let a = curr_addr.clone() as usize;
      let b = a + pls.clone() as usize;
      mem = curr_hand_mem.mem[a..b].to_vec();
    };
    return Some(HandlerMemory {
      mem,
      fractal_mem,
      gmem: curr_hand_mem.gmem,
      registers_ish: HashMap::new(),
    });
  }

  pub fn new() -> HandlerMemory {
    return HandlerMemory {
      gmem: &Program::global().gmem,
      mem: Vec::new(),
      fractal_mem: HashMap::new(),
      registers_ish: HashMap::new(),
    };
  }

  pub fn read(self: &HandlerMemory, addr: i64, size: u8) -> &[u8] {
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
      0 => {
        // string as array u8
        let arr = self.fractal_mem.get(&addr);
        return if arr.is_none() { &[] } else { arr.unwrap().mem.as_slice() }
      },
      1 => &self.mem[a..a + 1],
      2 => &self.mem[a..a + 2],
      4 => &self.mem[a..a + 4],
      8 => &self.mem[a..a + 8],
      _ => panic!("Impossible size selection on local memory!"),
    }
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
          mem: payload.to_vec(),
          fractal_mem: HashMap::new(),
          gmem: self.gmem,
          registers_ish: HashMap::new(),
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