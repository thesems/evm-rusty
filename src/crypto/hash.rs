use alloy_primitives::{Keccak256, B256, U256};

pub fn hash_string_to_u256(text: &str) -> U256 {
    let mut hasher = Keccak256::new();
    hasher.update(text);
    U256::from_be_slice(&hasher.finalize().as_slice())
}

pub fn hash_slice_to_b256(buffer: &[u8]) -> B256 {
    let mut hasher = Keccak256::new();
    hasher.update(buffer);
    B256::from_slice(&hasher.finalize().as_slice())
}
