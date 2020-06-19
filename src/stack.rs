use crate::execution_error::ExecutionError;
use ethereum_types::U256;

use std::fmt::{Debug, Formatter};
use std::result::Result;

pub struct Stack {
    stack: Vec<U256>,
}

const MAX_STACK_DEPTH: usize = 1024;

impl Stack {
    pub fn with_capacity(capacity: usize) -> Stack {
        Stack {
            stack: Vec::with_capacity(capacity),
        }
    }

    pub fn push(&mut self, value: U256) -> Result<(), ExecutionError> {
        if self.stack.len() == MAX_STACK_DEPTH {
            return Err(ExecutionError::StackOverflow);
        }

        self.stack.push(value);

        Ok(())
    }

    pub fn pop(&mut self) -> Result<U256, ExecutionError> {
        self.stack.pop().ok_or(ExecutionError::StackUnderflow)
    }

    pub fn read(&self, number_from_top: usize) -> Result<U256, ExecutionError> {
        if self.stack.len() <= number_from_top {
            return Err(ExecutionError::StackUnderflow);
        }

        Ok(self.stack[self.stack.len() - number_from_top - 1])
    }

    pub fn swap_with_top(&mut self, number_from_top: usize) -> Result<(), ExecutionError> {
        let stack_len = self.stack.len();

        if stack_len <= number_from_top {
            return Err(ExecutionError::StackUnderflow);
        }

        self.stack
            .swap(stack_len - 1, stack_len - number_from_top - 1);

        Ok(())
    }
}

impl Debug for Stack {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "[",)?;

        let mut buff = [0; 32];
        for elem in self.stack.iter().rev() {
            elem.to_big_endian(&mut buff);
            write!(f, "0x{} ", hex::encode_upper(buff).trim_start_matches('0'))?;
        }

        write!(f, "]")
    }
}
