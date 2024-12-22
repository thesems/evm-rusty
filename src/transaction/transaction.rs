// EIP-2718 - multiple transaction formats via Recursive Length Prefix (RLP) encoding

use alloy_primitives::{Address, Keccak256, B256};

pub struct Transaction {
    from: Address,
    to: Address,
    signature: B256,
    nonce: u64,
    value: u64,
    input_data: B256,
    // the maximum amount of gas units that can be consumed by the transaction.
    // The EVM specifies the units of gas required by each computational step
    gas_limit: u64,

    chain_id: u64,

    // EIP-2930
    // list of addresses and storage keys transaction intends to access
    // access_list: TBD

    // EIP-1559
    // the maximum price of the consumed gas to be included as a tip to the validator
    max_priority_fee_per_gas: u64,
    // the maximum fee per unit of gas willing to be paid for the transaction (inclusive of baseFeePerGas and maxPriorityFeePerGas)
    max_fee_per_gas: u64,
}

impl Transaction {
    // Calculate the hash that will be signed
    // This follows EIP-2718 and EIP-1559 transaction format
    fn hash_for_signing(&self) -> B256 {
        let mut hasher = Keccak256::new();

        // We use RLP encoding in practice, but for simplicity, we'll just concatenate fields
        // In a real implementation, you'd want to use proper RLP encoding here
        hasher.update(&[0x02]); // transaction type 2 (EIP-1559)
        hasher.update(&self.chain_id.to_be_bytes());
        hasher.update(&self.nonce.to_be_bytes());
        hasher.update(&self.max_priority_fee_per_gas.to_be_bytes());
        hasher.update(&self.max_fee_per_gas.to_be_bytes());
        hasher.update(&self.gas_limit.to_be_bytes());
        hasher.update(self.to.into_word().as_slice());
        hasher.update(&self.value.to_be_bytes());
        // In practice, we'd also include access_list and data field here

        hasher.finalize()
    }
}
