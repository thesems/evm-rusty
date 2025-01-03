use crate::block::account::Account;
use crate::block::state::State;
use crate::evm::bytecode_parser::{BytecodeParser, ParserError};
use crate::evm::evm::VMError::{NoItemsOnStack, NotEnoughItemsOnStack, StackFull};
use crate::evm::operation::Operation;
use crate::transaction::transaction::Transaction;
use alloy_rlp::{Encodable, RlpDecodable, RlpEncodable};

use crate::crypto::hash::hash_slice_to_b256;
use alloy_primitives::{keccak256, Address, B256, U256};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

const MAX_STACK_SIZE: u32 = 1024;

pub enum ExecutionResult {
    Success {
        return_data: Option<Vec<u8>>,
        gas_used: u64,
    },
    Revert {
        reason: Vec<u8>,
        gas_used: u64,
    },
}

#[derive(Debug, RlpEncodable, RlpDecodable, PartialEq)]
pub struct AddressNonce {
    pub address: Vec<u8>,
    pub nonce: u64,
}

#[derive(Debug)]
pub enum VMError {
    StackFull,
    NotEnoughItemsOnStack(String),
    NoItemsOnStack,
    NotImplemented,
    ContractNotFound,
    InvalidTransaction,
    InvalidBytecode,
    OutOfGas,
    StackUnderflow,
    NoOperationExecuted,
}
impl From<ParserError> for VMError {
    fn from(value: ParserError) -> Self {
        match value {
            ParserError::IncompletePush => VMError::InvalidBytecode,
            _ => VMError::InvalidTransaction,
        }
    }
}

#[derive(Clone)]
pub struct Contract {
    pub code: Vec<u8>,
    pub storage: HashMap<U256, U256>,
}

impl Contract {
    pub fn new(code: Vec<u8>) -> Self {
        Self {
            code,
            storage: HashMap::new(),
        }
    }
}

pub struct ExecutionContext {
    caller: Address,
    address: Address,
    value: u64,
    data: Vec<u8>,
    gas: u64,
}

impl ExecutionContext {
    pub fn new(caller: Address, address: Address, value: u64, data: Vec<u8>, gas: u64) -> Self {
        Self {
            caller,
            address,
            value,
            data,
            gas,
        }
    }
}

enum StorageChangeType {
    Set,
    Delete,
}

pub struct VM {
    stack: Vec<U256>,
    memory: Vec<u8>,
    contract: Contract,
    gas_available: u64,
    context: ExecutionContext,
    creation_offset: usize,
    pc: usize,
    state: Arc<Mutex<State>>,
    storage_revert: HashMap<U256, (StorageChangeType, U256)>,
}

impl VM {
    pub fn new(contract: Contract, context: ExecutionContext, state: Arc<Mutex<State>>) -> Self {
        // Find the runtime code start (look for 0xf3 0xfe sequence)
        let creation_offset = contract
            .code
            .windows(2)
            .position(|window| window == [0xf3, 0xfe])
            .map(|pos| pos + 2) // Skip past the f3 fe
            .unwrap_or(0); // If not found, assume it's all runtime code

        Self {
            stack: Vec::new(),
            memory: vec![],
            contract,
            gas_available: context.gas,
            context,
            creation_offset,
            pc: 0,
            state,
            storage_revert: HashMap::new(),
        }
    }

    fn load_into_memory(&mut self, offset: usize, value: U256) -> Result<(), VMError> {
        let bytes = value.to_be_bytes::<32>();
        self.expand_memory(offset, 32)?;
        self.memory[offset..offset + 32].copy_from_slice(&bytes);
        Ok(())
    }

    fn expand_memory(&mut self, offset: usize, required_size: usize) -> Result<(), VMError> {
        let new_size = offset + required_size;
        if self.memory.len() < new_size {
            if Self::calc_memory_expansion_gas(offset + 32) < self.gas_available {
                self.memory.resize(new_size, 0);
            } else {
                return Err(VMError::OutOfGas);
            }
        }
        Ok(())
    }

