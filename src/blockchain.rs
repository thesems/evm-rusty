use alloy_primitives::B256;
use std::sync::{Arc, Mutex};

use crate::block::block::Block;
use crate::block::state::State;
use crate::crypto::wallet::Wallet;
use crate::evm::executor::Executor;
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
    account: Wallet,
    running: bool,
    blocks: Vec<Block>,
    slot: u64,
    base_fee: u64,
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

impl App {
    pub fn new() -> Self {
        let (tx_send, tx_recv) = std::sync::mpsc::channel();

        Self {
            state: Arc::new(Mutex::new(State::new())),
            tx_send,
            tx_recv,
            account: Wallet::generate(),
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
            if Executor::process_transaction(&tx, self.base_fee, self.state.clone()).is_err() {
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
