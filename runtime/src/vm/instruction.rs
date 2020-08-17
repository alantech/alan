use std::sync::Arc;

use futures::future::join_all;
use tokio::sync::{RwLock, mpsc::UnboundedSender};
use tokio::task;

use crate::vm::event::{EventEmit, HandlerFragment};
use crate::vm::opcode::{ByteOpcode, EmptyFuture};
use crate::vm::memory::HandlerMemory;

pub struct Instruction {
  // only unique per fn/handler
  pub(crate) id: i64,
  pub(crate) opcode: &'static ByteOpcode,
  pub(crate) args: Vec<i64>,
  pub(crate) dep_ids: Vec<i64>,
}

pub struct InstructionScheduler {
  pub event_tx: UnboundedSender<EventEmit>,
  frag_tx: UnboundedSender<(HandlerFragment, HandlerMemory)>,
}

impl InstructionScheduler {
  pub fn new(event_tx: UnboundedSender<EventEmit>, frag_tx: UnboundedSender<(HandlerFragment, HandlerMemory)>) -> InstructionScheduler {
    return InstructionScheduler {
      event_tx,
      frag_tx,
    }
  }

  fn process_next_frag(frag_tx: &UnboundedSender<(HandlerFragment, HandlerMemory)>, frag: HandlerFragment, hand_mem: HandlerMemory) {
    let next_frag = frag.get_next_fragment();
    if next_frag.is_some() {
      let frag_sent = frag_tx.send((next_frag.unwrap(), hand_mem));
      if frag_sent.is_err() {
        eprintln!("Event transmission error");
        std::process::exit(3);
      }
    } else {
      // This method is being called from a tokio task or a thread within the rayon thread pool
      // https://abramov.io/rust-dropping-things-in-another-thread
      drop(hand_mem);
    }
  }

  pub async fn sched_frag(self: &InstructionScheduler, mut frag: HandlerFragment, mut hand_mem: HandlerMemory) {
    let instructions = frag.get_instruction_fragment();
    // io-bound fragment
    if !instructions[0].opcode.pred_exec && instructions[0].opcode.async_func.is_some() {
      // Is there really no way to avoid cloning the reference of the chan txs for tokio tasks? :'(
      let frag_tx = self.frag_tx.clone();
      let mem = Arc::new(RwLock::new(hand_mem));
      let futures: Vec<EmptyFuture> = instructions.iter().map(|ins| {
        let async_func = ins.opcode.async_func.unwrap();
        return async_func(ins.args.clone(), mem.clone());
      }).collect();
      task::spawn(async move {
        join_all(futures).await;
        let deref_res = std::sync::Arc::try_unwrap(mem);
        if deref_res.is_err() {
          panic!("Arc for handler memory passed to io opcodes has more than one strong reference.");
        };
        let hand_mem = deref_res.ok().unwrap().into_inner();
        InstructionScheduler::process_next_frag(&frag_tx, frag, hand_mem);
      });
    } else {
      // cpu-bound fragment of predictable or unpredictable execution
      rayon::scope(|s| {
        let event_tx = &self.event_tx;
        let frag_tx = &self.frag_tx;
        s.spawn(move |_| {
          instructions.iter().for_each( |i| {
            let func = i.opcode.func.unwrap();
            let event = func(&i.args, &mut hand_mem, &mut frag, self);
            if event.is_some() {
              let event_sent = event_tx.send(event.unwrap());
              if event_sent.is_err() {
                eprintln!("Event transmission error");
                std::process::exit(2);
              }
            }
          });
          InstructionScheduler::process_next_frag(frag_tx, frag, hand_mem);
        });
      })
    }
  }
}
