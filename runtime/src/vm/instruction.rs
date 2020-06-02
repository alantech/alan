use futures::future::join_all;
use rayon::{ThreadPool, ThreadPoolBuilder};
use tokio::sync::mpsc::UnboundedSender;
use tokio::task;

use crate::vm::event::{EventEmit, HandlerFragment};
use crate::vm::opcode::{ByteOpcode, EmptyFuture};
use crate::vm::program::Program;
use crate::vm::memory::MemoryFragment;

pub struct Instruction {
  // only unique per fn/handler
  pub(crate) id: i64,
  pub(crate) opcode: &'static ByteOpcode,
  pub(crate) args: Vec<i64>,
  pub(crate) dep_ids: Vec<i64>,
}

pub struct InstructionScheduler {
  event_tx: UnboundedSender<EventEmit>,
  frag_tx: UnboundedSender<(HandlerFragment, MemoryFragment)>,
  cpu_pool: ThreadPool,
}

impl InstructionScheduler {
  pub fn new(event_tx: UnboundedSender<EventEmit>, frag_tx: UnboundedSender<(HandlerFragment, MemoryFragment)>) -> InstructionScheduler {
    let cpu_threads = num_cpus::get() - 1;
    let cpu_pool = ThreadPoolBuilder::new().num_threads(cpu_threads).build().unwrap();
    return InstructionScheduler {
      event_tx,
      frag_tx,
      cpu_pool,
    }
  }

  pub async fn sched_frag(self: &mut InstructionScheduler, mut frag: HandlerFragment, mut mem_frag: MemoryFragment) {
    let instructions = frag.get_instruction_fragment();
    // io-bound fragment
    if !instructions[0].opcode.pred_exec && instructions[0].opcode.async_func.is_some() {
      // Is there really no way to avoid cloning the reference of the chan txs for tokio tasks? :'(
      let frag_tx = self.frag_tx.clone();
      let futures: Vec<EmptyFuture> = instructions.iter().map(|ins| {
        let async_func = ins.opcode.async_func.unwrap();
        return async_func(&ins.args, &mut mem_frag, &mut frag);
      }).collect();
      task::spawn(async move {
        // Poll futures concurrently, but not in parallel, using a single thread.
        // This is akin to Promise.all in JavaScript Promises.
        join_all(futures).await;
        // Unbounded channels, async or not, are non-blocking. `Send` succeeds automatically.
        // https://github.com/tokio-rs/tokio/issues/2447
        frag_tx.send((frag, mem_frag));
      });
    } else {
      // cpu-bound fragment of predictable or unpredictable execution
      self.cpu_pool.scope(|s| {
        let event_tx = &self.event_tx;
        let frag_tx = &self.frag_tx;
        s.spawn(move |_| {
          instructions.iter().for_each( |i| {
            let func = i.opcode.func.unwrap();
            let event = func(&i.args, &mut mem_frag, &mut frag);
            if event.is_some() {
              event_tx.send(event.unwrap());
            }
          });
          // register fragment from handler call as done
          frag_tx.send((frag, mem_frag));
        });
      })
    }
  }
}
