use alloy_primitives::{Address, B256};
use std::sync::{Arc, Mutex};

use crate::block::block::Block;
use crate::block::state::State;
use crate::crypto::wallet::EthereumWallet;
use crate::crypto::wallet::Wallet;

pub trait Blockchain {
    fn run(&mut self);
    fn get_next_block(&self) -> Block;
}

pub struct App {
    state: Arc<Mutex<State>>,
    account: EthereumWallet,
    running: bool,
    blocks: Vec<Block>,
    slot: u64,
}

impl App {
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(State::new())),
            account: EthereumWallet::generate(),
            running: true,
            blocks: vec![],
            slot: 0,
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

    fn get_next_block(&self) -> Block {
        let proposer_index = 0;
        let parent_root = B256::ZERO;
        let state_root = B256::ZERO;
        Block::new(self.slot, proposer_index, parent_root, state_root)
    }
}
