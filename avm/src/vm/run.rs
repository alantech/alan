use std::convert::TryInto;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::sync::Arc;

use byteorder::ByteOrder;
use byteorder::LittleEndian;
use flate2::read::GzDecoder;
use once_cell::sync::OnceCell;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};

use crate::vm::event::{BuiltInEvents, EventEmit, HandlerFragment};
use crate::vm::http::{HttpConfig, HttpType};
use crate::vm::memory::HandlerMemory;
use crate::vm::program::{Program, PROGRAM};
use crate::vm::{VMError, VMResult};

pub static EVENT_TX: OnceCell<UnboundedSender<EventEmit>> = OnceCell::new();

pub struct VM {
  /// chan for the events queue
  event_tx: UnboundedSender<EventEmit>,
  event_rx: UnboundedReceiver<EventEmit>,
}

impl VM {
  pub fn new() -> VMResult<VM> {
    let (event_tx, event_rx) = unbounded_channel();
    // Hackery relying on VM being a singleton :( TODO: Refactor such that event_tx is accessible
    // outside of the opcodes and instruction scheduler for http and future IO sources
    EVENT_TX
      .set(event_tx.clone())
      .map_err(|_| VMError::AlreadyRunning)?;
    return Ok(VM { event_tx, event_rx });
  }

  pub fn add(self: &mut VM, event: EventEmit) -> VMResult<()> {
    self
      .event_tx
      .send(event)
      .map_err(|_| VMError::ShutDown)
  }

  fn sched_event(self: &mut VM, event: EventEmit) -> VMResult<()> {
    // skip NOP event always, but it's not an error to receive it.
    if event.id == i64::from(BuiltInEvents::NOP) {
      return Ok(());
    }

    // schedule 1st fragment of each handler of this event
    let handlers = Program::global()
      .event_handlers
      .get(&event.id)
      .ok_or(VMError::EventNotDefined(event.id))?;
    for (i, hand) in handlers.iter().enumerate() {
      // first fragment of this handler
      let frag = HandlerFragment::new(event.id, i);
      let payload = match event.payload.as_ref() {
        Some(upstream_mem) => Some(Arc::new(HandlerMemory::clone(upstream_mem))),
        None => None,
      };
      // memory frag representing the memory for each handler call
      let hand_mem = HandlerMemory::new(payload, hand.mem_req)?;
      frag.spawn(hand_mem);
    }
    Ok(())
  }

  // run the vm backed by an event loop
  pub async fn run(self: &mut VM) -> VMResult<()> {
    while let Some(event) = self.event_rx.recv().await {
      self.sched_event(event)?;
    }
    Ok(())
  }
}

pub async fn run_file(fp: &str, delete_after_load: bool) -> VMResult<()> {
  let filepath = Path::new(fp);
  let mut file = File::open(filepath).map_err(|_| VMError::FileNotFound(fp.to_string()))?;
  let metadata = file.metadata().map_err(|_| {
    VMError::InvalidFile(format!(
      "Unable to get file metadata for file at {}. Are you sure it's a file?",
      fp
    ))
  })?;
  let fsize = metadata
    .len()
    .try_into()
    .map_err(|_| VMError::InvalidFile(format!("{} is a very big file on a 32-bit system", fp)))?;
  // Test if it's gzip compressed
  // TODO: new_uninit is nightly-only right now, we can use it to do this and achieve gains:
  // let mut bytes = Box::new_uninit_slice(fsize);
  // file.read_exact(&mut bytes).or(...)?;
  let mut bytes = Vec::with_capacity(fsize);
  file.read_to_end(&mut bytes).map_err(VMError::IOError)?;
  let mut gz = GzDecoder::new(bytes.as_slice());
  if gz.header().is_some() {
    let mut uncompressed = Vec::with_capacity(fsize * 2);
    let _bytes_read = gz
      .read_to_end(&mut uncompressed)
      .map_err(VMError::IOError)?;
    bytes = uncompressed;
  }
  let bytecode = bytes
    .as_slice()
    .chunks(8)
    .map(LittleEndian::read_i64)
    .collect::<Vec<_>>();
  if delete_after_load {
    std::fs::remove_file(Path::new(fp)).map_err(VMError::IOError)?;
  }
  run(bytecode, HttpType::HTTP(HttpConfig { port: 8000 })).await
}

pub async fn run(bytecode: Vec<i64>, http_config: HttpType) -> VMResult<()> {
  let program = Program::load(bytecode, http_config);
  PROGRAM.set(program).map_err(|_| {
    VMError::Other(
      "A program is already loaded".to_string(),
    )
  })?;
  let mut vm = VM::new()?;
  const START: EventEmit = EventEmit {
    id: BuiltInEvents::START as i64,
    payload: None,
  };
  vm.add(START)?;
  vm.run().await
}
