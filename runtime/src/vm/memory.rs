use std::collections::HashMap;

use crate::vm::event::{EventHandler, EventEmit};

/// Memory representation of a handler call
pub struct HandlerMemory {
  /// global memory reference
  gmem: &'static Vec<u8>,
  /// memory of the handler for fixed size data types
  mem: Vec<u8>,
  /// optional payload address
  payload_addr: Option<i64>,
  /// the memory storage for variable data types (strings, etc)
  var_mem: HashMap<i64, Vec<u8>>,
  // Fractal memory storage for variable-length arrays registered as a new uuid to an instance of the same struct
  // fractal_mem: HashMap<Uuid, Option<Box<MemoryFragment>>>,
}

impl HandlerMemory {
  pub fn alloc(
    gmem: &'static Vec<u8>,
    handler: &EventHandler,
    event: &EventEmit,
  ) -> HandlerMemory {
    let mem_len = if handler.mem_req < 0 { 0 } else { handler.mem_req as usize };
    let mut mem = vec![0; mem_len];
    let mut payload_addr = None;
    let mut var_mem = HashMap::new();
    if event.payload.is_some() {
      let payload = event.payload.to_owned().unwrap();
      // Signal that this event actually takes a variable memory object
      if handler.mem_req < 0 {
        var_mem.insert(0, payload);
      } else {
        // allocate payload at beg of handler's memory
        mem.splice(0..0, payload);
      }
      payload_addr = Some(0);
    };
    return HandlerMemory {
      gmem,
      mem,
      var_mem,
      payload_addr,
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
        let result = self.var_mem.get(&(a as i64));
        return if result.is_none() { &[] } else { result.unwrap() }
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
      0 => { self.var_mem.insert(addr, payload.to_vec()); },
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