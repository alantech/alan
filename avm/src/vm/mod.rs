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
  AlreadyRunning,
  EventNotDefined(i64),
  FileNotFound(String), // path
  HandMemDanglingPtr,
  InvalidFile(String), // reason
  IOError(std::io::Error),
  IllegalAccess,
  InvalidNOP,
  InvalidString,
  MemoryNotOwned,
  OrphanMemory,
  ShutDown,
  Other(String),
  UnexpectedInstruction(InstrType),
}

impl Display for VMError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      VMError::AlreadyRunning => write!(f, "A VM instance was already detected to be running"),
      VMError::EventNotDefined(event_id) => {
        write!(f, "Event with event id {} is not defined", event_id)
      }
      VMError::FileNotFound(path) => write!(f, "File not found: {}", path),
      VMError::HandMemDanglingPtr => write!(f, "There is a dangling pointer to a HandlerMemory"),
      VMError::InvalidFile(reason) => write!(f, "File is invalid: {}", reason),
      VMError::IOError(err) => err.fmt(f),
      VMError::IllegalAccess => write!(f, "Illegal access"),
      VMError::InvalidNOP => write!(f, "A NOP operation was used in an illegal context"),
      VMError::InvalidString => write!(f, "Invalid string"),
      VMError::MemoryNotOwned => write!(
        f,
        "Attempting to write to memory not owned by the current handler"
      ),
      VMError::OrphanMemory => write!(
        f,
        "Memory referenced in parent, but no parent pointer defined"
      ),
      VMError::ShutDown => write!(f, "The AVM instance appears to be shut down"),
      VMError::Other(reason) => reason.fmt(f),
      VMError::UnexpectedInstruction(instr_ty) => {
        write!(f, "Expected another {} instruction", instr_ty)
      }
    }
  }
}

impl std::error::Error for VMError {}

#[derive(Debug)]
pub enum InstrType {
  CPU,
  IO,
  UnpredictableCPU,
}

impl Display for InstrType {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      InstrType::CPU => write!(f, "CPU"),
      InstrType::IO => write!(f, "IO"),
      InstrType::UnpredictableCPU => write!(f, "Unpredictable CPU"),
    }
  }
}
