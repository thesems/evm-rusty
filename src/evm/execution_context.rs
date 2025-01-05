use alloy_primitives::Address;

pub struct ExecutionContext {
    pub caller: Address,
    pub address: Address,
    pub value: u64,
    pub data: Vec<u8>,
    pub gas: u64,
}

impl Default for ExecutionContext {
    fn default() -> Self {
        Self {
            caller: Address::ZERO,
            address: Address::ZERO,
            value: 0,
            data: Vec::new(),
            gas: 0,
        }
    }
}

impl ExecutionContext {
    pub fn new(caller: Address, address: Address, value: u64, data: Vec<u8>, gas: u64) -> Self {
        Self {
            caller,
            address,
            value,
            data,
            gas,
        }
    }
}
