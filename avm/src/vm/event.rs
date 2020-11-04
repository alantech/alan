use std::sync::Arc;

//use futures::future::join_all;
use tokio::sync::RwLock;
use tokio::task;

use crate::vm::instruction::Instruction;
use crate::vm::memory::HandlerMemory;
use crate::vm::opcode::EmptyFuture;
use crate::vm::program::Program;
use crate::vm::run::{EVENT_TX};

#[derive(PartialEq, Eq, Hash)]
/// Special events in alan found in standard library modules, @std.
/// The IDs for built-in events are negative to avoid collision with positive, custom event IDs.
/// The first hexadecimal byte of the ID in an integer is between 80 and FF
/// The remaining 7 bytes can be used for ASCII-like values
pub enum BuiltInEvents {
  /// Alan application start
  /// '"start"' in ASCII or 2273 7461 7274 22(80)
  START,
  /// '__conn ' in ASCII or 5f5f 636f 6e6e 20(80)
  HTTPCONN,
}

impl From<BuiltInEvents> for i64 {
  fn from(ev: BuiltInEvents) -> Self {
    match ev {
      BuiltInEvents::START => -9213673853036498142,
      BuiltInEvents::HTTPCONN => -9214243417005793441,
    }
  }
}

/// Describes an event emission received by the event loop from the thread worker
pub struct EventEmit {
  /// event id
  pub(crate) id: i64,
  /// optional handler memory with payload. each handler will get its own to consume
  pub(crate) payload: Option<HandlerMemory>,
}

/// Describes the handler for an event
pub struct EventHandler {
  /// event id
  pub(crate) event_id: i64,
  /// number of bytes each handler call requires in memory, or -1 if it's a variable length type
  pub(crate) mem_req: i64,
  /// the indices of fragments that have unpredictable execution and could be moved around
  movable_capstones: Vec<usize>,
  /// topological order of the instructions split into fragments
  /// by unpredictable or partially predictable opcodes
  fragments: Vec<Vec<Instruction>>,
  /// total count of instructions within fragments
  ins_count: usize,
}

impl EventHandler {
  pub fn new(mem_req: i64, event_id: i64) -> EventHandler {
    return EventHandler {
      fragments: Vec::new(),
      movable_capstones: Vec::new(),
      ins_count: 0,
      mem_req,
      event_id,
    };
  }

  pub fn add_instruction(self: &mut EventHandler, ins: Instruction) {
    self.ins_count += 1;
    if ins.opcode.func.is_some() {
      let mut frag = self.fragments.pop().unwrap_or(Vec::new());
      if frag.len() > 0 && !frag.get(frag.len() - 1).unwrap().opcode.pred_exec {
        // if last instruction in the last fragment is a (io or cpu) capstone start a new fragment
        self.fragments.push(frag);
        self.fragments.push(vec![ins]);
      } else {
        // add to last fragment
        frag.push(ins);
        self.fragments.push(frag);
      }
    } else {
      // TODO: Restore this logic. For now just turn it into a new fragment by itself
      /*
      // non-predictable io opcode is a "movable capstone" in execution
      let cur_max_dep = ins.dep_ids.iter().max().unwrap_or(&-1);
      // merge this capstone with an existing one if possible
      for frag_idx in &self.movable_capstones {
        let fragment = self.fragments.get_mut(*frag_idx).unwrap();
        let prev_max_dep = fragment.iter().map(|i| i.dep_ids.iter().max().unwrap_or(&-1)).max().unwrap();
        let prev_min_id = &fragment.iter().map(|i| i.id).min().unwrap();
        // merge curr in prev if *everything* that cur needs has ran by prev.
        // since poset is ranked we can check the max dep id of curr is:
        // less than the min id in the prev capstone
        // less than or equal to the max dep id from prev capstone
        if prev_min_id > cur_max_dep && prev_max_dep >= cur_max_dep {
          fragment.push(ins);
          return;
        }
      }
      // this is the first capstone or it cannot be merged
      // mark it as a new capstone
      self.movable_capstones.push(self.fragments.len());
      self.fragments.push(vec![ins]);
      */
      self.fragments.push(vec![ins]);
    }
  }

  pub fn len(self: &EventHandler) -> usize {
    return self.ins_count;
  }

  pub fn last_frag_idx(self: &EventHandler) -> usize {
    return self.fragments.len() - 1;
  }

