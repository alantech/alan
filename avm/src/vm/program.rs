use std::collections::HashMap;

use once_cell::sync::OnceCell;

use crate::vm::event::{BuiltInEvents, EventHandler};
use crate::vm::instruction::Instruction;
use crate::vm::opcode::{ByteOpcode, opcode_id, OPCODES};

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
    return result;
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
    let closure_num: i64 = i64::from_le_bytes([b'c', b'l', b'o', b's', b'u', b'r', b'e', b':']);
    let line_num: i64 = i64::from_le_bytes([b'l', b'i', b'n', b'e', b'n', b'o', b':', b' ']);
    let custom_num: i64 = i64::from_le_bytes([b'e', b'v', b'e', b'n', b't', b'd', b'd', b':']);
    // TODO: Figure out why `match` failed here
    if v == handler_num || v == closure_num {
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

/// Representation of the alan graph code program as static, global data
#[derive(Debug)]
pub struct Program {
  /// Event id to Map of handler id to handler obj
  pub(crate) event_handlers: HashMap<i64, Vec<EventHandler>>,
  /// Event id to payload size which is the number of bytes if fixed length type,
  /// or -1 if it's a variable length type or 0 if the event is void
  pub(crate) event_pls: HashMap<i64, i64>,
  /// Memory of the program for global variables and string literals
  pub(crate) gmem: Vec<(usize, i64)>,
  /// The port the http server should use
  pub(crate) http_port: u16,
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
    self.event_pls.insert(start, 0);
    self.event_handlers.insert(start, Vec::new());
    // HTTPCONN
    let httpconn = i64::from(BuiltInEvents::HTTPCONN);
    self.event_pls.insert(httpconn, 0);
    self.event_handlers.insert(httpconn, Vec::new());
    // NOP
    // the compiler does not allow explicitly defining handlers for this event
    // and it's used internally by the compiler for closures in unused conditional branches
    let nop: i64 = i64::from(BuiltInEvents::NOP);
    self.event_pls.insert(nop, 0);
    self.event_handlers.insert(nop, Vec::with_capacity(0));
  }

  // Parses and safely initializes the alan graph code program as static, global data
  pub fn load(bytecode: Vec<i64>, http_port: u16) -> Program {
    let mut parser = BytecodeParser { pc: 0, bytecode };
    let mut program = Program {
      event_handlers: HashMap::new(),
      event_pls: HashMap::new(),
      gmem: Vec::new(),
      http_port,
    };
    program.load_builtin();
    // parse agc version
    let _agcv = parser.next_64_bits();
    // let bytes = agcv.to_le_bytes();
    // let repr = std::str::from_utf8(&bytes).unwrap();
    // println!("using alan graph code version {}", repr);
    // parse size of global memory constants and string literals in bytes
    let gms = parser.next_64_bits();
    if gms > 0 && gms % 8 != 0 {
      panic!("Global memory is not divisible by 8");
    }
    for _ in 0..gms / 8 {
      program.gmem.push((std::usize::MAX, parser.next_64_bits()));
    }
    // instantiate null handler
    let mut cur_handler = EventHandler::new(0, 0);
    // parse rest of agc through opcodes
    while parser.bytecode.len() > parser.pc {
      match GraphOpcode::from(parser.next_64_bits()) {
        GraphOpcode::CUSTOMEVENT => {
          let id = parser.next_64_bits();
          let pls = parser.next_64_bits(); // number of bytes payload consumes
          program.event_pls.insert(id, pls);
          program.event_handlers.insert(id, Vec::new());
        }
        GraphOpcode::HANDLER => {
          // save instructions for previously defined handler
          if cur_handler.len() > 0 {
            let handlers = program
              .event_handlers
              .get_mut(&cur_handler.event_id)
              .unwrap();
            handlers.push(cur_handler);
          }
          let id = parser.next_64_bits();
          // error if event has not been defined
          if !program.event_pls.contains_key(&id) || !program.event_handlers.contains_key(&id) {
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
          let ins = Instruction {
            id,
            opcode,
            args,
            dep_ids,
          };
          cur_handler.add_instruction(ins);
        }
      }
    }
    // check in last handler
    let handlers = program
      .event_handlers
      .get_mut(&cur_handler.event_id)
      .unwrap();
    handlers.push(cur_handler);
    // special logic to auto-listen on the http server if it is being used
    if program.event_handlers.get(&BuiltInEvents::HTTPCONN.into()).unwrap().len() > 0 {
      let start_handlers = program.event_handlers.get_mut(&BuiltInEvents::START.into()).unwrap();
      if start_handlers.len() == 0 {
        // Create a new handler that just executes `httplsn`
        let mut listen_handler = EventHandler::new(1, BuiltInEvents::START.into());
        listen_handler.add_instruction(Instruction {
          id: 0,
          opcode: OPCODES.get(&opcode_id("httplsn")).unwrap(),
          args: vec![0, 0, 0],
          dep_ids: vec![],
        });
        start_handlers.push(listen_handler);
      } else {
        // Append the listen opcode to the end of the first start handler found
        let start_handler = &mut start_handlers[0];
        let start_handler_fragment_idx = start_handler.fragments.len() - 1;
        let last_frag = &mut start_handler.fragments[start_handler_fragment_idx];
        let last_id = last_frag[last_frag.len() - 1].id;
        start_handler.add_instruction(Instruction {
          id: i64::MAX, // That shouldn't collide with anything
          opcode: OPCODES.get(&opcode_id("httplsn")).unwrap(),
          args: vec![0, 0, 0],
          dep_ids: vec![last_id], // To make sure this is the last instruction run (usually)
        });
      }
    }
    return program;
  }
}
