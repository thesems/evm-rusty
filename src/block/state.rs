use std::collections::HashMap;

use crate::block::account::Account;
use crate::evm::evm::{Contract, ExecutionContext, VMError, VM};
use crate::transaction::errors::TransactionError;
use crate::transaction::transaction::Transaction;
use crate::transaction::transaction::TRANSACTION_GAS_COST;
use alloy_primitives::{Address, B256};

pub struct State {
    pub accounts: HashMap<Address, Account>,
    pub storage: HashMap<(Address, B256), B256>,
    pub contract: HashMap<Address, Contract>,
}

impl Default for State {
    fn default() -> Self {
        Self::new()
    }
}

impl State {
    pub fn new() -> Self {
        Self {
            accounts: HashMap::new(),
            storage: HashMap::new(),
            contract: Default::default(),
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
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};
    use super::*;
    use crate::block::account::Account;
    use crate::crypto::wallet::Wallet;
    use crate::transaction::transaction::{Transaction, ETH_TO_WEI, TRANSACTION_GAS_COST};
    use crate::evm::executor::Executor;

    #[test]
    fn test_transaction_basic() {
        let eth_wallet_sender = Wallet::generate();
        let eth_wallet_receiver = Wallet::generate();

        let mut state_arc = Arc::new(Mutex::new(State::new()));
        let mut state = state_arc.lock().unwrap();

        {
            assert!(state.get_account(&eth_wallet_receiver.address).is_none());
            assert!(state.get_account(&eth_wallet_sender.address).is_none());
        }
        {
            state.set_account(eth_wallet_sender.address, Account::default());
            let sender = state.get_account(&eth_wallet_sender.address).unwrap();

            // transaction cost + base fee + priority fee
            sender.balance = 3 * ETH_TO_WEI;
        }

        let tx = Transaction::new(
            eth_wallet_receiver.address,
            ETH_TO_WEI,
            TRANSACTION_GAS_COST,
            2_000_000_000,  // 2 Gwei max tip
            12_000_000_000, // 12 Gwei max total (base + priority)
            vec![],
            Some(&eth_wallet_sender.private_key)
        );

        let base_fee = 10;
        Executor::process_transaction(&tx, base_fee, state_arc.clone()).unwrap();

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