  pub fn get_fragment(self: &EventHandler, idx: usize) -> &Vec<Instruction> {
    return self.fragments.get(idx).unwrap();
  }

}

/// Identifies an exact fragment of an event handler
#[derive(Clone)]
struct HandlerFragmentID {
  event_id: i64,
  handler_idx: usize,
  fragment_idx: Option<usize>,
}

/// Identifies the fragment of an event handler
#[derive(Clone)]
pub struct HandlerFragment {
  /// handler stack for other handlers sequentially running within itself.
  /// Required IDs to identify the event handler placed into a Vec
  handlers: Vec<HandlerFragmentID>,
}

impl HandlerFragmentID {
  /// increments or initializes fragment idx to 0 if it does not exist
  fn incr_frag_idx(self: &mut HandlerFragmentID) {
    if self.fragment_idx.is_none() {
      return self.fragment_idx = Some(0);
    }
    self.fragment_idx = Some(self.fragment_idx.unwrap() + 1);
  }
}

impl HandlerFragment {
  pub fn new(event_id: i64, handler_idx: usize) -> HandlerFragment {
    return HandlerFragment {
      handlers: vec!(HandlerFragmentID {
        event_id,
        handler_idx,
        fragment_idx: Some(0),
      }),
    }
  }

  pub fn get_instruction_fragment(self: &mut HandlerFragment) -> &'static Vec<Instruction> {
    let hand_id = self.handlers.get_mut(0).unwrap();
    let handlers = Program::global().event_handlers.get(&hand_id.event_id).unwrap();
    let handler: &EventHandler = handlers.get(hand_id.handler_idx).unwrap();
    return handler.get_fragment(hand_id.fragment_idx.unwrap());
  }

  pub fn get_next_fragment(mut self) -> Option<HandlerFragment> {
    let hand_id = self.handlers.get_mut(0).unwrap();
    let handlers = Program::global().event_handlers.get(&hand_id.event_id).unwrap();
    let handler: &EventHandler = handlers.get(hand_id.handler_idx).unwrap();
    let last_frag_idx = handler.last_frag_idx();
    return if hand_id.fragment_idx.is_some() && last_frag_idx <= hand_id.fragment_idx.unwrap() {
      self.handlers.remove(0);
      if self.handlers.len() == 0 {
        None
      } else {
        Some(self)
      }
    } else {
      hand_id.incr_frag_idx();
      Some(self)
    }
  }

  /// Runs the specified handler in Tokio tasks. Tokio tasks are allocated to a threadpool bound by
  /// the number of CPU cores on the machine it is executing on. Actual IO work gets scheduled into
  /// an IO threadpool (unbounded, according to Tokio) while CPU work uses the special
  /// `block_in_place` function to indicate to Tokio to not push new Tokio tasks to this particular
  /// thread at this time. Event-level parallelism and Array-level parallelism are unified into
  /// this same threadpool as tasks which should minimize contentions at the OS level on work and
  /// help throughput (in theory). This could be a problem for super-IO-bound applications with a
  /// very high volume of small IO requests, but as most IO operations are several orders of
  /// magnitude slower than CPU operations, this is considered to be a very small minority of
  /// workloads and is ignored for now.
  pub async fn run(mut self: HandlerFragment, mut hand_mem: HandlerMemory) -> HandlerMemory {
    task::spawn(async move {
      let mut instructions = self.get_instruction_fragment();
      loop {
        // io-bound fragment
        if !instructions[0].opcode.pred_exec && instructions[0].opcode.async_func.is_some() {
          // Is there really no way to avoid cloning the reference of the chan txs for tokio tasks? :'(
          let mem = Arc::new(RwLock::new(hand_mem));
          let futures: Vec<EmptyFuture> = instructions.iter().map(|ins| {
            let async_func = ins.opcode.async_func.unwrap();
            return async_func(ins.args.clone(), mem.clone());
          }).collect();
          hand_mem = task::spawn(async move {
            //join_all(futures).await;
            // Temporarily disable io parallelism until io opcode dependencies are declared
            // correctly by the compiler
            for future in futures {
              future.await;
            }
            let deref_res = Arc::try_unwrap(mem);
            if deref_res.is_err() {
              panic!("Arc for handler memory passed to io opcodes has more than one strong reference.");
            };
            deref_res.ok().unwrap().into_inner()
          }).await.unwrap();
        } else {
          // cpu-bound fragment of predictable or unpredictable execution
          let self_and_hand_mem = task::block_in_place(move || {
            instructions.iter().for_each( |i| {
              let func = i.opcode.func.unwrap();
              let event = func(i.args.as_slice(), &mut hand_mem);
              if event.is_some() {
                let event_tx = EVENT_TX.get().unwrap();
                let event_sent = event_tx.send(event.unwrap());
                if event_sent.is_err() {
                  eprintln!("Event transmission error");
                  std::process::exit(2);
                }
              }
            });
            (self, hand_mem)
          });
          self = self_and_hand_mem.0;
          hand_mem = self_and_hand_mem.1;
        }
        let next_frag = self.get_next_fragment();
        if next_frag.is_some() {
          self = next_frag.unwrap();
          instructions = self.get_instruction_fragment();
        } else {
          break;
        }
      }
      hand_mem
    }).await.unwrap()
  }
}

