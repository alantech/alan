use crate::vm::opcode::ByteOpcode;

#[derive(Debug)]
pub struct Instruction {
  // only unique per fn/handler
  pub(crate) id: i64,
  pub(crate) opcode: &'static ByteOpcode,
  pub(crate) args: Vec<i64>,
  pub(crate) dep_ids: Vec<i64>,
}
