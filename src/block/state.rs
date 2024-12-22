use std::collections::HashMap;

use alloy_primitives::{Address, B256};

use crate::block::account::Account;

pub struct State {
    pub accounts: HashMap<Address, Account>,
    pub storage: HashMap<(Address, B256), B256>,
}

impl State {
    pub fn new() -> Self {
        Self {
            accounts: HashMap::new(),
            storage: HashMap::new(),
        }
    }

    pub fn get_account(&mut self, address: &Address) -> Option<&mut Account> {
        self.accounts.get_mut(address)
    }

    pub fn set_account(&mut self, address: Address, account: Account) {
        self.accounts.insert(address, account);
    }

    pub fn get_storage(&self, address: &Address, key: &B256) -> B256 {
        self.storage
            .get(&(*address, *key))
            .copied()
            .unwrap_or_default()
    }

    pub fn set_storage(&mut self, address: Address, key: B256, value: B256) {
        self.storage.insert((address, key), value);
    }

    pub fn process_transaction(
        &mut self,
        from: Address,
        to: Address,
        value: u64,
    ) -> Result<(), &'static str> {
        // Get sender account
        let sender = self
            .accounts
            .get_mut(&from)
            .ok_or("Sender account doesn't exist")?;

        // Check balance
        if sender.balance < value {
            return Err("Insufficient balance");
        }

        // Deduct from sender
        sender.balance -= value;
        sender.nonce += 1; // Increment nonce for each transaction

        // Credit recipient (create account if it doesn't exist)
        let recipient = self.accounts.entry(to).or_insert(Account {
            nonce: 0,
            balance: 0,
            code_hash: B256::ZERO, // Empty account has zero code hash
            storage_root: B256::ZERO,
        });
        recipient.balance += value;

        Ok(())
    }
}