#[cfg(test)]
mod tests {
  use crate::vm::opcode::{opcode_id, OPCODES};

  use super::*;

  fn get_io_ins(id: i64, dep_ids: Vec<i64>) -> Instruction {
    return Instruction {
      id,
      opcode: &OPCODES.get(&opcode_id("waitop")).unwrap(),
      args: vec![],
      dep_ids
    };
  }

  fn get_cpu_ins(id: i64, dep_ids: Vec<i64>) -> Instruction {
    return Instruction {
      id,
      opcode: &OPCODES.get(&opcode_id("addi64")).unwrap(),
      args: vec![],
      dep_ids
    };
  }

  fn get_cond_ins(id: i64, dep_ids: Vec<i64>) -> Instruction {
    return Instruction {
      id,
      opcode: &OPCODES.get(&opcode_id("condfn")).unwrap(),
      args: vec![],
      dep_ids
    };
  }

  // multiple io operations with no dependencies forms a single fragment
  /*#[test]
  fn test_frag_grouping_1() {
    let mut hand = EventHandler::new(123, 123);
    hand.add_instruction(get_io_ins(0, vec![]));
    hand.add_instruction(get_io_ins(1, vec![]));
    hand.add_instruction(get_io_ins(2, vec![]));
    hand.add_instruction(get_io_ins(3, vec![]));
    assert_eq!(hand.last_frag_idx(), 0);
  }*/

  // chained io operations forms a fragment per io operation
  #[test]
  fn test_frag_grouping_2() {
    let mut hand = EventHandler::new(123, 123);
    hand.add_instruction(get_io_ins(0, vec![]));
    hand.add_instruction(get_io_ins(1, vec![0]));
    hand.add_instruction(get_io_ins(2, vec![1]));
    hand.add_instruction(get_io_ins(3, vec![2]));
    assert_eq!(hand.last_frag_idx(), 3);
  }

  // multiple io operations and one cpu operation in between
  // with no dependencies form 2 fragments
  /*#[test]
  fn test_frag_grouping_3() {
    let mut hand = EventHandler::new(123, 123);
    hand.add_instruction(get_io_ins(0, vec![]));
    hand.add_instruction(get_io_ins(1, vec![]));
    hand.add_instruction(get_cpu_ins(2, vec![]));
    hand.add_instruction(get_io_ins(3, vec![]));
    assert_eq!(hand.last_frag_idx(), 1);
    assert_eq!(hand.get_fragment(0).len(), 3);
    assert_eq!(hand.get_fragment(1).len(), 1);
  }*/

  // independent io operations, then independent cpu operation
  // and then io operation dependent on cpu operation forms 3 fragments
  /*#[test]
  fn test_frag_grouping_4() {
    let mut hand = EventHandler::new(123, 123);
    hand.add_instruction(get_io_ins(0, vec![]));
    hand.add_instruction(get_io_ins(1, vec![]));
    hand.add_instruction(get_cpu_ins(2, vec![]));
    hand.add_instruction(get_io_ins(3, vec![2]));
    assert_eq!(hand.last_frag_idx(), 2);
    assert_eq!(hand.get_fragment(0).len(), 2);
    assert_eq!(hand.get_fragment(1).len(), 1);
    assert_eq!(hand.get_fragment(2).len(), 1);
  }*/

