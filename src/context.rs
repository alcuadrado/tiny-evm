use ethereum_types::Address;
use ethereum_types::U256;

#[derive(Debug)]
pub struct CallContext<'context> {
    pub value: U256,
    pub calldata: &'context [u8],
    pub contract_address: Address,
    pub caller_address: Address,
    pub origin_address: Address,
    pub gas_price: U256,
}

impl Default for CallContext<'_> {
    fn default() -> Self {
        CallContext {
            value: U256::zero(),
            calldata: &[],
            contract_address: Address::zero(),
            caller_address: Address::zero(),
            origin_address: Address::zero(),
            gas_price: U256::zero(),
        }
    }
}

#[derive(Debug)]
pub struct BlockContext {
    pub coinbase_address: Address,
    pub timestamp: u32,
    pub number: u32,
    pub gas_limit: u32,
    pub difficulty: u32,
    pub chain_id: u32,
}

impl Default for BlockContext {
    fn default() -> Self {
        BlockContext {
            coinbase_address: Address::zero(),
            timestamp: 0,
            number: 0,
            gas_limit: 0,
            difficulty: 0,
            chain_id: 0,
        }
    }
}
