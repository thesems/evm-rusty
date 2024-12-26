use alloy_primitives::{Address, B256};
use std::sync::{Arc, Mutex};

use crate::block::block::Block;
use crate::block::state::State;
use crate::crypto::wallet::EthereumWallet;
use crate::crypto::wallet::Wallet;
use crate::transaction::transaction::Transaction;

pub trait Blockchain {
    fn run(&mut self);
    fn execute_transactions(&mut self);
    fn get_next_block(&self) -> Block;
}

pub struct App {
    state: Arc<Mutex<State>>,
    tx_send: std::sync::mpsc::Sender<Transaction>,
    tx_recv: std::sync::mpsc::Receiver<Transaction>,
    account: EthereumWallet,
    running: bool,
    blocks: Vec<Block>,
    slot: u64,
    base_fee: u64,
}

impl App {
    pub fn new() -> Self {
        let (tx_send, tx_recv) = std::sync::mpsc::channel();

        Self {
            state: Arc::new(Mutex::new(State::new())),
            tx_send,
            tx_recv,
            account: EthereumWallet::generate(),
            running: true,
            blocks: vec![],
            slot: 0,
            base_fee: 10,
        }
    }
}

impl Blockchain for App {
    fn run(&mut self) {
        use std::thread;
        use std::time::{Duration, Instant};

        let target_block_time = Duration::from_secs(12);

        while self.running {
            let start_time = Instant::now();

            self.execute_transactions();

            // Generate and push the next block
            let new_block = self.get_next_block();
            self.blocks.push(new_block);
            self.slot += 1;
            log::info!("Block {} generated.", self.slot);

            // Sleep to maintain the target block time
            let elapsed_time = start_time.elapsed();
            if elapsed_time < target_block_time {
                thread::sleep(target_block_time - elapsed_time);
            }
        }
    }

    fn execute_transactions(&mut self) {
        if let Ok(tx) = self.tx_recv.try_recv() {
            if self
                .state
                .lock()
                .unwrap()
                .process_transaction(&tx, self.base_fee)
                .is_err()
            {
                log::error!("Transaction failed.");
            };
        }
    }

    fn get_next_block(&self) -> Block {
        let proposer_index = 0;
        let parent_root = B256::ZERO;
        let state_root = B256::ZERO;
        Block::new(self.slot, proposer_index, parent_root, state_root)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::block::account::Account;
    use crate::transaction::transaction::{Transaction, ETH_TO_WEI, TRANSACTION_GAS_COST};

    #[test]
    fn test_transaction_basic() {
        let eth_wallet_sender = EthereumWallet::generate();
        let eth_wallet_receiver = EthereumWallet::generate();

        let mut app = App::new();

        {
            let mut state = app.state.lock().unwrap();
            assert!(state
                .get_account(eth_wallet_receiver.get_address())
                .is_none());
            assert!(state.get_account(eth_wallet_sender.get_address()).is_none());
        }
        {
            let mut state = app.state.lock().unwrap();
            state.set_account(eth_wallet_sender.get_address().clone(), Account::new());
            let mut sender = state.get_account(eth_wallet_sender.get_address()).unwrap();

            // transaction cost + base fee + priority fee
            sender.balance = 3 * ETH_TO_WEI;
        }

        let tx = Transaction::new(
            eth_wallet_sender.get_address().clone(),
            eth_wallet_receiver.get_address().clone(),
            ETH_TO_WEI,
            TRANSACTION_GAS_COST,
            2_000_000_000,  // 2 Gwei max tip
            12_000_000_000, // 12 Gwei max total (base + priority)
        );
        app.tx_send.send(tx).unwrap();
        app.execute_transactions();

        let mut state = app.state.lock().unwrap();
        let sender_balance = state
            .get_account(eth_wallet_sender.get_address())
            .unwrap()
            .balance;
        let recv_balance = state
            .get_account(eth_wallet_receiver.get_address())
            .unwrap()
            .balance;
        let sender_nonce = state
            .get_account(eth_wallet_sender.get_address())
            .unwrap()
            .nonce;

        assert_eq!(sender_nonce, 1);
        assert_eq!(
            sender_balance,
            2 * ETH_TO_WEI - (TRANSACTION_GAS_COST * (app.base_fee + 2_000_000_000))
        );
        assert_eq!(recv_balance, ETH_TO_WEI);
    }
}
