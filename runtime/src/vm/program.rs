use std::collections::HashMap;
use std::str;

use crate::vm::event::{BuiltInEvents, EventHandler};
use crate::vm::instruction::Instruction;
use crate::vm::opcode::{ByteOpcode};

use once_cell::sync::OnceCell;

// Facilitates parsing the alan graph code program
struct BytecodeParser {
  // Program counter that tracks which byte is being executed
  pc: usize,
  // The bytecode of the program being run
  bytecode: Vec<i64>,
}

impl BytecodeParser {
  // Grabs the next 64 bits (8 bytes)
  fn next_64_bits(self: &mut BytecodeParser) -> i64 {
    let result = self.bytecode[self.pc];
    self.pc += 1;
    return result
  }
}

#[derive(Debug)]
enum GraphOpcode {
  /// event handler declaration
  /// "handler:" in ASCII OR 6861 6e64 6c65 723a in hex
  HANDLER,
  /// statement declaration within handler
  /// "lineno:" in ASCII OR 6c69 6e65 6e6f 3a20 in hex
  LINENO,
  /// custom event declaration
  /// "eventdc:" in ASCII OR 6576 656e 7464 633a in hex
  CUSTOMEVENT,
}

impl From<i64> for GraphOpcode {
  fn from(v: i64) -> Self {
    let handler_num: i64 = i64::from_le_bytes([b'h', b'a', b'n', b'd', b'l', b'e', b'r', b':']);
    let line_num: i64 = i64::from_le_bytes([b'l', b'i', b'n', b'e', b'n', b'o', b':', b' ']);
    let custom_num: i64 = i64::from_le_bytes([b'e', b'v', b'e', b'n', b't', b'd', b'd', b':']);
    // TODO: Figure out why `match` failed here
    if v == handler_num {
      return GraphOpcode::HANDLER;
    } else if v == line_num {
      return GraphOpcode::LINENO;
    } else if v == custom_num {
      return GraphOpcode::CUSTOMEVENT;
    } else {
      panic!(format!("Illegal graph opcode {}", v));
    }
  }
}

// Representation of the alan graph code program as static, global data
pub struct Program {
  // Event id to Map of handler id to handler obj
  pub(crate) event_handlers: HashMap<i64, Vec<EventHandler>>,
  // Event id to payload size
  pub(crate) event_declrs: HashMap<i64, i64>,
  // Memory of the program for global variables and string literals
  pub(crate) gmem: Vec<u8>,
}

pub static PROGRAM: OnceCell<Program> = OnceCell::new();

impl Program {
  pub fn global() -> &'static Program {
    PROGRAM.get().unwrap()
  }

  // instantiate handlers and define payload sizes for built-in events
  fn load_builtin(self: &mut Program) {
    // START
    let start = i64::from(BuiltInEvents::START);
    self.event_declrs.insert(start, 0);
    self.event_handlers.insert(start, Vec::new());
    // INSTALL
    let install = i64::from(BuiltInEvents::INSTALL);
    self.event_declrs.insert(install, 0);
    self.event_handlers.insert(install, Vec::new());
  }

  // Parses and safely initializes the alan graph code program as static, global data
  pub fn load(bytecode: Vec<i64>) -> Program {
    let mut parser = BytecodeParser {
      pc: 0,
      bytecode,
    };
    let mut program = Program {
      event_handlers: HashMap::new(),
      event_declrs: HashMap::new(),
      gmem: Vec::new()
    };
    program.load_builtin();
    // parse agc version
    let agcv = parser.next_64_bits();
    let bytes = agcv.to_le_bytes();
    let repr = str::from_utf8(&bytes).unwrap();
    // println!("using alan graph code version {}", repr);
    // parse size of global memory constants and string literals in bytes
    let gms = parser.next_64_bits();
    if gms > 0  && gms % 8 != 0 {
      panic!("Global memory is not divisible by 8");
    }
    for _ in 0..gms/8 {
      for byte in &parser.next_64_bits().to_le_bytes() {
        program.gmem.push(byte.clone());
      }
    }
    // instantiate null handler
    let mut cur_handler = EventHandler::new(0, 0);
    // parse rest of agc through opcodes
    while parser.bytecode.len() > parser.pc {
      match GraphOpcode::from(parser.next_64_bits()) {
        GraphOpcode::CUSTOMEVENT => {
          let id = parser.next_64_bits();
          let pls = parser.next_64_bits(); // number of bytes payload consumes
          program.event_declrs.insert(id, pls);
          program.event_handlers.insert(id, Vec::new());
        }
        GraphOpcode::HANDLER => {
          // save instructions for previously defined handler
          if cur_handler.len() > 0 {
            let handlers = program.event_handlers.get_mut(&cur_handler.event_id).unwrap();
            handlers.push(cur_handler);
          }
          let id = parser.next_64_bits();
          // error if event has not been defined
          if !program.event_declrs.contains_key(&id) || !program.event_handlers.contains_key(&id) {
            eprintln!("Handler for undefined event with id: {}", id);
          }
          let handler_mem = parser.next_64_bits();
          cur_handler = EventHandler::new(handler_mem, id);
        }
        GraphOpcode::LINENO => {
          let id = parser.next_64_bits();
          let num_deps = parser.next_64_bits();
          let mut dep_ids = Vec::new();
          for _ in 0..num_deps {
            dep_ids.push(parser.next_64_bits());
          }
          let opcode = <&ByteOpcode>::from(parser.next_64_bits());
          let mut args = Vec::new();
          args.push(parser.next_64_bits());
          args.push(parser.next_64_bits());
          args.push(parser.next_64_bits());
          let ins = Instruction { id, opcode, args, dep_ids };
          cur_handler.add_instruction(ins);
        }
      }
    }
    // check in last handler
    let handlers = program.event_handlers.get_mut(&cur_handler.event_id).unwrap();
    handlers.push(cur_handler);
    return program;
  }
}

