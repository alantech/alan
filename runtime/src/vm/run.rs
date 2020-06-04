use std::fs::File;
use std::io::Read;

use byteorder::{LittleEndian, ReadBytesExt};
use tokio::runtime;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel};

use crate::vm::event::{BuiltInEvents, EventEmit, HandlerFragment};
use crate::vm::instruction::InstructionScheduler;
use crate::vm::memory::HandlerMemory;
use crate::vm::program::{PROGRAM, Program};

pub struct VM {
  /// Bytecode program to run
  pgm: &'static Program,
  /// chan for the events queue
  event_tx: UnboundedSender<EventEmit>,
  event_rx: UnboundedReceiver<EventEmit>,
  /// chan for the done fragments queue of all handler calls
  /// used by io and cpu fragments
  frag_rx: UnboundedReceiver<(HandlerFragment, HandlerMemory)>,
  /// Instruction scheduler for fragments that manages the cpu + io threadpools
  ins_sched: InstructionScheduler,
}

impl VM {
  pub fn new(pgm: &'static Program) -> VM {
    let (event_tx, event_rx) = unbounded_channel();
    let (frag_tx, frag_rx) = unbounded_channel();
    return VM {
      ins_sched: InstructionScheduler::new(event_tx.clone(), frag_tx),
      pgm,
      event_tx,
      event_rx,
      frag_rx,
    };
  }

  pub fn add(self: &mut VM, event: EventEmit) {
    self.event_tx.send(event);
  }

  async fn sched_fragment(self: &mut VM, frag_tup: (HandlerFragment, HandlerMemory)) {
    let (frag, hand_mem) = frag_tup;
    self.ins_sched.sched_frag(frag, hand_mem).await;
  }

  async fn sched_event(self: &mut VM, event: EventEmit) {
    // schedule 1st fragment of each handler of this event
    let handlers = self.pgm.event_handlers.get(&event.id).unwrap();
    for (i, hand) in handlers.iter().enumerate() {
      // first fragment of this handler
      let frag = HandlerFragment::new(self.pgm, event.id, i);
      // memory frag representing the memory for each handler call
      let hand_mem = if event.payload.is_none() {
        HandlerMemory::new(hand.mem_req)
      } else {
        let mut res = event.payload.clone().unwrap();
        res.resize_mem_req(hand.mem_req);
        res
      };
      // TODO join on all these futures
      self.ins_sched.sched_frag(frag, hand_mem).await;
    }
  }

  // run the vm backed by an event loop
  pub async fn run(self: &mut VM) {
    loop {
      // Wait on fragments and events queue concurrently, but not in parallel, using a single thread.
      // Returns when the first branch and executes its handler while cancelling the other.
      // select! randomly picks a branch to provide some level of fairness within the loop.
      tokio::select! {
        event = self.event_rx.recv() => {
          self.sched_event(event.unwrap()).await;
        }
        frag = self.frag_rx.recv() => {
          self.sched_fragment(frag.unwrap()).await;
        }
      }
    }
  }
}

pub fn exec(fp: &str) {
  let mut rt = runtime::Builder::new()
    .basic_scheduler()
    .enable_time()
    .build()
    .unwrap();
  // Start the root task backed by a single thread
  rt.block_on(async {
    let bytes = File::open(fp).unwrap().bytes().count();
    let mut bytecode = vec![0;bytes/8];
    let mut f = File::open(fp).unwrap();
    f.read_i64_into::<LittleEndian>(&mut bytecode).unwrap();
    let program = Program::load(bytecode);
    PROGRAM.set(program);
    let mut vm = VM::new(Program::global());
    let start = EventEmit { id: i64::from(BuiltInEvents::START), payload: None };
    vm.add(start);
    vm.run().await;
  })
}