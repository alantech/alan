use std::fs::File;
use std::io::Read;

use byteorder::{LittleEndian, ReadBytesExt};
use tokio::runtime;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel};
use uuid::Uuid;

use crate::vm::event::{BuiltInEvents, EventEmit, HandlerFragment};
use crate::vm::instruction::InstructionScheduler;
use crate::vm::memory::{MemoryFragment, VMMemory};
use crate::vm::program::{PROGRAM, Program};

pub struct VM {
  /// Bytecode program to run
  pgm: &'static Program,
  /// chan for the events queue
  event_tx: UnboundedSender<EventEmit>,
  event_rx: UnboundedReceiver<EventEmit>,
  /// chan for the done fragments queue of all handler calls
  /// includes uuid for handler call, fragment done and handler memory
  /// used by io and cpu fragments
  frag_rx: UnboundedReceiver<(Uuid, HandlerFragment, MemoryFragment)>,
  /// memory manager of the program
  mem_man: VMMemory,
  /// Instruction scheduler for fragments that manages the cpu + io threadpools
  ins_sched: InstructionScheduler,
}

impl VM {
  pub fn new(pgm: &'static Program) -> VM {
    let (event_tx, event_rx) = unbounded_channel();
    let (frag_tx, frag_rx) = unbounded_channel();
    return VM {
      ins_sched: InstructionScheduler::new(pgm, event_tx.clone(), frag_tx),
      mem_man: VMMemory::new(&pgm.gmem),
      pgm,
      event_tx,
      event_rx,
      frag_rx,
    };
  }

  pub fn add(self: &mut VM, event: EventEmit) {
    self.event_tx.send(event);
  }

  async fn sched_fragment(self: &mut VM, frag_tup: (Uuid, HandlerFragment, MemoryFragment)) {
    let (call_uuid, mut frag, mut mem_frag) = frag_tup;
    self.mem_man.update_handler(call_uuid, &mem_frag);
    let next_frag = frag.get_next_fragment();
    if next_frag.is_none() {
      self.mem_man.dealloc_handler(call_uuid);
    } else {
      self.ins_sched.sched_frag(next_frag.unwrap(), call_uuid, mem_frag).await;
    }
  }

  async fn sched_event(self: &mut VM, event: EventEmit) {
    let payload = event.payload.unwrap_or(Vec::new());
    // schedule 1st fragment of each handler of this event
    let handlers = self.pgm.event_handlers.get(&event.id).unwrap();
    for (i, handler) in handlers.iter().enumerate() {
      // uuid for this specific handler call
      let call_uuid = Uuid::new_v4();
      let mem_frag = self.mem_man.alloc_handler(handler, call_uuid, &payload, event.gmem_addr);
      // first fragment of this handler
      let frag = HandlerFragment::new(self.pgm, event.id, i);
      self.ins_sched.sched_frag(frag, call_uuid, mem_frag).await;
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
    let start = EventEmit { id: i64::from(BuiltInEvents::START), payload: None, gmem_addr: None };
    vm.add(start);
    vm.run().await;
  })
}
