use alloy_primitives::{Address, B256};

use crate::transaction::transaction::Transaction;

struct AttestationData {
    slot: u64,
    index: u64,
    beacon_block_root: B256,
    // the last justified checkpoint
    source: u64,
    // the latest epoch boundary block
    target: u64,
}

struct Attestation {
    aggregation_bits: Vec<u64>,
    data: AttestationData,
    signature: B256,
}

pub struct Withdrawal {
    address: Address,
    amount: u64,
    index: u64,
    validator_index: u64,
}

struct ExecutionPayload {
    parent_hash: B256,
    fee_recipient: Address,
    state_root: B256,
    receipts_root: B256,
    logs_bloom: Vec<String>,
    prev_randao: u64,
    block_number: u64,
    gas_limit: u64,
    gas_used: u64,
    timestamp: u64,
    extra_data: Vec<u8>,
    base_fee_per_gas: u64,
    block_hash: B256,
    transactions: Vec<Transaction>,
    withdrawals: Vec<Withdrawal>,
}

struct BlockBody {
    randao_reveal: u64,
    eth1_data: B256,
    graffiti: B256,
    proposer_slashings: Vec<u64>,
    attester_slashings: Vec<u64>,
    attestations: Vec<Attestation>,
    deposits: Vec<u64>,
    voluntary_exits: Vec<u64>,
    sync_aggregate: Vec<u64>,
    execution_payload: ExecutionPayload,
}

pub struct Block {
    // bounded-size, targets 15 million gas, but can grow more/less depending on demand
    // hard limit on 2x target size (30 million gas)
    slot: u64,
    proposer_index: u64,
    parent_root: B256,
    state_root: B256,
    body: BlockBody,
}

impl Block {
    /// Creates a new `Block` with the given parameters.
    pub fn new(
        slot: u64,
        proposer_index: u64,
        parent_root: B256,
        state_root: B256,
    ) -> Self {
        Block {
            slot,
            proposer_index,
            parent_root,
            state_root,
            body: BlockBody::default(),
        }
    }

    /// Adds a transaction to the block's execution payload.
    pub fn add_transaction(&mut self, transaction: Transaction) {
        self.body.execution_payload.transactions.push(transaction);
    }

    /// Adds a withdrawal to the block's execution payload.
    pub fn add_withdrawal(&mut self, withdrawal: Withdrawal) {
        self.body.execution_payload.withdrawals.push(withdrawal);
    }
}

impl BlockBody {
    /// Provides a default `BlockBody` with empty fields.
    pub fn default() -> Self {
        BlockBody {
            randao_reveal: 0,
            eth1_data: B256::default(),
            graffiti: B256::default(),
            proposer_slashings: Vec::new(),
            attester_slashings: Vec::new(),
            attestations: Vec::new(),
            deposits: Vec::new(),
            voluntary_exits: Vec::new(),
            sync_aggregate: Vec::new(),
            execution_payload: ExecutionPayload::default(),
        }
    }
}

impl ExecutionPayload {
    /// Provides a default `ExecutionPayload` with empty fields.
    pub fn default() -> Self {
        ExecutionPayload {
            parent_hash: B256::default(),
            fee_recipient: Address::default(),
            state_root: B256::default(),
            receipts_root: B256::default(),
            logs_bloom: Vec::new(),
            prev_randao: 0,
            block_number: 0,
            gas_limit: 0,
            gas_used: 0,
            timestamp: 0,
            extra_data: Vec::new(),
            base_fee_per_gas: 0,
            block_hash: B256::default(),
            transactions: Vec::new(),
            withdrawals: Vec::new(),
        }
    }
}
