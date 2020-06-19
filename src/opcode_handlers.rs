use crate::bytecode::Bytecode;
use crate::execution_error::ExecutionError;
use crate::opcodes::Opcode;

use crate::execution_error::ExecutionError::{
    InvalidJump, InvalidOpcode, OutOfGas, Revert, UnsupportedOpcode,
};

use crate::context::{BlockContext, CallContext};
use crate::i256::{Sign, I256};
use crate::memory::{Memory, MEMORY_LIMIT};
use crate::stack::Stack;
use crate::vm::VmState;
use ethereum_types::{Address, U256, U512};
use sha3::{Digest, Keccak256};
use std::convert::TryFrom;
use std::io::Write;
use ExecutionStatus::{Halted, Running};

#[derive(Debug)]
pub enum ExecutionStatus {
    Running,
    Halted,
}

pub type StepResult = Result<ExecutionStatus, ExecutionError>;

pub fn execute_opcode(
    opcode: Opcode,
    vm_state: &mut VmState,
    bytecode: &Bytecode,
    call_context: &CallContext,
    block_context: &BlockContext,
) -> StepResult {
    match opcode {
        Opcode::STOP => Ok(Halted),
        Opcode::ADD => {
            let u0 = vm_state.stack.pop()?;
            let u1 = vm_state.stack.pop()?;

            let (result, _) = u0.overflowing_add(u1);

            vm_state.stack.push(result)?;

            Ok(Running)
        }
        Opcode::MUL => {
            let u0 = vm_state.stack.pop()?;
            let u1 = vm_state.stack.pop()?;

            let (result, _) = u0.overflowing_mul(u1);

            vm_state.stack.push(result)?;

            Ok(Running)
        }
        Opcode::SUB => {
            let u0 = vm_state.stack.pop()?;
            let u1 = vm_state.stack.pop()?;

            let (result, _) = u0.overflowing_sub(u1);

            vm_state.stack.push(result)?;

            Ok(Running)
        }
        Opcode::DIV => {
            let u0 = vm_state.stack.pop()?;
            let u1 = vm_state.stack.pop()?;

            let result = if u1 == U256::zero() {
                U256::zero()
            } else {
                u0 / u1
            };

            vm_state.stack.push(result)?;

            Ok(Running)
        }
        Opcode::SDIV => {
            let u0 = vm_state.stack.pop()?;
            let u1 = vm_state.stack.pop()?;

            let op1: I256 = u0.into();
            let op2: I256 = u1.into();
            let value: U256 = (op1 / op2).into();

            vm_state.stack.push(value)?;

            Ok(Running)
        }
        Opcode::MOD => {
            let u0 = vm_state.stack.pop()?;
            let u1 = vm_state.stack.pop()?;

            let result = if u1 == U256::zero() {
                U256::zero()
            } else {
                u0 % u1
            };

            vm_state.stack.push(result)?;

            Ok(Running)
        }
        Opcode::SMOD => {
            let u0 = vm_state.stack.pop()?;
            let u1 = vm_state.stack.pop()?;

            let result = if u1 == U256::zero() {
                U256::zero()
            } else {
                let s0: I256 = u0.into();
                let s1: I256 = u1.into();

                (s0 % s1).into()
            };

            vm_state.stack.push(result)?;

            Ok(Running)
        }
        Opcode::ADDMOD => {
            let u0: U512 = vm_state.stack.pop()?.into();
            let u1: U512 = vm_state.stack.pop()?.into();
            let u2: U512 = vm_state.stack.pop()?.into();

            let result = if u2 == U512::zero() {
                U256::zero()
            } else {
                let value = (u0 + u1) % u2;

                U256::try_from(value)
                    .expect("We applied (mod 256_bits_value), so the result fits in 256 bits")
            };

            vm_state.stack.push(result)?;

            Ok(Running)
        }
        Opcode::MULMOD => {
            let u0: U512 = vm_state.stack.pop()?.into();
            let u1: U512 = vm_state.stack.pop()?.into();
            let u2: U512 = vm_state.stack.pop()?.into();

            let result = if u2 == U512::zero() {
                U256::zero()
            } else {
                let value = (u0 * u1) % u2;

                U256::try_from(value)
                    .expect("We applied (mod 256_bits_value), so the result fits in 256 bits")
            };

            vm_state.stack.push(result)?;

            Ok(Running)
        }
        Opcode::EXP => {
            let u0 = vm_state.stack.pop()?;
            let u1 = vm_state.stack.pop()?;

            let (result, _) = u0.overflowing_pow(u1);

            vm_state.stack.push(result)?;

            Ok(Running)
        }
        Opcode::SIGNEXTEND => {
            let u0 = vm_state.stack.pop()?;
            let u1 = vm_state.stack.pop()?;

            let value = if u0 < U256::from(32) {
                let value_bytes = u0 + U256::one();
                let sig_bit = 256 - 8 * value_bytes.as_usize();

                let value_mask = (U256::one() << sig_bit) - U256::one();

                let sig = u1.bit(sig_bit);

                if sig {
                    u1 | !value_mask
                } else {
                    u1 & value_mask
                }
            } else {
                u1
            };

            vm_state.stack.push(value)?;

            Ok(Running)
        }
        Opcode::UNRECOGNIZED0C => Err(InvalidOpcode),
        Opcode::UNRECOGNIZED0D => Err(InvalidOpcode),
        Opcode::UNRECOGNIZED0E => Err(InvalidOpcode),
        Opcode::UNRECOGNIZED0F => Err(InvalidOpcode),
        Opcode::LT => {
            let u0 = vm_state.stack.pop()?;
            let u1 = vm_state.stack.pop()?;

            let result = if u0 < u1 { U256::one() } else { U256::zero() };

            vm_state.stack.push(result)?;

            Ok(Running)
        }
        Opcode::GT => {
            let u0 = vm_state.stack.pop()?;
            let u1 = vm_state.stack.pop()?;

            let result = if u0 > u1 { U256::one() } else { U256::zero() };

            vm_state.stack.push(result)?;

            Ok(Running)
        }
        Opcode::SLT => {
            let u0 = vm_state.stack.pop()?;
            let u1 = vm_state.stack.pop()?;

            let s0: I256 = u0.into();
            let s1: I256 = u1.into();

            let result = if s0 < s1 { U256::one() } else { U256::zero() };

            vm_state.stack.push(result)?;

            Ok(Running)
        }
        Opcode::SGT => {
            let u0 = vm_state.stack.pop()?;
            let u1 = vm_state.stack.pop()?;

            let s0: I256 = u0.into();
            let s1: I256 = u1.into();

            let result = if s0 > s1 { U256::one() } else { U256::zero() };

            vm_state.stack.push(result)?;

            Ok(Running)
        }
        Opcode::EQ => {
            let u0 = vm_state.stack.pop()?;
            let u1 = vm_state.stack.pop()?;

            let result = if u0 == u1 { U256::one() } else { U256::zero() };

            vm_state.stack.push(result)?;

            Ok(Running)
        }
        Opcode::ISZERO => {
            let u0 = vm_state.stack.pop()?;

            let result = if u0 == U256::zero() {
                U256::one()
            } else {
                U256::zero()
            };

            vm_state.stack.push(result)?;

            Ok(Running)
        }
        Opcode::AND => {
            let u0 = vm_state.stack.pop()?;
            let u1 = vm_state.stack.pop()?;

            let result = u0 & u1;

            vm_state.stack.push(result)?;

            Ok(Running)
        }
        Opcode::OR => {
            let u0 = vm_state.stack.pop()?;
            let u1 = vm_state.stack.pop()?;

            let result = u0 | u1;

            vm_state.stack.push(result)?;

            Ok(Running)
        }
        Opcode::XOR => {
            let u0 = vm_state.stack.pop()?;
            let u1 = vm_state.stack.pop()?;

            let result = u0 | u1;

            vm_state.stack.push(result)?;

            Ok(Running)
        }
        Opcode::NOT => {
            let u0 = vm_state.stack.pop()?;

            let result = !u0;

            vm_state.stack.push(result)?;

            Ok(Running)
        }
        Opcode::BYTE => {
            let u0 = vm_state.stack.pop()?;
            let u1 = vm_state.stack.pop()?;

            let result = if u0 > U256::from(31) {
                U256::zero()
            } else {
                (u1 >> (U256::from(31) - u0) * 8) & U256::from(0xFF)
            };

            vm_state.stack.push(result)?;

            Ok(Running)
        }
        Opcode::SHL => {
            let u0 = vm_state.stack.pop()?;
            let u1 = vm_state.stack.pop()?;

            let result = u1 << u0;

            vm_state.stack.push(result)?;

            Ok(Running)
        }
        Opcode::SHR => {
            let u0 = vm_state.stack.pop()?;
            let u1 = vm_state.stack.pop()?;

            let result = u1 >> u0;

            vm_state.stack.push(result)?;

            Ok(Running)
        }
        Opcode::SAR => {
            let u0 = vm_state.stack.pop()?;
            let u1 = vm_state.stack.pop()?;

            // This was heavily inspired by sorpass
            let result = if u1 == U256::zero() {
                U256::zero()
            } else if u0 >= U256::from(256) {
                if u1 > U256::zero() {
                    U256::zero()
                } else {
                    I256(Sign::Minus, U256::one()).into()
                }
            } else {
                let I256(sig, value) = I256::from(u1);

                if sig == Sign::Plus {
                    value >> u0
                } else {
                    let shift = u0.as_usize();
                    let shifted = ((value.overflowing_sub(U256::one()).0) >> shift)
                        .overflowing_add(U256::one())
                        .0;
                    I256(Sign::Minus, shifted).into()
                }
            };

            vm_state.stack.push(result)?;

            Ok(Running)
        }
        Opcode::UNRECOGNIZED1E => Err(InvalidOpcode),
        Opcode::UNRECOGNIZED1F => Err(InvalidOpcode),
        Opcode::SHA3 => {
            let u0 = vm_state.stack.pop()?;
            let u1 = vm_state.stack.pop()?;

            ensure_offset_and_length_fit_usize(u0, u1)?;
            let offset = u0.as_usize();
            let length = u1.as_usize();

            let mut keccak = Keccak256::new();

            if offset >= vm_state.memory.size() {
                if length >= MEMORY_LIMIT {
                    return Err(OutOfGas);
                }

                let data = vec![0; length];

                keccak
                    .write(data.as_slice())
                    .expect("Keccak's write should never fail");
            } else {
                let data = vm_state.memory.read(offset, length)?;
                keccak
                    .write(data)
                    .expect("Keccak's write should never fail");
            };

            let hash = keccak.finalize();

            let result = U256::from(hash.as_slice());

            vm_state.stack.push(result)?;

            Ok(Running)
        }
        Opcode::UNRECOGNIZED21 => Err(InvalidOpcode),
        Opcode::UNRECOGNIZED22 => Err(InvalidOpcode),
        Opcode::UNRECOGNIZED23 => Err(InvalidOpcode),
        Opcode::UNRECOGNIZED24 => Err(InvalidOpcode),
        Opcode::UNRECOGNIZED25 => Err(InvalidOpcode),
        Opcode::UNRECOGNIZED26 => Err(InvalidOpcode),
        Opcode::UNRECOGNIZED27 => Err(InvalidOpcode),
        Opcode::UNRECOGNIZED28 => Err(InvalidOpcode),
        Opcode::UNRECOGNIZED29 => Err(InvalidOpcode),
        Opcode::UNRECOGNIZED2A => Err(InvalidOpcode),
        Opcode::UNRECOGNIZED2B => Err(InvalidOpcode),
        Opcode::UNRECOGNIZED2C => Err(InvalidOpcode),
        Opcode::UNRECOGNIZED2D => Err(InvalidOpcode),
        Opcode::UNRECOGNIZED2E => Err(InvalidOpcode),
        Opcode::UNRECOGNIZED2F => Err(InvalidOpcode),
        Opcode::ADDRESS => {
            let address = address_to_u256(&call_context.contract_address);

            vm_state.stack.push(address)?;

            Ok(Running)
        }
        Opcode::BALANCE => unsupported_opcode_handler(Opcode::BALANCE),
        Opcode::ORIGIN => {
            let address = address_to_u256(&call_context.origin_address);

            vm_state.stack.push(address)?;

            Ok(Running)
        }
        Opcode::CALLER => {
            let address = address_to_u256(&call_context.caller_address);

            vm_state.stack.push(address)?;

            Ok(Running)
        }
        Opcode::CALLVALUE => {
            vm_state.stack.push(call_context.value)?;

            Ok(Running)
        }
        Opcode::CALLDATALOAD => {
            let u0 = vm_state.stack.pop()?;

            let value = if u0 > U256::from(usize::max_value()) {
                U256::zero()
            } else {
                let data = get_slice(call_context.calldata, u0.as_usize(), 32);
                U256::from(data)
            };

            vm_state.stack.push(value)?;

            Ok(Running)
        }
        Opcode::CALLDATASIZE => {
            let size = call_context.calldata.len();

            vm_state.stack.push(U256::from(size))?;

            Ok(Running)
        }
        Opcode::CALLDATACOPY => data_copy_handler(
            &mut vm_state.stack,
            &mut vm_state.memory,
            call_context.calldata,
        ),
        Opcode::CODESIZE => {
            let size = bytecode.size();

            vm_state.stack.push(U256::from(size))?;

            Ok(Running)
        }
        Opcode::CODECOPY => data_copy_handler(
            &mut vm_state.stack,
            &mut vm_state.memory,
            bytecode.as_bytes(),
        ),
        Opcode::GASPRICE => {
            vm_state.stack.push(call_context.gas_price)?;

            Ok(Running)
        }
        Opcode::EXTCODESIZE => unsupported_opcode_handler(Opcode::EXTCODESIZE),
        Opcode::EXTCODECOPY => unsupported_opcode_handler(Opcode::EXTCODECOPY),
        Opcode::RETURNDATASIZE => {
            let size = vm_state.return_data.len();

            vm_state.stack.push(U256::from(size))?;

            Ok(Running)
        }
        Opcode::RETURNDATACOPY => data_copy_handler(
            &mut vm_state.stack,
            &mut vm_state.memory,
            vm_state.return_data.as_slice(),
        ),
        Opcode::EXTCODEHASH => unsupported_opcode_handler(Opcode::EXTCODEHASH),
        Opcode::BLOCKHASH => unsupported_opcode_handler(Opcode::BLOCKHASH),
        Opcode::COINBASE => {
            let address = address_to_u256(&block_context.coinbase_address);

            vm_state.stack.push(address)?;

            Ok(Running)
        }
        Opcode::TIMESTAMP => {
            let value = U256::from(block_context.timestamp);

            vm_state.stack.push(value)?;

            Ok(Running)
        }
        Opcode::NUMBER => {
            let value = U256::from(block_context.number);

            vm_state.stack.push(value)?;

            Ok(Running)
        }
        Opcode::DIFFICULTY => {
            let value = U256::from(block_context.difficulty);

            vm_state.stack.push(value)?;

            Ok(Running)
        }
        Opcode::GASLIMIT => {
            let value = U256::from(block_context.gas_limit);

            vm_state.stack.push(value)?;

            Ok(Running)
        }
        Opcode::CHAINID => {
            let value = U256::from(block_context.chain_id);

            vm_state.stack.push(value)?;

            Ok(Running)
        }
        Opcode::UNRECOGNIZED47 => Err(InvalidOpcode),
        Opcode::UNRECOGNIZED48 => Err(InvalidOpcode),
        Opcode::UNRECOGNIZED49 => Err(InvalidOpcode),
        Opcode::UNRECOGNIZED4A => Err(InvalidOpcode),
        Opcode::UNRECOGNIZED4B => Err(InvalidOpcode),
        Opcode::UNRECOGNIZED4C => Err(InvalidOpcode),
        Opcode::UNRECOGNIZED4D => Err(InvalidOpcode),
        Opcode::UNRECOGNIZED4E => Err(InvalidOpcode),
        Opcode::UNRECOGNIZED4F => Err(InvalidOpcode),
        Opcode::POP => {
            vm_state.stack.pop()?;

            Ok(Running)
        }
        Opcode::MLOAD => {
            let u0 = vm_state.stack.pop()?;

            ensure_fits_usize(u0)?;
            let offset = u0.as_usize();

            let data = vm_state.memory.read(offset, 32)?;
            let value = U256::from(data);

            vm_state.stack.push(value)?;

            Ok(Running)
        }
        Opcode::MSTORE => {
            let u0 = vm_state.stack.pop()?;
            let u1 = vm_state.stack.pop()?;

            let mut bytes = [0; 32];
            u1.to_big_endian(&mut bytes);

            ensure_fits_usize(u0)?;
            vm_state.memory.write(u0.as_usize(), 32, &bytes)?;

            Ok(Running)
        }
        Opcode::MSTORE8 => {
            let u0 = vm_state.stack.pop()?;
            let u1 = vm_state.stack.pop()?;

            let byte = u1.byte(0);

            ensure_fits_usize(u0)?;
            vm_state.memory.write(u0.as_usize(), 1, &[byte])?;

            Ok(Running)
        }
        Opcode::SLOAD => {
            let u0 = vm_state.stack.pop()?;

            let default = U256::zero();
            let value = vm_state.storage.get(&u0).unwrap_or(&default);

            vm_state.stack.push(*value)?;

            Ok(Running)
        }
        Opcode::SSTORE => {
            let u0 = vm_state.stack.pop()?;
            let u1 = vm_state.stack.pop()?;

            vm_state.storage.insert(u0, u1);

            Ok(Running)
        }
        Opcode::JUMP => {
            let u0 = vm_state.stack.pop()?;

            jump(vm_state, bytecode, u0)
        }
        Opcode::JUMPI => {
            let u0 = vm_state.stack.pop()?;
            let u1 = vm_state.stack.pop()?;

            if u1 != U256::zero() {
                return jump(vm_state, bytecode, u0);
            }

            Ok(Running)
        }
        Opcode::PC => {
            let pc = U256::from(vm_state.pc - 1);

            vm_state.stack.push(pc)?;

            Ok(Running)
        }
        Opcode::MSIZE => {
            let size = U256::from(vm_state.memory.size());

            vm_state.stack.push(size)?;

            Ok(Running)
        }
        Opcode::GAS => unsupported_opcode_handler(Opcode::GAS),
        Opcode::JUMPDEST => Ok(Running),
        Opcode::UNRECOGNIZED5C => Err(InvalidOpcode),
        Opcode::UNRECOGNIZED5D => Err(InvalidOpcode),
        Opcode::UNRECOGNIZED5E => Err(InvalidOpcode),
        Opcode::UNRECOGNIZED5F => Err(InvalidOpcode),
        Opcode::PUSH1 => push_handler(vm_state, bytecode, 1),
        Opcode::PUSH2 => push_handler(vm_state, bytecode, 2),
        Opcode::PUSH3 => push_handler(vm_state, bytecode, 3),
        Opcode::PUSH4 => push_handler(vm_state, bytecode, 4),
        Opcode::PUSH5 => push_handler(vm_state, bytecode, 5),
        Opcode::PUSH6 => push_handler(vm_state, bytecode, 6),
        Opcode::PUSH7 => push_handler(vm_state, bytecode, 7),
        Opcode::PUSH8 => push_handler(vm_state, bytecode, 8),
        Opcode::PUSH9 => push_handler(vm_state, bytecode, 9),
        Opcode::PUSH10 => push_handler(vm_state, bytecode, 10),
        Opcode::PUSH11 => push_handler(vm_state, bytecode, 11),
        Opcode::PUSH12 => push_handler(vm_state, bytecode, 12),
        Opcode::PUSH13 => push_handler(vm_state, bytecode, 13),
        Opcode::PUSH14 => push_handler(vm_state, bytecode, 14),
        Opcode::PUSH15 => push_handler(vm_state, bytecode, 15),
        Opcode::PUSH16 => push_handler(vm_state, bytecode, 16),
        Opcode::PUSH17 => push_handler(vm_state, bytecode, 17),
        Opcode::PUSH18 => push_handler(vm_state, bytecode, 18),
        Opcode::PUSH19 => push_handler(vm_state, bytecode, 19),
        Opcode::PUSH20 => push_handler(vm_state, bytecode, 20),
        Opcode::PUSH21 => push_handler(vm_state, bytecode, 21),
        Opcode::PUSH22 => push_handler(vm_state, bytecode, 22),
        Opcode::PUSH23 => push_handler(vm_state, bytecode, 23),
        Opcode::PUSH24 => push_handler(vm_state, bytecode, 24),
        Opcode::PUSH25 => push_handler(vm_state, bytecode, 25),
        Opcode::PUSH26 => push_handler(vm_state, bytecode, 26),
        Opcode::PUSH27 => push_handler(vm_state, bytecode, 27),
        Opcode::PUSH28 => push_handler(vm_state, bytecode, 28),
        Opcode::PUSH29 => push_handler(vm_state, bytecode, 29),
        Opcode::PUSH30 => push_handler(vm_state, bytecode, 30),
        Opcode::PUSH31 => push_handler(vm_state, bytecode, 31),
        Opcode::PUSH32 => push_handler(vm_state, bytecode, 32),
        Opcode::DUP1 => dup_handler(vm_state, 1),
        Opcode::DUP2 => dup_handler(vm_state, 2),
        Opcode::DUP3 => dup_handler(vm_state, 3),
        Opcode::DUP4 => dup_handler(vm_state, 4),
        Opcode::DUP5 => dup_handler(vm_state, 5),
        Opcode::DUP6 => dup_handler(vm_state, 6),
        Opcode::DUP7 => dup_handler(vm_state, 7),
        Opcode::DUP8 => dup_handler(vm_state, 8),
        Opcode::DUP9 => dup_handler(vm_state, 9),
        Opcode::DUP10 => dup_handler(vm_state, 10),
        Opcode::DUP11 => dup_handler(vm_state, 11),
        Opcode::DUP12 => dup_handler(vm_state, 12),
        Opcode::DUP13 => dup_handler(vm_state, 13),
        Opcode::DUP14 => dup_handler(vm_state, 14),
        Opcode::DUP15 => dup_handler(vm_state, 15),
        Opcode::DUP16 => dup_handler(vm_state, 16),
        Opcode::SWAP1 => swap_handler(vm_state, 1),
        Opcode::SWAP2 => swap_handler(vm_state, 2),
        Opcode::SWAP3 => swap_handler(vm_state, 3),
        Opcode::SWAP4 => swap_handler(vm_state, 4),
        Opcode::SWAP5 => swap_handler(vm_state, 5),
        Opcode::SWAP6 => swap_handler(vm_state, 6),
        Opcode::SWAP7 => swap_handler(vm_state, 7),
        Opcode::SWAP8 => swap_handler(vm_state, 8),
        Opcode::SWAP9 => swap_handler(vm_state, 9),
        Opcode::SWAP10 => swap_handler(vm_state, 10),
        Opcode::SWAP11 => swap_handler(vm_state, 11),
        Opcode::SWAP12 => swap_handler(vm_state, 12),
        Opcode::SWAP13 => swap_handler(vm_state, 13),
        Opcode::SWAP14 => swap_handler(vm_state, 14),
        Opcode::SWAP15 => swap_handler(vm_state, 15),
        Opcode::SWAP16 => swap_handler(vm_state, 16),
        Opcode::LOG0 => unsupported_opcode_handler(Opcode::LOG0),
        Opcode::LOG1 => unsupported_opcode_handler(Opcode::LOG1),
        Opcode::LOG2 => unsupported_opcode_handler(Opcode::LOG2),
        Opcode::LOG3 => unsupported_opcode_handler(Opcode::LOG3),
        Opcode::LOG4 => unsupported_opcode_handler(Opcode::LOG4),
        Opcode::UNRECOGNIZEDA5 => Err(InvalidOpcode),
        Opcode::UNRECOGNIZEDA6 => Err(InvalidOpcode),
        Opcode::UNRECOGNIZEDA7 => Err(InvalidOpcode),
        Opcode::UNRECOGNIZEDA8 => Err(InvalidOpcode),
        Opcode::UNRECOGNIZEDA9 => Err(InvalidOpcode),
        Opcode::UNRECOGNIZEDAA => Err(InvalidOpcode),
        Opcode::UNRECOGNIZEDAB => Err(InvalidOpcode),
        Opcode::UNRECOGNIZEDAC => Err(InvalidOpcode),
        Opcode::UNRECOGNIZEDAD => Err(InvalidOpcode),
        Opcode::UNRECOGNIZEDAE => Err(InvalidOpcode),
        Opcode::UNRECOGNIZEDAF => Err(InvalidOpcode),
        Opcode::UNRECOGNIZEDB0 => Err(InvalidOpcode),
        Opcode::UNRECOGNIZEDB1 => Err(InvalidOpcode),
        Opcode::UNRECOGNIZEDB2 => Err(InvalidOpcode),
        Opcode::UNRECOGNIZEDB3 => Err(InvalidOpcode),
        Opcode::UNRECOGNIZEDB4 => Err(InvalidOpcode),
        Opcode::UNRECOGNIZEDB5 => Err(InvalidOpcode),
        Opcode::UNRECOGNIZEDB6 => Err(InvalidOpcode),
        Opcode::UNRECOGNIZEDB7 => Err(InvalidOpcode),
        Opcode::UNRECOGNIZEDB8 => Err(InvalidOpcode),
        Opcode::UNRECOGNIZEDB9 => Err(InvalidOpcode),
        Opcode::UNRECOGNIZEDBA => Err(InvalidOpcode),
        Opcode::UNRECOGNIZEDBB => Err(InvalidOpcode),
        Opcode::UNRECOGNIZEDBC => Err(InvalidOpcode),
        Opcode::UNRECOGNIZEDBD => Err(InvalidOpcode),
        Opcode::UNRECOGNIZEDBE => Err(InvalidOpcode),
        Opcode::UNRECOGNIZEDBF => Err(InvalidOpcode),
        Opcode::UNRECOGNIZEDC0 => Err(InvalidOpcode),
        Opcode::UNRECOGNIZEDC1 => Err(InvalidOpcode),
        Opcode::UNRECOGNIZEDC2 => Err(InvalidOpcode),
        Opcode::UNRECOGNIZEDC3 => Err(InvalidOpcode),
        Opcode::UNRECOGNIZEDC4 => Err(InvalidOpcode),
        Opcode::UNRECOGNIZEDC5 => Err(InvalidOpcode),
        Opcode::UNRECOGNIZEDC6 => Err(InvalidOpcode),
        Opcode::UNRECOGNIZEDC7 => Err(InvalidOpcode),
        Opcode::UNRECOGNIZEDC8 => Err(InvalidOpcode),
        Opcode::UNRECOGNIZEDC9 => Err(InvalidOpcode),
        Opcode::UNRECOGNIZEDCA => Err(InvalidOpcode),
        Opcode::UNRECOGNIZEDCB => Err(InvalidOpcode),
        Opcode::UNRECOGNIZEDCC => Err(InvalidOpcode),
        Opcode::UNRECOGNIZEDCD => Err(InvalidOpcode),
        Opcode::UNRECOGNIZEDCE => Err(InvalidOpcode),
        Opcode::UNRECOGNIZEDCF => Err(InvalidOpcode),
        Opcode::UNRECOGNIZEDD0 => Err(InvalidOpcode),
        Opcode::UNRECOGNIZEDD1 => Err(InvalidOpcode),
        Opcode::UNRECOGNIZEDD2 => Err(InvalidOpcode),
        Opcode::UNRECOGNIZEDD3 => Err(InvalidOpcode),
        Opcode::UNRECOGNIZEDD4 => Err(InvalidOpcode),
        Opcode::UNRECOGNIZEDD5 => Err(InvalidOpcode),
        Opcode::UNRECOGNIZEDD6 => Err(InvalidOpcode),
        Opcode::UNRECOGNIZEDD7 => Err(InvalidOpcode),
        Opcode::UNRECOGNIZEDD8 => Err(InvalidOpcode),
        Opcode::UNRECOGNIZEDD9 => Err(InvalidOpcode),
        Opcode::UNRECOGNIZEDDA => Err(InvalidOpcode),
        Opcode::UNRECOGNIZEDDB => Err(InvalidOpcode),
        Opcode::UNRECOGNIZEDDC => Err(InvalidOpcode),
        Opcode::UNRECOGNIZEDDD => Err(InvalidOpcode),
        Opcode::UNRECOGNIZEDDE => Err(InvalidOpcode),
        Opcode::UNRECOGNIZEDDF => Err(InvalidOpcode),
        Opcode::UNRECOGNIZEDE0 => Err(InvalidOpcode),
        Opcode::UNRECOGNIZEDE1 => Err(InvalidOpcode),
        Opcode::UNRECOGNIZEDE2 => Err(InvalidOpcode),
        Opcode::UNRECOGNIZEDE3 => Err(InvalidOpcode),
        Opcode::UNRECOGNIZEDE4 => Err(InvalidOpcode),
        Opcode::UNRECOGNIZEDE5 => Err(InvalidOpcode),
        Opcode::UNRECOGNIZEDE6 => Err(InvalidOpcode),
        Opcode::UNRECOGNIZEDE7 => Err(InvalidOpcode),
        Opcode::UNRECOGNIZEDE8 => Err(InvalidOpcode),
        Opcode::UNRECOGNIZEDE9 => Err(InvalidOpcode),
        Opcode::UNRECOGNIZEDEA => Err(InvalidOpcode),
        Opcode::UNRECOGNIZEDEB => Err(InvalidOpcode),
        Opcode::UNRECOGNIZEDEC => Err(InvalidOpcode),
        Opcode::UNRECOGNIZEDED => Err(InvalidOpcode),
        Opcode::UNRECOGNIZEDEE => Err(InvalidOpcode),
        Opcode::UNRECOGNIZEDEF => Err(InvalidOpcode),
        Opcode::CREATE => unsupported_opcode_handler(Opcode::CREATE),
        Opcode::CALL => unsupported_opcode_handler(Opcode::CALL),
        Opcode::CALLCODE => unsupported_opcode_handler(Opcode::CALLCODE),
        Opcode::RETURN => {
            let u0 = vm_state.stack.pop()?;
            let u1 = vm_state.stack.pop()?;

            ensure_offset_and_length_fit_usize(u0, u1)?;
            let offset = u0.as_usize();
            let length = u1.as_usize();

            let data = vm_state.memory.read(offset, length)?;

            vm_state.return_data.extend_from_slice(data);

            Ok(Halted)
        }
        Opcode::DELEGATECALL => unsupported_opcode_handler(Opcode::DELEGATECALL),
        Opcode::CREATE2 => unsupported_opcode_handler(Opcode::CREATE2),
        Opcode::UNRECOGNIZEDF6 => Err(InvalidOpcode),
        Opcode::UNRECOGNIZEDF7 => Err(InvalidOpcode),
        Opcode::UNRECOGNIZEDF8 => Err(InvalidOpcode),
        Opcode::UNRECOGNIZEDF9 => Err(InvalidOpcode),
        Opcode::STATICCALL => unsupported_opcode_handler(Opcode::STATICCALL),
        Opcode::UNRECOGNIZEDFB => Err(InvalidOpcode),
        Opcode::UNRECOGNIZEDFC => Err(InvalidOpcode),
        Opcode::REVERT => {
            let u0 = vm_state.stack.pop()?;
            let u1 = vm_state.stack.pop()?;

            ensure_offset_and_length_fit_usize(u0, u1)?;
            let offset = u0.as_usize();
            let length = u1.as_usize();

            let data = vm_state.memory.read(offset, length)?;

            vm_state.return_data.extend_from_slice(data);

            Err(Revert)
        }
        Opcode::INVALID => Err(InvalidOpcode),
        Opcode::SELFDESTRUCT => Ok(Halted),
    }
}

