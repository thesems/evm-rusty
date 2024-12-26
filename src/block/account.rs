use alloy_primitives::B256;

pub struct Account {
    // count of number transactions made or number of contracts made
    // only one transaction can use same nonce, to protect against replay attacks
    pub nonce: u64,
    pub balance: u64,
    // Contracts: hash of the the code?
    // EOA: hash of empty string
    pub code_hash: B256,
    // hash of the merkle patricia trie of the storage
    pub storage_root: B256,
}

impl Account {
    pub fn new() -> Self {
        Self {
            nonce: 0,
            balance: 0,
            code_hash: B256::ZERO,
            storage_root: B256::ZERO,
        }
    }
}
