use crate::memory::Memory;
use crate::stack::Stack;

#[derive(Debug)]
pub struct VmState {
    pub pc: usize,
    pub stack: Stack,
    pub memory: Memory,
    pub return_data: Vec<u8>,
}

// Solidity always writes the free pointer in 0x40, so we same some allocations by starting with
// that capacity.
const INITIAL_MEMORY_CAPACITY: usize = 0x40 + 32;

impl VmState {
    pub fn new() -> VmState {
        VmState {
            pc: 0,
            stack: Stack::with_capacity(16),
            memory: Memory::with_capacity(INITIAL_MEMORY_CAPACITY),
            return_data: Vec::new(),
        }
    }
}
