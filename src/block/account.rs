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

impl Default for Account {
    fn default() -> Self {
        Self::new(0, B256::ZERO, B256::ZERO)
    }
}

impl Account {
    pub fn new(balance: u64, code_hash: B256, storage_root: B256) -> Self {
        Self {
            nonce: 0,
            balance,
            code_hash,
            storage_root,
        }
    }
}