    fn calc_memory_expansion_gas(memory_byte_size: usize) -> u64 {
        /// Calculates the gas cost for expanding the memory to the given size.
        ///
        /// # Arguments
        ///
        /// * `memory_byte_size` - The size in bytes of the memory to expand to.
        ///
        /// # Returns
        ///
        /// The calculated gas cost for the memory expansion.
        ///
        /// The gas cost is calculated based on the EVM formula:
        /// - The word size is the memory size rounded up to the nearest multiple of 32.
        /// - The memory cost combines a quadratic term and a linear term:
        ///   - Quadratic term: `(memory_size_word^2) / 512`
        ///   - Linear term: `3 * memory_size_word`
        ///
        let memory_size_word = (memory_byte_size + 31) / 32;
        let memory_cost = (memory_size_word * memory_size_word / 512) + (3 * memory_size_word);
        memory_cost as u64
    }

    fn read_from_memory(&mut self, offset: usize, length: usize) -> &[u8] {
        if self.memory.len() < offset + length {
            self.memory.resize(offset + length, 0);
        }
        &self.memory[offset..offset + length]
    }

    fn revert_storage(&mut self) {
        // Revert all changes made to the storage by replacing current values
        // with the appropriate actions from the storage_revert map.
        for (key, (change_type, old_value)) in &self.storage_revert {
            match change_type {
                StorageChangeType::Set => {
                    if let Some(storage) = self.contract.storage.get_mut(key) {
                        *storage = *old_value;
                    }
                }
                StorageChangeType::Delete => {
                    self.contract.storage.remove(key);
                }
            }
        }
        // Clear the storage_revert map after reverting changes.
        self.storage_revert.clear();
    }

    pub fn execute_operations(&mut self, code: Vec<u8>) -> Result<ExecutionResult, VMError> {
        let mut parser = BytecodeParser::new(code);
        let operations = parser.compile().map_err(|e| VMError::InvalidBytecode)?;

        let mut execution_result: Option<ExecutionResult> = None;
        while self.pc < operations.len() {
            execution_result = Some(self.process_operation(&operations[self.pc])?);
            self.pc += 1;
        }
        execution_result.ok_or(VMError::NoOperationExecuted)
    }

    pub fn execute_transaction(
        &mut self,
        transaction: Transaction,
    ) -> Result<ExecutionResult, VMError> {
        // differentiate contract creation
        if transaction.to.is_zero() {
            self.call_contract_create(transaction)
        } else {
            self.call_contract(transaction)
        }
    }

    fn generate_contract_address(&self, address: Address, nonce: u64) -> Address {
        let mut buffer = Vec::<u8>::new();
        AddressNonce {
            address: address.0.as_slice().to_vec(),
            nonce,
        }
        .encode(&mut buffer);
        let hash = keccak256(&buffer);
        Address::from_slice(&hash[12..])
    }

    pub fn call_contract_create(
        &mut self,
        transaction: Transaction,
    ) -> Result<ExecutionResult, VMError> {
        let sender = transaction
            .get_sender_address()
            .ok_or(VMError::InvalidTransaction)?;

        let contract_address = self.generate_contract_address(sender, transaction.nonce);
        self.context.address = contract_address;

        self.state.lock().unwrap().accounts.insert(
            contract_address,
            Account::new(
                transaction.value,
                hash_slice_to_b256(transaction.input_data.as_slice()),
                B256::ZERO, // TODO: storage root hash?
            ),
        );

        self.execute_operations(transaction.input_data.clone())
    }

    pub fn call_contract(&mut self, transaction: Transaction) -> Result<ExecutionResult, VMError> {
        // extract function selector
        let selector = &transaction.input_data[0..4];

        self.pc = 0;
        self.stack.clear();
        self.memory.clear();

        Ok(ExecutionResult::Success {
            return_data: None,
            gas_used: 0,
        })
    }

    fn stack_size(&self) -> u32 {
        self.stack.len() as u32
    }

