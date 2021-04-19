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

pub enum VMError {
  FileNotFound(String), // path
  InvalidFile(String),  // reason
  IOError(std::io::Error),
  InvalidState(InvalidState), // reason
  EventNotDefined(i64),
}

pub enum InvalidState {
  AlreadyRunning,
  UnexpectedInstruction(String),
  HandMemDanglingPtr,
  ShutDown,
  Other(String),
}
