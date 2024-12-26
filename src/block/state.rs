use std::collections::HashMap;

use alloy_primitives::{Address, B256};

use crate::block::account::Account;
use crate::transaction::errors::TransactionError;
use crate::transaction::transaction::Transaction;
use crate::transaction::transaction::TRANSACTION_GAS_COST;

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
        transaction: &Transaction,
        base_fee: u64,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Get sender account
        let sender = self
            .accounts
            .get_mut(&transaction.from)
            .ok_or(Box::new(TransactionError::SenderAccountDoesNotExist))?;

        if base_fee > transaction.max_fee_per_gas {
            return Err(Box::new(TransactionError::MaximumGasFeeBelowBaseFee));
        }

        let total_fee = TRANSACTION_GAS_COST
            * transaction
                .max_fee_per_gas
                .min(base_fee + transaction.max_priority_fee_per_gas);

        if transaction.gas_limit < TRANSACTION_GAS_COST {
            return Err(Box::new(TransactionError::InsufficientGas));
        }

        // Check balance
        if sender.balance < transaction.value + total_fee {
            return Err(Box::new(TransactionError::InsufficientBalance));
        }

        // Deduct from sender
        sender.balance -= (transaction.value + total_fee);
        sender.nonce += 1;

        // Credit recipient (create account if it doesn't exist)
        let recipient = self.accounts.entry(transaction.to).or_insert(Account {
            nonce: 0,
            balance: 0,
            code_hash: B256::ZERO,
            storage_root: B256::ZERO,
        });

        // Credit receiver
        recipient.balance += transaction.value;

        Ok(())
    }
}