    fn push(&mut self, value: U256) -> Result<(), VMError> {
        if self.stack_size() >= MAX_STACK_SIZE {
            return Err(StackFull);
        }
        self.stack.push(value);
        Ok(())
    }

    fn pop(&mut self) -> Result<U256, VMError> {
        self.stack.pop().ok_or(NoItemsOnStack)
    }

    fn add(&mut self) -> Result<(), VMError> {
        let a = self.pop()?;
        let b = self.pop()?;
        self.push(a + b)
    }

    fn process_operation(&mut self, operation: &Operation) -> Result<ExecutionResult, VMError> {
        let stack_req = operation.stack_req();
        let operation_name = format!("{:?}", operation);

        if self.stack_size() < stack_req.min_stack_height {
            return Err(NotEnoughItemsOnStack(operation_name));
        }

        let gas_cost = operation.gas_cost();
        if self.gas_available < gas_cost.base {
            return Err(VMError::OutOfGas);
        }

        let not_impl_error = format!("Operation {:?} is not implemented", operation_name);

        match operation {
            Operation::Stop => panic!("{}", not_impl_error),
            Operation::Add => {
                self.add()?;
            }
            Operation::Mul => panic!("{}", not_impl_error),
            Operation::Sub => panic!("{}", not_impl_error),
            Operation::Div => panic!("{}", not_impl_error),
            Operation::SDiv => panic!("{}", not_impl_error),
            Operation::Mod => panic!("{}", not_impl_error),
            Operation::SMod => panic!("{}", not_impl_error),
            Operation::AddMod => panic!("{}", not_impl_error),
            Operation::MulMod => panic!("{}", not_impl_error),
            Operation::Exp => panic!("{}", not_impl_error),
            Operation::SignExtend => panic!("{}", not_impl_error),
            Operation::Lt => panic!("{}", not_impl_error),
            Operation::Gt => panic!("{}", not_impl_error),
            Operation::Slt => panic!("{}", not_impl_error),
            Operation::Sgt => panic!("{}", not_impl_error),
            Operation::Eq => panic!("{}", not_impl_error),
            Operation::IsZero => {
                let item = self.pop()?;
                self.push(U256::from(item.is_zero()))?;
            }
            Operation::And => panic!("{}", not_impl_error),
            Operation::Or => panic!("{}", not_impl_error),
            Operation::Xor => panic!("{}", not_impl_error),
            Operation::Not => panic!("{}", not_impl_error),
            Operation::Byte => panic!("{}", not_impl_error),
            Operation::Shl => panic!("{}", not_impl_error),
            Operation::Shr => panic!("{}", not_impl_error),
            Operation::Sar => panic!("{}", not_impl_error),
            Operation::Address => {
                self.push(U256::from_be_slice(self.context.address.as_slice()))?;
            }
            Operation::Balance => panic!("{}", not_impl_error),
            Operation::Origin => {
                self.push(U256::from_be_slice(self.context.caller.as_slice()))?;
            }
            Operation::Caller => panic!("{}", not_impl_error),
            Operation::CallValue => {
                self.push(U256::from(self.context.value))?;
            }
            Operation::CallDataLoad => {
                let i = self.pop()?.to::<usize>();
                let mut result = [0u8; 32];

                if i < self.context.data.len() {
                    let slice_end: usize = (i + 32).min(self.context.data.len());
                    result.copy_from_slice(&self.context.data[i..slice_end]);
                }

                self.push(U256::from_be_slice(&result))?;
            }
            Operation::CallDataSize => {
                self.push(U256::from(self.context.data.len()))?;
            }
            Operation::CallDataCopy => panic!("{}", not_impl_error),
            Operation::CodeSize => {
                self.push(U256::from(self.contract.code.len()))?;
            }
            Operation::CodeCopy => {
                let dest_offset = self.pop()?.to::<usize>();
                let offset = self.pop()?.to::<usize>();
                let size = self.pop()?.to::<usize>();

                let minimum_word_size = (size as u64 + 31) / 32;
                let static_gas = 3;
                let dynamic_gas =
                    3 * minimum_word_size + Self::calc_memory_expansion_gas(size);

                if self.gas_available < static_gas + dynamic_gas {
                    return Err(VMError::OutOfGas);
                }

                self.gas_available -= static_gas + dynamic_gas;

                self.expand_memory(dest_offset, size)?;

                // Get the raw bytecode slice
                for i in 0..size {
                    let byte = if offset + i < self.contract.code.len() {
                        self.contract.code[offset + i] // Copy raw byte directly
                    } else {
                        0 // For out-of-bound bytes, pad with 0
                    };
                    self.memory[dest_offset + i] = byte;
                }
            }
            Operation::GasPrice => panic!("{}", not_impl_error),
            Operation::ExtCodeSize => panic!("{}", not_impl_error),
            Operation::ExtCodeCopy => panic!("{}", not_impl_error),
            Operation::ReturnDataSize => panic!("{}", not_impl_error),
            Operation::ReturnDataCopy => panic!("{}", not_impl_error),
            Operation::ExtCodeHash => panic!("{}", not_impl_error),
            Operation::BlockHash => panic!("{}", not_impl_error),
            Operation::Coinbase => panic!("{}", not_impl_error),
            Operation::Timestamp => panic!("{}", not_impl_error),
            Operation::Number => panic!("{}", not_impl_error),
            Operation::Difficulty => panic!("{}", not_impl_error),
            Operation::GasLimit => panic!("{}", not_impl_error),
            Operation::ChainId => panic!("{}", not_impl_error),
            Operation::SelfBalance => panic!("{}", not_impl_error),
            Operation::BaseFee => panic!("{}", not_impl_error),
            Operation::Pop => {
                self.pop()?; // Simply discard the value at the top of the stack
            }
            Operation::MLoad => panic!("{}", not_impl_error),
            Operation::MStore => {
                let offset = self.pop()?.to::<usize>();
                let value = self.pop()?;
                self.load_into_memory(offset, value)?;
            }
            Operation::MStore8 => panic!("{}", not_impl_error),
            Operation::SLoad => {
                let key = self.pop()?; // Get the storage key from the stack
                let value = self
                    .contract
                    .storage
                    .get(&key)
                    .cloned()
                    .unwrap_or(U256::ZERO);
                self.push(value)?;
            }
            Operation::SStore => {
                let storage_key = self.pop()?;
                let storage_value = self.pop()?;

                let prev_value = self.contract.storage.insert(storage_key, storage_value);

                if prev_value.is_none() {
                    self.storage_revert
                        .insert(storage_key, (StorageChangeType::Delete, storage_value));
                } else {
                    self.storage_revert
                        .insert(storage_key, (StorageChangeType::Set, prev_value.unwrap()));
                }
            }
            Operation::Jump => panic!("{}", not_impl_error),
            Operation::JumpI => {
                let offset = self.pop()?.to::<usize>();
                let jump = self.pop()?;

                if !jump.is_zero() {
                    // -1 since it will get incremented by 1
                    self.pc = offset - 1;
                }
            }
            Operation::PC => panic!("{}", not_impl_error),
            Operation::MSize => panic!("{}", not_impl_error),
            Operation::Gas => panic!("{}", not_impl_error),
            Operation::JumpDest => {
                // JUMPDEST is a marker for valid jump destinations. It has no effect
                // on the machine state, so we simply proceed to the next instruction.
                // No changes are made to the stack, memory, or storage.
            }
            Operation::Push0 => {
                self.push(U256::ZERO)?;
            }
            Operation::Push1(value)
            | Operation::Push2(value)
            | Operation::Push3(value)
            | Operation::Push4(value)
            | Operation::Push5(value)
            | Operation::Push6(value)
            | Operation::Push7(value)
            | Operation::Push8(value)
            | Operation::Push9(value)
            | Operation::Push10(value)
            | Operation::Push11(value)
            | Operation::Push12(value)
            | Operation::Push13(value)
            | Operation::Push14(value)
            | Operation::Push15(value)
            | Operation::Push16(value)
            | Operation::Push17(value)
            | Operation::Push18(value)
            | Operation::Push19(value)
            | Operation::Push20(value)
            | Operation::Push21(value)
            | Operation::Push22(value)
            | Operation::Push23(value)
            | Operation::Push24(value)
            | Operation::Push25(value)
            | Operation::Push26(value)
            | Operation::Push27(value)
            | Operation::Push28(value)
            | Operation::Push29(value)
            | Operation::Push30(value)
            | Operation::Push31(value)
            | Operation::Push32(value) => {
                self.push(*value)?;
            }
            Operation::Dup(item_num) => {
                let item_num = *item_num as usize;
                if item_num == 0 || item_num > self.stack.len() {
                    return Err(VMError::StackUnderflow);
                }
                let item_to_duplicate = self.stack[self.stack.len() - item_num].clone();
                self.stack.push(item_to_duplicate);
            }
            Operation::Swap1 => panic!("{}", not_impl_error),
            Operation::Swap2 => panic!("{}", not_impl_error),
            Operation::Swap3 => panic!("{}", not_impl_error),
            Operation::Swap4 => panic!("{}", not_impl_error),
            Operation::Swap5 => panic!("{}", not_impl_error),
            Operation::Swap6 => panic!("{}", not_impl_error),
            Operation::Swap7 => panic!("{}", not_impl_error),
            Operation::Swap8 => panic!("{}", not_impl_error),
            Operation::Swap9 => panic!("{}", not_impl_error),
            Operation::Swap10 => panic!("{}", not_impl_error),
            Operation::Swap11 => panic!("{}", not_impl_error),
            Operation::Swap12 => panic!("{}", not_impl_error),
            Operation::Swap13 => panic!("{}", not_impl_error),
            Operation::Swap14 => panic!("{}", not_impl_error),
            Operation::Swap15 => panic!("{}", not_impl_error),
            Operation::Swap16 => panic!("{}", not_impl_error),
            Operation::Log0 => panic!("{}", not_impl_error),
            Operation::Log1 => panic!("{}", not_impl_error),
            Operation::Log2 => panic!("{}", not_impl_error),
            Operation::Log3 => panic!("{}", not_impl_error),
            Operation::Log4 => panic!("{}", not_impl_error),
            Operation::Create => panic!("{}", not_impl_error),
            Operation::Call => panic!("{}", not_impl_error),
            Operation::CallCode => panic!("{}", not_impl_error),
            Operation::Return => {
                let size = self.pop()?.to::<usize>();
                let offset = self.pop()?.to::<usize>();

                let return_data = self.read_from_memory(offset, size);

                return Ok(ExecutionResult::Success {
                    return_data: Some(return_data.to_vec()),
                    gas_used: gas_cost.base,
                });
            }
            Operation::DelegateCall => panic!("{}", not_impl_error),
            Operation::Create2 => panic!("{}", not_impl_error),
            Operation::StaticCall => panic!("{}", not_impl_error),
            Operation::Revert => {
                let length = self.pop()?.to::<usize>();
                let offset = self.pop()?.to::<usize>();

                self.revert_storage();
                let revert_data = self.read_from_memory(offset, length);

                // Return the revert result
                return Ok(ExecutionResult::Revert {
                    reason: revert_data.to_vec(),
                    gas_used: gas_cost.base,
                });
            }
            Operation::Invalid => panic!("{}", not_impl_error),
            Operation::SelfDestruct => panic!("{}", not_impl_error),
            _ => panic!("Unknown operation: {:?}", operation),
        }

        Ok(ExecutionResult::Success {
            return_data: None,
            gas_used: gas_cost.base,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::hash::hash_string_to_u256;
    use crate::crypto::wallet::Wallet;
    use crate::evm::bytecode_parser::BytecodeParser;
    use crate::transaction::transaction::{ETH_TO_WEI, GWEI_TO_WEI};
    use alloy_primitives::hex::FromHex;

    #[test]
    fn test_add_operation() {
        let code = vec![
            Operation::Push1(U256::from(1)).opcode(),
            Operation::Push1(U256::from(1)).opcode(),
            Operation::Add.opcode(),
        ];

        let mut vm = VM::new(
            Contract::new(code.clone()),
            ExecutionContext::new(
                Address::from_hex("0x169EE3A023A8D9fF2E0D94cf8220b1Ba40D59794").unwrap(),
                Address::from_hex("0x169EE3A023A8D9fF2E0D94cf8220b1Ba40D59794").unwrap(),
                0,
                vec![],
                0,
            ),
            Arc::new(Mutex::new(State::new())),
        );
        vm.execute_operations(code).unwrap();
        assert_eq!(*vm.stack.last().unwrap(), U256::from(2));
    }

    #[test]
    fn test_contract_basics() {
        let mut parser = BytecodeParser::from("./test/Counter.evm").unwrap();

        let value = 1 * ETH_TO_WEI;
        let gas = 100 * GWEI_TO_WEI;

        let mut vm = VM::new(
            Contract::new(parser.bytecode.clone()),
            ExecutionContext::new(
                Address::from_hex("0x169EE3A023A8D9fF2E0D94cf8220b1Ba40D59794").unwrap(),
                Address::from_hex("0x169EE3A023A8D9fF2E0D94cf8220b1Ba40D59794").unwrap(),
                value,
                vec![],
                gas,
            ),
            Arc::new(Mutex::new(State::new())),
        );

        let eth_wallet = Wallet::generate();

        let tx_create = Transaction::new(
            Address::ZERO,
            value,
            gas,
            100,
            100,
            parser.bytecode,
            Some(&eth_wallet.private_key),
        );

        vm.execute_transaction(tx_create).unwrap();

        assert_eq!(
            *vm.contract
                .storage
                .get(&hash_string_to_u256("counter"))
                .unwrap(),
            U256::from(0)
        );

        let tx_inc = Transaction::new(
            eth_wallet.address,
            100,
            100,
            100,
            100,
            hash_string_to_u256("inc()").to_be_bytes::<32>()[..4].to_vec(),
            Some(&eth_wallet.private_key),
        );

        vm.execute_transaction(tx_inc).unwrap();

        assert_eq!(
            *vm.contract
                .storage
                .get(&hash_string_to_u256("counter"))
                .unwrap(),
            U256::from(1)
        );
    }

    #[test]
    fn test_storage_revert() {
        let code = vec![
            Operation::Push1(U256::from(42)).opcode(), // Value to store
            42,
            Operation::Push1(U256::from(0)).opcode(), // Key
            0,
            Operation::SStore.opcode(), // Store the value (SSTORE)
            Operation::Push1(U256::from(0)).opcode(), // Key
            0,
            Operation::SLoad.opcode(), // Load the value back (SLOAD)
            Operation::Push1(U256::from(10)).opcode(), // Revert memory length
            10,
            Operation::Push1(U256::from(0)).opcode(), // Revert memory offset
            0,
            Operation::Revert.opcode(), // Trigger revert
        ];

        let mut vm = VM::new(
            Contract::new(code.clone()),
            ExecutionContext::new(
                Address::from_hex("0x169EE3A023A8D9fF2E0D94cf8220b1Ba40D59794").unwrap(),
                Address::from_hex("0x169EE3A023A8D9fF2E0D94cf8220b1Ba40D59794").unwrap(),
                ETH_TO_WEI,
                vec![],
                ETH_TO_WEI,
            ),
            Arc::new(Mutex::new(State::new())),
        );

        // Execute the operations in sequence
        let result = vm.execute_operations(code).unwrap();

        // Assert that execution resulted in a revert
        // Check that no storage modifications persist after revert
        let key = U256::from(0);
        assert_eq!(vm.contract.storage.get(&key), None);
        assert!(
            matches!(result, ExecutionResult::Revert { .. }),
            "Expected a revert operation."
        );
    }
}
