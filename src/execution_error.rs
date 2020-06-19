use crate::opcodes::Opcode;
use std::fmt::{Display, Formatter, Result};

#[derive(Debug, Eq, PartialEq)]
pub enum ExecutionError {
    StackOverflow,
    StackUnderflow,
    InvalidJump,
    Revert,
    InvalidOpcode,
    OutOfGas,
    UnsupportedOpcode(Opcode),
}

impl Display for ExecutionError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for ExecutionError {}
