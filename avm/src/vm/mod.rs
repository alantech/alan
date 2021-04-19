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
  FileNotFound(String), // path
  InvalidFile(String),  // reason
  IOError(std::io::Error),
  InvalidState(InvalidState), // reason
  EventNotDefined(i64),
}

impl Display for VMError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      VMError::FileNotFound(path) => write!(f, "file not found: {}", path),
      VMError::InvalidFile(reason) => write!(f, "file is invalid: {}", reason),
      VMError::IOError(err) => err.fmt(f),
      VMError::InvalidState(InvalidState::AlreadyRunning) => write!(f, ""),
      VMError::InvalidState(InvalidState::UnexpectedInstruction(expected)) => write!(f, ""),
      VMError::InvalidState(InvalidState::HandMemDanglingPtr) => write!(f, ""),
      VMError::InvalidState(InvalidState::ShutDown) => write!(f, ""),
      VMError::InvalidState(InvalidState::Other(reason)) => write!(f, ""),
      VMError::EventNotDefined(_) => write!(f, ""),
    }
  }
}

impl std::error::Error for VMError {}

#[derive(Debug)]
pub enum InvalidState {
  AlreadyRunning,
  UnexpectedInstruction(InstrType),
  HandMemDanglingPtr,
  ShutDown,
  Other(String),
}

#[derive(Debug)]
pub enum InstrType {
  IO,
  CPU,
  UnpredictableCPU,
}
