use futures::future::join_all;
use rayon::{ThreadPool, ThreadPoolBuilder};
use tokio::sync::mpsc::UnboundedSender;
use tokio::task;

use crate::vm::event::{EventEmit, HandlerFragment};
use crate::vm::opcode::{ByteOpcode, EmptyFuture};
use crate::vm::program::Program;
use crate::vm::memory::HandlerMemory;

pub struct Instruction {
  // only unique per fn/handler
  pub(crate) id: i64,
  pub(crate) opcode: &'static ByteOpcode,
  pub(crate) args: Vec<i64>,
  pub(crate) dep_ids: Vec<i64>,
}

pub struct InstructionScheduler {
  event_tx: UnboundedSender<EventEmit>,
  frag_tx: UnboundedSender<(HandlerFragment, HandlerMemory)>,
  cpu_pool: ThreadPool,
}

impl InstructionScheduler {
  pub fn new(event_tx: UnboundedSender<EventEmit>, frag_tx: UnboundedSender<(HandlerFragment, HandlerMemory)>) -> InstructionScheduler {
    let cpu_threads = num_cpus::get() - 1;
    let cpu_pool = ThreadPoolBuilder::new().num_threads(cpu_threads).build().unwrap();
    return InstructionScheduler {
      event_tx,
      frag_tx,
      cpu_pool,
    }
  }

  pub fn process_next_frag(mut frag_tx: &UnboundedSender<(HandlerFragment, HandlerMemory)>, mut frag: HandlerFragment, mut hand_mem: HandlerMemory) {
    let next_frag = frag.get_next_fragment();
    if next_frag.is_some() {
      frag_tx.send((next_frag.unwrap(), hand_mem));
    } else {
      // https://abramov.io/rust-dropping-things-in-another-thread
      drop(hand_mem);
    }
  }

  pub async fn sched_frag(self: &mut InstructionScheduler, mut frag: HandlerFragment, mut hand_mem: HandlerMemory) {
    let instructions = frag.get_instruction_fragment();
    // io-bound fragment
    if !instructions[0].opcode.pred_exec && instructions[0].opcode.async_func.is_some() {
      // Is there really no way to avoid cloning the reference of the chan txs for tokio tasks? :'(
      let frag_tx = self.frag_tx.clone();
      let futures: Vec<EmptyFuture> = instructions.iter().map(|ins| {
        let async_func = ins.opcode.async_func.unwrap();
        return async_func(&ins.args, &mut hand_mem, &mut frag);
      }).collect();
      task::spawn(async move {
        // Poll futures concurrently, but not in parallel, using a single thread.
        // This is akin to Promise.all in JavaScript Promises.
        join_all(futures).await;
        InstructionScheduler::process_next_frag(&frag_tx, frag, hand_mem);
      });
    } else {
      // cpu-bound fragment of predictable or unpredictable execution
      self.cpu_pool.scope(|s| {
        let event_tx = &self.event_tx;
        let frag_tx = &self.frag_tx;
        s.spawn(move |_| {
          instructions.iter().for_each( |i| {
            let func = i.opcode.func.unwrap();
            let event = func(&i.args, &mut hand_mem, &mut frag);
            if event.is_some() {
              event_tx.send(event.unwrap());
            }
          });
          InstructionScheduler::process_next_frag(frag_tx, frag, hand_mem);
        });
      })
    }
  }
}
