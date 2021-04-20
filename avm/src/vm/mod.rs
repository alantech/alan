use std::fmt::Display;

pub mod event;
#[macro_use]
pub mod http;
pub mod instruction;
pub mod memory;
pub mod opcode;
pub mod program;
pub mod protos;
pub mod run;
pub mod telemetry;

pub type VMResult<T> = Result<T, VMError>;

#[derive(Debug)]
pub enum VMError {
  EventNotDefined(i64),
  FileNotFound(String), // path
  HandMemDanglingPtr,
  InvalidFile(String), // reason
  IOError(std::io::Error),
  IllegalAccess,
  InvalidState(InvalidState), // reason
  InvalidString,
  OrphanMemory,
  Other(String),
}

impl Display for VMError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      VMError::EventNotDefined(event_id) => {
        writeln!(f, "Event with event id {} is not defined", event_id)
      }
      VMError::FileNotFound(path) => writeln!(f, "File not found: {}", path),
      VMError::HandMemDanglingPtr => writeln!(f, "There is a dangling pointer to a HandlerMemory"),
      VMError::InvalidFile(reason) => writeln!(f, "File is invalid: {}", reason),
      VMError::IOError(err) => err.fmt(f),
      VMError::IllegalAccess => writeln!(f, "Illegal access"),
      VMError::InvalidState(state) => writeln!(f, "Invalid AVM state: {}", state),
      VMError::InvalidString => writeln!(f, "Invalid string"),
      VMError::OrphanMemory => writeln!(
        f,
        "Memory referenced in parent, but no parent pointer defined"
      ),
      VMError::Other(reason) => reason.fmt(f),
    }
  }
}

impl std::error::Error for VMError {}

#[derive(Debug)]
pub enum InvalidState {
  AlreadyRunning,
  HandMemDanglingPtr,
  IllegalAccess,
  InvalidNOP,
  MemoryNotOwned,
  OrphanMemory,
  ShutDown,
  Other(String),
  UnexpectedInstruction(InstrType),
}

impl Display for InvalidState {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      InvalidState::AlreadyRunning => write!(f, "another AVM instance is already running"),
      InvalidState::HandMemDanglingPtr => {
        write!(f, "there is a dangling pointer to a HandlerMemory")
      }
      InvalidState::IllegalAccess => write!(f, "illegal access"),
      InvalidState::InvalidNOP => write!(f, "a NOP operation was used where it's not permitted"),
      InvalidState::MemoryNotOwned => write!(
        f,
        "attempting to write to memory not owned by the current handler"
      ),
      InvalidState::OrphanMemory => write!(
        f,
        "memory referenced in parent, but no parent pointer defined"
      ),
      InvalidState::ShutDown => write!(f, "the AVM has been shut down"),
      InvalidState::Other(reason) => reason.fmt(f),
      InvalidState::UnexpectedInstruction(InstrType::CPU) => {
        write!(f, "expected another CPU instruction")
      }
      InvalidState::UnexpectedInstruction(InstrType::IO) => {
        write!(f, "expected another IO instruction")
      }
      InvalidState::UnexpectedInstruction(InstrType::UnpredictableCPU) => {
        write!(f, "expected another unpredictable CPU instruction")
      }
    }
  }
}

#[derive(Debug)]
pub enum InstrType {
  CPU,
  IO,
  UnpredictableCPU,
}