  // independent io operations, then independent cpu operation
  // and then io operation dependent on io operations forms 3 fragments
  /*#[test]
  fn test_frag_grouping_5() {
    let mut hand = EventHandler::new(123, 123);
    hand.add_instruction(get_io_ins(0, vec![]));
    hand.add_instruction(get_io_ins(1, vec![]));
    hand.add_instruction(get_cpu_ins(2, vec![]));
    hand.add_instruction(get_io_ins(3, vec![1]));
    assert_eq!(hand.last_frag_idx(), 2);
    assert_eq!(hand.get_fragment(0).len(), 2);
    assert_eq!(hand.get_fragment(1).len(), 1);
    assert_eq!(hand.get_fragment(2).len(), 1);
  }*/

  // chained cpu operations form one fragment
  #[test]
  fn test_frag_grouping_6() {
    let mut hand = EventHandler::new(123, 123);
    hand.add_instruction(get_cpu_ins(0, vec![]));
    hand.add_instruction(get_cpu_ins(1, vec![0]));
    hand.add_instruction(get_cpu_ins(2, vec![1]));
    hand.add_instruction(get_cpu_ins(3, vec![2]));
    assert_eq!(hand.last_frag_idx(), 0);
  }

  // independent: io operation, then independent cpu operation
  // and then independent io operation then ind cpu operation then
  // dep io operation on first cpu operation forms 3 fragments
  /*#[test]
  fn test_frag_grouping_7() {
    let mut hand = EventHandler::new(123, 123);
    hand.add_instruction(get_io_ins(0, vec![]));
    hand.add_instruction(get_io_ins(1, vec![]));
    hand.add_instruction(get_cpu_ins(2, vec![]));
    hand.add_instruction(get_io_ins(3, vec![]));
    hand.add_instruction(get_cpu_ins(4, vec![]));
    hand.add_instruction(get_io_ins(5, vec![2]));
    assert_eq!(hand.last_frag_idx(), 2);
    assert_eq!(hand.get_fragment(0).len(), 3);
    assert_eq!(hand.get_fragment(1).len(), 2);
    assert_eq!(hand.get_fragment(2).len(), 1);
  }

  // independent: io operation, then independent cpu operation
  // and then dep io operation then ind cpu operation then
  // dep io operation on prev io operation forms 4 fragments
  #[test]
  fn test_frag_grouping_8() {
    let mut hand = EventHandler::new(123, 123);
    hand.add_instruction(get_io_ins(0, vec![]));
    hand.add_instruction(get_io_ins(1, vec![]));
    hand.add_instruction(get_cpu_ins(2, vec![]));
    hand.add_instruction(get_io_ins(3, vec![0]));
    hand.add_instruction(get_cpu_ins(4, vec![]));
    hand.add_instruction(get_io_ins(5, vec![]));
    assert_eq!(hand.last_frag_idx(), 3);
    assert_eq!(hand.get_fragment(0).len(), 3);
    assert_eq!(hand.get_fragment(1).len(), 1);
    assert_eq!(hand.get_fragment(2).len(), 1);
    assert_eq!(hand.get_fragment(3).len(), 1);
  }

  // condfn is an unmovable capstone for cpu operations that come *after* it
  // even when no deps
  #[test]
  fn test_frag_grouping_9() {
    let mut hand = EventHandler::new(123, 123);
    hand.add_instruction(get_cpu_ins(0, vec![]));
    hand.add_instruction(get_cpu_ins(1, vec![]));
    hand.add_instruction(get_cond_ins(2, vec![]));
    hand.add_instruction(get_cpu_ins(3, vec![]));
    assert_eq!(hand.movable_capstones.len(), 0);
    assert_eq!(hand.last_frag_idx(), 1);
  }

  // condfn is an unmovable capstone among io operations even when no deps
  #[test]
  fn test_frag_grouping_10() {
    let mut hand = EventHandler::new(123, 123);
    hand.add_instruction(get_io_ins(0, vec![]));
    hand.add_instruction(get_cond_ins(1, vec![]));
    hand.add_instruction(get_io_ins(2, vec![]));
    assert_eq!(hand.movable_capstones.len(), 1);
    assert_eq!(hand.last_frag_idx(), 1);
  }*/

  // multiple condfns each run in their own fragment
  #[test]
  fn test_frag_grouping_11() {
    let mut hand = EventHandler::new(123, 123);
    hand.add_instruction(get_cond_ins(0, vec![]));
    hand.add_instruction(get_cond_ins(1, vec![]));
    hand.add_instruction(get_cond_ins(2, vec![]));
    assert_eq!(hand.movable_capstones.len(), 0);
    assert_eq!(hand.last_frag_idx(), 2);
  }
}
