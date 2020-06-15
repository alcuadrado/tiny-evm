mod bytecode;
mod context;
mod evm;
mod execution_error;
mod i256;
mod memory;
mod opcode_handlers;
mod opcodes;
mod stack;
mod vm;

use crate::context::{BlockContext, CallContext};
use bytecode::Bytecode;
use evm::run;

fn main() {
    // This is the result of compiling:
    //  contract C { function sum(uint a, uint b) public pure returns (uint) { return a + b; } }
    let runtime_bytecode = hex::decode("6080604052348015600f57600080fd5b506004361060285760003560e01c8063cad0899b14602d575b600080fd5b606060048036036040811015604157600080fd5b8101908080359060200190929190803590602001909291905050506076565b6040518082815260200191505060405180910390f35b600081830190509291505056fea26469706673582212202af3fe2625b7faf66c537dbb4d9460001847afb68cb596f9e655c6b4d8fb652164736f6c63430006060033").unwrap();
    let bytecode = Bytecode::new(&runtime_bytecode);

    // Calling sum(1, 2)
    let calldata = hex::decode("cad0899b00000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000002").unwrap();

    // 3
    let expected_result =
        hex::decode("0000000000000000000000000000000000000000000000000000000000000003").unwrap();

    let call_context = CallContext {
        calldata: &calldata[..],
        ..CallContext::default()
    };

    let result = run(&bytecode, &call_context, &BlockContext::default());

    assert_eq!(result.error, Option::None);
    assert_eq!(result.return_data, expected_result);

    println!("{:?}", result);
}