fn jump(vm_state: &mut VmState, bytecode: &Bytecode, dest: U256) -> StepResult {
    if dest >= U256::from(bytecode.size()) {
        return Err(InvalidJump);
    }

    let pc = dest.as_usize();
    if !bytecode.is_jumpdest(pc) {
        return Err(InvalidJump);
    }

    vm_state.pc = pc;

    Ok(Running)
}

fn push_handler(vm_state: &mut VmState, bytecode: &Bytecode, size: usize) -> StepResult {
    let value = bytecode.read_push_value(vm_state.pc, size);
    vm_state.stack.push(value)?;

    vm_state.pc += size;

    Ok(Running)
}

fn dup_handler(vm_state: &mut VmState, dup_number: usize) -> StepResult {
    let value = vm_state.stack.read(dup_number - 1)?;
    vm_state.stack.push(value)?;

    Ok(Running)
}

fn swap_handler(vm_state: &mut VmState, swap_number: usize) -> StepResult {
    vm_state.stack.swap_with_top(swap_number)?;

    Ok(Running)
}

fn unsupported_opcode_handler(opcode: Opcode) -> StepResult {
    Err(UnsupportedOpcode(opcode))
}

fn get_slice(data: &[u8], start: usize, length: usize) -> &[u8] {
    let data_len = data.len();
    if start >= data_len {
        return &[];
    }

    let end = std::cmp::min(start + length, data_len);
    &data[start..end]
}

