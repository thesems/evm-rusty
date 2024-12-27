use std::collections::HashMap;

use crate::block::account::Account;
use crate::transaction::errors::TransactionError;
use crate::transaction::transaction::Transaction;
use crate::transaction::transaction::TRANSACTION_GAS_COST;
use alloy_primitives::{Address, B256};

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

        // Verify signature
        if !transaction.verify_signature() {
            return Err(Box::new(TransactionError::InvalidSignature));
        }

        // Check balance
        if sender.balance < transaction.value + total_fee {
            return Err(Box::new(TransactionError::InsufficientBalance));
        }

        // Deduct from snder
        sender.balance -= transaction.value + total_fee;
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::block::account::Account;
    use crate::crypto::wallet::{EthereumWallet, Wallet};
    use crate::transaction::transaction::{Transaction, ETH_TO_WEI, TRANSACTION_GAS_COST};
    use k256::{
        ecdsa::{signature::Signer, Signature, SigningKey},
        SecretKey,
    };

    #[test]
    fn test_transaction_basic() {
        let eth_wallet_sender = EthereumWallet::generate();
        let eth_wallet_receiver = EthereumWallet::generate();

        let mut state = State::new();

        {
            assert!(state.get_account(&eth_wallet_receiver.address).is_none());
            assert!(state.get_account(&eth_wallet_sender.address).is_none());
        }
        {
            state.set_account(eth_wallet_sender.address.clone(), Account::new());
            let mut sender = state.get_account(&eth_wallet_sender.address).unwrap();

            // transaction cost + base fee + priority fee
            sender.balance = 3 * ETH_TO_WEI;
        }

        let mut tx = Transaction::new(
            eth_wallet_sender.address.clone(),
            eth_wallet_receiver.address.clone(),
            ETH_TO_WEI,
            TRANSACTION_GAS_COST,
            2_000_000_000,  // 2 Gwei max tip
            12_000_000_000, // 12 Gwei max total (base + priority)
        );

        tx.sign(&eth_wallet_sender.private_key);

        let base_fee = 10;
        state.process_transaction(&tx, base_fee).unwrap();

        let sender_balance = state
            .get_account(&eth_wallet_sender.address)
            .unwrap()
            .balance;
        let recv_balance = state
            .get_account(&eth_wallet_receiver.address)
            .unwrap()
            .balance;
        let sender_nonce = state.get_account(&eth_wallet_sender.address).unwrap().nonce;

        assert_eq!(sender_nonce, 1);
        assert_eq!(
            sender_balance,
            2 * ETH_TO_WEI - (TRANSACTION_GAS_COST * (base_fee + 2_000_000_000))
        );
        assert_eq!(recv_balance, ETH_TO_WEI);
    }
}
