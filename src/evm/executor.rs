use crate::block::account::Account;
use crate::block::state::State;
use crate::evm::evm::{ExecutionContext, VMError, VM};
use crate::transaction::errors::TransactionError;
use crate::transaction::transaction::{Transaction, TRANSACTION_GAS_COST};
use alloy_primitives::B256;
use std::sync::{Arc, Mutex};

pub struct Executor;

impl Executor {
    pub fn process_transaction_contract(
        transaction: Transaction,
        state: Arc<Mutex<State>>,
    ) -> Result<(), VMError> {
        // TODO: unnecessarily wasteful clone on large list
        let contract = state
            .lock()
            .unwrap()
            .contract
            .get(&transaction.to)
            .ok_or(VMError::ContractNotFound)?
            .clone();

        let mut evm = VM::new(
            contract.clone(),
            ExecutionContext::new(
                transaction.get_sender_address().unwrap(),
                transaction.to,
                transaction.value,
                transaction.input_data.clone(),
                transaction.gas_limit,
            ),
            state,
        );
        evm.execute_transaction(transaction)?;
        Ok(())
    }

    pub fn process_transaction(
        transaction: &Transaction,
        base_fee: u64,
        state: Arc<Mutex<State>>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut state = state.lock().unwrap();

        // Get sender account
        let sender = state
            .accounts
            .get_mut(
                &transaction
                    .get_sender_address()
                    .ok_or(Box::new(TransactionError::InvalidTransaction))?,
            )
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

        if !transaction.verify_signature() {
            return Err(Box::new(TransactionError::InvalidSignature));
        }

        if sender.balance < transaction.value + total_fee {
            return Err(Box::new(TransactionError::InsufficientBalance));
        }

        sender.balance -= transaction.value + total_fee;
        sender.nonce += 1;

        let recipient = state.accounts.entry(transaction.to).or_insert(Account {
            nonce: 0,
            balance: 0,
            code_hash: B256::ZERO,
            storage_root: B256::ZERO,
        });

        recipient.balance += transaction.value;

        Ok(())
    }
}
