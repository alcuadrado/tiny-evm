use crate::bytecode::Bytecode;

use crate::context::{BlockContext, CallContext};
use crate::execution_error::ExecutionError;
use crate::opcode_handlers::{execute_opcode, ExecutionStatus, StepResult};
use crate::vm::VmState;

#[derive(Debug)]
pub struct ExecutionResult {
    pub return_data: Vec<u8>,
    pub error: Option<ExecutionError>,
}

pub fn run(
    bytecode: &Bytecode,
    call_context: &CallContext,
    block_context: &BlockContext,
) -> ExecutionResult {
    let mut vm_state = VmState::new();

    let bytecode_size = bytecode.size();

    loop {
        // vm_state.pc > bytecode_len means that the last instruction was a PUSH with incomplete
        // data, which is fine.
        // Apart from PUSH, only jumps could bring us to a similar situation, but those are handled
        // differently.
        if vm_state.pc >= bytecode_size {
            return ExecutionResult {
                error: None,
                return_data: vm_state.return_data,
            };
        }

        let step_result = run_next_step(&mut vm_state, bytecode, call_context, block_context);

        if let Err(error) = step_result {
            return ExecutionResult {
                error: Some(error),
                return_data: vm_state.return_data,
            };
        } else if let Ok(ExecutionStatus::Halted) = step_result {
            return ExecutionResult {
                error: None,
                return_data: vm_state.return_data,
            };
        }
    }
}

fn run_next_step(
    vm_state: &mut VmState,
    bytecode: &Bytecode,
    call_context: &CallContext,
    block_context: &BlockContext,
) -> StepResult {
    let opcode = bytecode.get_opcode_at(vm_state.pc);

    vm_state.pc += 1;

    execute_opcode(opcode, vm_state, bytecode, call_context, block_context)
}
