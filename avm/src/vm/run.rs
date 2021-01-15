use std::fs::File;
use std::io::Read;
use std::path::Path;

use byteorder::{LittleEndian, ReadBytesExt};
use futures::future::join_all;
use once_cell::sync::OnceCell;
use tokio::runtime;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};

use crate::vm::event::{BuiltInEvents, EventEmit, HandlerFragment};
use crate::vm::memory::HandlerMemory;
use crate::vm::program::{Program, PROGRAM};
use crate::vm::telemetry;

pub static EVENT_TX: OnceCell<UnboundedSender<EventEmit>> = OnceCell::new();

pub struct VM {
  /// chan for the events queue
  event_tx: UnboundedSender<EventEmit>,
  event_rx: UnboundedReceiver<EventEmit>,
}

impl VM {
  pub fn new() -> VM {
    let (event_tx, event_rx) = unbounded_channel();
    // Hackery relying on VM being a singleton :( TODO: Refactor such that event_tx is accessible
    // outside of the opcodes and instruction scheduler for http and future IO sources
    EVENT_TX.set(event_tx.clone()).unwrap();
    return VM { event_tx, event_rx };
  }

  pub fn add(self: &mut VM, event: EventEmit) {
    let event_sent = self.event_tx.send(event);
    if event_sent.is_err() {
      eprintln!("Event transmission error");
      std::process::exit(1);
    }
  }

  async fn sched_event(self: &mut VM, event: EventEmit) {
    // schedule 1st fragment of each handler of this event
    let handlers = Program::global().event_handlers.get(&event.id).unwrap();
    let mut futures = vec![];
    for (i, hand) in handlers.iter().enumerate() {
      // first fragment of this handler
      let frag = HandlerFragment::new(event.id, i);
      // memory frag representing the memory for each handler call
      let hand_mem = HandlerMemory::new(event.payload.clone(), hand.mem_req);
      futures.push(frag.run(hand_mem));
    }
    join_all(futures).await;
  }

  // run the vm backed by an event loop
  pub async fn run(self: &mut VM) {
    loop {
      let event = self.event_rx.recv().await;
      self.sched_event(event.unwrap()).await;
    }
  }
}

pub fn exec(fp: &str, delete_after_load: bool) {
  let rt = runtime::Builder::new_multi_thread()
    .enable_time()
    .enable_io()
    .build()
    .unwrap();
  // Start the root task backed by a single thread
  rt.block_on(async {
    let fptr = File::open(fp);
    if fptr.is_err() {
      eprintln!("File not found: {}", fp);
      std::process::exit(2);
    }
    let bytes = File::open(fp).unwrap().bytes().count();
    let mut bytecode = vec![0; bytes / 8];
    let mut f = File::open(fp).unwrap();
    f.read_i64_into::<LittleEndian>(&mut bytecode).unwrap();
    if delete_after_load {
      std::fs::remove_file(Path::new(fp)).unwrap();
    }
    let program = Program::load(bytecode);
    let set_global = PROGRAM.set(program);
    if set_global.is_err() {
      eprintln!("Failed to load bytecode");
      std::process::exit(1);
    }
    let mut vm = VM::new();
    let start = EventEmit {
      id: i64::from(BuiltInEvents::START),
      payload: None,
    };
    vm.add(start);
    telemetry::log().await;
    vm.run().await;
  })
}