fn address_to_u256(address: &Address) -> U256 {
    U256::from_big_endian(address.as_bytes())
}

fn data_copy_handler(stack: &mut Stack, memory: &mut Memory, data: &[u8]) -> StepResult {
    let u0 = stack.pop()?;
    let u1 = stack.pop()?;
    let u2 = stack.pop()?;

    ensure_fits_usize(u0)?;

    let u0_usize = u0.as_usize();
    let u2_usize = u2.as_usize();

    let data = if u1 > U256::from(usize::max_value()) {
        // We don't really need the data here, we use an empty slice
        // and write will take care of this
        &[]
    } else {
        let u1_usize = u1.as_usize();
        ensure_offset_and_length_fit_usize(u1, u2)?;

        get_slice(data, u1_usize, u2_usize)
    };

    memory.write(u0_usize, u2_usize, data)?;

    Ok(Running)
}

fn ensure_offset_and_length_fit_usize(offset: U256, length: U256) -> Result<(), ExecutionError> {
    let (size, overflow) = offset.overflowing_add(length);
    if overflow {
        return Err(OutOfGas);
    }

    ensure_fits_usize(size)?;

    Ok(())
}

fn ensure_fits_usize(number: U256) -> Result<(), ExecutionError> {
    if number > U256::from(usize::max_value()) {
        return Err(OutOfGas);
    }

    Ok(())
}
