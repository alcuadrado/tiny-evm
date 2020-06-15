use std::convert::TryFrom;
use std::iter;

use crate::opcodes::Opcode;
use ethereum_types::U256;

#[derive(Debug)]
pub struct Bytecode<'data> {
    data: &'data [u8],
    jumpdests: Vec<usize>,
}

impl<'data> Bytecode<'data> {
    pub fn new(data: &'data [u8]) -> Bytecode<'data> {
        Bytecode {
            data,
            jumpdests: BytecodeIterator::new(data)
                .filter(|inst| inst.opcode == Opcode::JUMPDEST)
                .map(|inst| inst.pc)
                .collect(),
        }
    }

    pub fn iter(&self) -> BytecodeIterator<'data> {
        BytecodeIterator::new(self.data)
    }

    pub fn get_opcode_at(&self, index: usize) -> Opcode {
        Opcode::try_from(self.data[index]).unwrap()
    }

    pub fn read_push_value(&self, start: usize, length: usize) -> U256 {
        let bytecode_len = self.data.len();
        if start >= bytecode_len {
            return U256::zero();
        }

        let end = std::cmp::min(start + length, bytecode_len);
        let push_data = &self.data[start..end];

        U256::from_big_endian(push_data)
    }

    pub fn as_bytes(&self) -> &[u8] {
        return self.data;
    }

    pub fn size(&self) -> usize {
        self.data.len()
    }

    pub fn is_jumpdest(&self, pc: usize) -> bool {
        if pc >= self.size() {
            return false;
        }

        self.jumpdests.contains(&pc)
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Instruction {
    pub opcode: Opcode,
    pub pc: usize,
}

pub struct BytecodeIterator<'data> {
    data: &'data [u8],
    next_byte: usize,
}

impl<'data> BytecodeIterator<'data> {
    fn new(data: &'data [u8]) -> BytecodeIterator<'data> {
        BytecodeIterator { data, next_byte: 0 }
    }
}

impl<'data> iter::Iterator for BytecodeIterator<'data> {
    type Item = Instruction;

    fn next(&mut self) -> Option<Self::Item> {
        if self.next_byte >= self.data.len() {
            return None;
        }

        let opcode = Opcode::try_from(self.data[self.next_byte]).unwrap();
        let pc = self.next_byte;

        self.next_byte += instruction_size(opcode);

        Some(Instruction { opcode, pc })
    }
}

fn instruction_size(opcode: Opcode) -> usize {
    let n = opcode as u8;
    let push1 = Opcode::PUSH1 as u8;
    let push32 = Opcode::PUSH32 as u8;

    if n >= push1 && n <= push32 {
        return (n - push1 + 2) as usize;
    }

    return 1;
}
