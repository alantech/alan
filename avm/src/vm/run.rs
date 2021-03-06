use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::sync::Arc;

use base64;
use byteorder::{LittleEndian, ReadBytesExt};
use flate2::read::GzDecoder;
use once_cell::sync::OnceCell;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};

use crate::vm::event::{BuiltInEvents, EventEmit, HandlerFragment};
use crate::vm::memory::HandlerMemory;
use crate::vm::program::{Program, PROGRAM};

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

  fn sched_event(self: &mut VM, event: EventEmit) {
    // skip NOP event always
    if event.id == i64::from(BuiltInEvents::NOP) {
      return;
    }

    // schedule 1st fragment of each handler of this event
    let handlers = Program::global().event_handlers.get(&event.id).unwrap();
    for (i, hand) in handlers.iter().enumerate() {
      // first fragment of this handler
      let frag = HandlerFragment::new(event.id, i);
      let payload = match event.payload.as_ref() {
        Some(upstream_mem) => Some(Arc::new(HandlerMemory::clone(upstream_mem))),
        None => None,
      };
      // memory frag representing the memory for each handler call
      let hand_mem = HandlerMemory::new(payload, hand.mem_req);
      frag.spawn(hand_mem);
    }
  }

  // run the vm backed by an event loop
  pub async fn run(self: &mut VM) {
    loop {
      let event = self.event_rx.recv().await;
      self.sched_event(event.unwrap());
    }
  }
}

pub async fn run_file(fp: &str, delete_after_load: bool) {
  let fptr = File::open(fp);
  if fptr.is_err() {
    eprintln!("File not found: {}", fp);
    std::process::exit(2);
  }
  // Test if it's gzip compressed
  let mut bytes = Vec::new();
  File::open(fp).unwrap().read_to_end(&mut bytes).unwrap();
  let gz = GzDecoder::new(bytes.as_slice());
  let mut bytecode;
  if gz.header().is_some() {
    let count = gz.bytes().count();
    bytecode = vec![0; count / 8];
    let mut gz = GzDecoder::new(bytes.as_slice());
    gz.read_i64_into::<LittleEndian>(&mut bytecode).unwrap();
  } else {
    let bytes = File::open(fp).unwrap().bytes().count();
    bytecode = vec![0; bytes / 8];
    let mut f = File::open(fp).unwrap();
    f.read_i64_into::<LittleEndian>(&mut bytecode).unwrap();
  }
  if delete_after_load {
    std::fs::remove_file(Path::new(fp)).unwrap();
  }
  run(bytecode, 8000).await;
}

pub async fn run_agz_b64(agz_b64: &str) {
  let bytes = base64::decode(agz_b64).unwrap();
  let agz = GzDecoder::new(bytes.as_slice());
  let count = agz.bytes().count();
  let mut bytecode = vec![0; count / 8];
  let mut gz = GzDecoder::new(bytes.as_slice());
  gz.read_i64_into::<LittleEndian>(&mut bytecode).unwrap();
  run(bytecode, 80).await;
}

pub async fn run(bytecode: Vec<i64>, http_port: u16) {
  let program = Program::load(bytecode, http_port);
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
  vm.run().await;
}
