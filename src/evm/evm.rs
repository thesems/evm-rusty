use crate::block::account::Account;
use crate::block::state::State;
use crate::evm::bytecode_parser::BytecodeParser;
use crate::evm::evm::VMError::NotEnoughItemsOnStack;
use crate::evm::operation::Operation;
use crate::transaction::transaction::Transaction;
use alloy_rlp::{Encodable, RlpDecodable, RlpEncodable};

use crate::crypto::hash::hash_slice_to_b256;
use crate::evm::errors::VMError;
use crate::evm::stack::Stack;
use alloy_primitives::{keccak256, Address, FixedBytes, B256, I256, U256};
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use crate::evm::execution_context::ExecutionContext;
use crate::evm::executor::ExecutionResult;
use crate::evm::memory::Memory;

#[derive(Debug, RlpEncodable, RlpDecodable, PartialEq)]
pub struct AddressNonce {
    pub address: Vec<u8>,
    pub nonce: u64,
}

#[derive(Clone)]
pub struct Contract {
    pub code: Rc<Vec<u8>>,
    pub storage: HashMap<U256, U256>,
}

impl Contract {
    pub fn new(code: Vec<u8>) -> Self {
        Self {
            code: Rc::new(code),
            storage: HashMap::new(),
        }
    }
}

enum StorageChangeType {
    Set,
    Delete,
}

pub struct VM {
    stack: Stack,
    memory: Memory,
    contract: Contract,
    gas_available: u64,
    context: ExecutionContext,
    creation_offset: usize,
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
            stack: Stack::default(),
            memory: Memory::default(),
            contract,
            gas_available: context.gas,
            context,
            creation_offset,
            state,
            storage_revert: HashMap::new(),
        }
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

    pub fn execute_operations(&mut self) -> Result<ExecutionResult, VMError> {
        let code_clone = self.contract.code.clone();
        let mut parser = BytecodeParser::new(code_clone.as_slice());

        let mut execution_result = ExecutionResult::Revert {
            reason: vec![],
            gas_used: 0,
        };

        while let Some(operation) = parser.next() {
            // let start = self.stack.len().checked_sub(3).unwrap_or(0);
            // let values: Vec<U256> = self.stack[start..self.stack.len()].iter().map(|x| x.clone()).collect();
            // eprintln!("Top 3 stack items: {:?} <-", values);
            // eprintln!("Executing operation {:?}", operation);
            execution_result = self.process_operation(&operation)?;

            self.gas_available -= match execution_result {
                ExecutionResult::Success {
                    gas_used,
                    jump_dest,
                    halt,
                    ..
                } => {
                    if halt {
                        return Ok(execution_result);
                    }
                    if jump_dest != 0 {
                        parser.pc = jump_dest;
                    }
                    gas_used
                }
                ExecutionResult::Revert { gas_used, .. } => gas_used,
            };
        }

        Ok(execution_result)
    }

    pub fn execute_transaction(
        &mut self,
        transaction: Transaction,
    ) -> Result<ExecutionResult, VMError> {
        self.stack.clear();
        self.memory.clear();
        self.storage_revert.clear();

        self.context = ExecutionContext::new(
            transaction
                .get_sender_address()
                .ok_or(VMError::InvalidTransaction)?,
            transaction.to,
            transaction.value,
            transaction.input_data.clone(),
            transaction.gas_limit,
        );
        self.gas_available = self.context.gas;

        // differentiate contract creation
        if transaction.to.is_zero() {
            self.call_contract_create(transaction)
        } else {
            self.call_contract()
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
                0,
                hash_slice_to_b256(&transaction.input_data),
                B256::ZERO, // TODO: storage root hash?
            ),
        );

        self.contract.code = Rc::new(transaction.input_data);

        match self.execute_operations() {
            Ok(result) => {
                if let ExecutionResult::Success { return_data, .. } = result.clone() {
                    self.contract.code =
                        Rc::new(return_data.ok_or(VMError::InvalidContractCreationResponse)?);
                }
                Ok(result)
            }
            Err(err) => Err(err),
        }
    }

    pub fn call_contract(&mut self) -> Result<ExecutionResult, VMError> {
        self.execute_operations()
    }

    fn jump_to(&mut self, offset: usize) -> Result<ExecutionResult, VMError> {
        if let Operation::JumpDest = Operation::from_byte(self.contract.code[offset], None)
            .map_err(|_| VMError::InvalidBytecode)?
        {
            Ok(ExecutionResult::Success {
                return_data: None,
                gas_used: 0,
                jump_dest: offset,
                halt: false,
            })
        } else {
            Err(VMError::InvalidJumpDest)
        }
    }

    fn process_operation(&mut self, operation: &Operation) -> Result<ExecutionResult, VMError> {
        let stack_req = operation.stack_req();
        let operation_name = format!("{:?}", operation);

        if self.stack.stack_size() < stack_req.min_stack_height as usize {
            return Err(NotEnoughItemsOnStack(operation_name));
        }

        let gas_cost = operation.gas_cost();
        if self.gas_available < gas_cost.base {
            return Err(VMError::OutOfGas);
        }

        let not_impl_error = format!("Operation {:?} is not implemented", operation_name);

        match operation {
            Operation::Stop => {
                return Ok(ExecutionResult::Success {
                    return_data: None,
                    gas_used: gas_cost.base,
                    jump_dest: 0,
                    halt: true,
                });
            }
            Operation::Add => {
                let a = self.stack.pop()?;
                let b = self.stack.pop()?;
                self.stack.push(a + b)?;
            }
            Operation::Mul => {
                let a = self.stack.pop()?;
                let b = self.stack.pop()?;
                self.stack.push(a * b)?;
            }
            Operation::Sub => {
                let a = self.stack.pop()?;
                let b = self.stack.pop()?;
                self.stack.push(a - b)?;
            }
            Operation::Div => {
                let a = self.stack.pop()?;
                let b = self.stack.pop()?;
                if b.is_zero() {
                    return Err(VMError::DivisionByZero);
                }
                self.stack.push(a / b)?;
            }
            Operation::SDiv => {
                let a = I256::from_limbs(*self.stack.pop()?.as_limbs());
                let b = I256::from_limbs(*self.stack.pop()?.as_limbs());
                if b.is_zero() {
                    return Err(VMError::DivisionByZero);
                }
                self.stack.push(U256::from_limbs(*(a / b).as_limbs()))?;
            }
            Operation::Mod => {
                let a = self.stack.pop()?;
                let b = self.stack.pop()?;
                if b.is_zero() {
                    return Err(VMError::DivisionByZero);
                }
                self.stack.push(a % b)?;
            }
            Operation::SMod => {
                let a = I256::from_limbs(*self.stack.pop()?.as_limbs());
                let b = I256::from_limbs(*self.stack.pop()?.as_limbs());
                if b.is_zero() {
                    return Err(VMError::DivisionByZero);
                }
                self.stack.push(U256::from_limbs(*(a % b).as_limbs()))?;
            }
            Operation::AddMod => panic!("{}", not_impl_error),
            Operation::MulMod => panic!("{}", not_impl_error),
            Operation::Exp => panic!("{}", not_impl_error),
            Operation::SignExtend => panic!("{}", not_impl_error),
            Operation::Lt => {
                let a = self.stack.pop()?;
                let b = self.stack.pop()?;
                self.stack.push(U256::from(a < b))?;
            }
            Operation::Gt => {
                let a = self.stack.pop()?;
                let b = self.stack.pop()?;
                self.stack.push(U256::from(a > b))?;
            }
            Operation::Slt => {
                let a = I256::from_limbs(*self.stack.pop()?.as_limbs());
                let b = I256::from_limbs(*self.stack.pop()?.as_limbs());
                self.stack.push(U256::from(a < b))?;
            }
            Operation::Sgt => {
                let a = I256::from_limbs(*self.stack.pop()?.as_limbs());
                let b = I256::from_limbs(*self.stack.pop()?.as_limbs());
                self.stack.push(U256::from(a > b))?;
            }
            Operation::Eq => {
                let a = self.stack.pop()?;
                let b = self.stack.pop()?;
                self.stack.push(U256::from(a == b))?;
            }
            Operation::IsZero => {
                let item = self.stack.pop()?;
                self.stack.push(U256::from(item.is_zero()))?;
            }
            Operation::And => {
                let a = self.stack.pop()?;
                let b = self.stack.pop()?;
                self.stack.push(a & b)?;
            }
            Operation::Or => {
                let a = self.stack.pop()?;
                let b = self.stack.pop()?;
                self.stack.push(a | b)?;
            }
            Operation::Xor => {
                let a = self.stack.pop()?;
                let b = self.stack.pop()?;
                self.stack.push(a ^ b)?;
            }
            Operation::Not => {
                let a = self.stack.pop()?;
                self.stack.push(!a)?;
            }
            Operation::Byte => {
                let i = self.stack.pop()?.to::<usize>(); // Byte offset
                let x = self.stack.pop()?; // 32-byte value

                // Extract the byte at the specified offset, handle out-of-range access
                let byte = if i < 32 { x.byte(31 - i) } else { 0 };
                self.stack.push(U256::from(byte))?;
            }
            Operation::Shl => {
                let shift = self.stack.pop()?;
                let value = self.stack.pop()?;
                self.stack.push(value << shift)?;
            }
            Operation::Shr => {
                let shift = self.stack.pop()?;
                let value = self.stack.pop()?;
                self.stack.push(value >> shift)?;
            }
            Operation::Sar => {
                let shift = self.stack.pop()?.to::<usize>();
                let value = I256::from_limbs(*self.stack.pop()?.as_limbs());
                let shifted = value >> shift;
                self.stack.push(U256::from_limbs(*shifted.as_limbs()))?;
            }
            Operation::Address => {
                self.stack.push(U256::from_be_slice(self.context.address.as_slice()))?;
            }
            Operation::Balance => {
                let address = Address::from_word(FixedBytes::from(self.stack.pop()?.to_be_bytes::<32>()));

                let balance = self
                    .state
                    .lock()
                    .unwrap()
                    .accounts
                    .get(&address)
                    .map_or(U256::ZERO, |account| U256::from(account.balance));

                self.stack.push(balance)?;
            }
            Operation::Origin => {
                self.stack.push(U256::from_be_slice(self.context.caller.as_slice()))?;
            }
            Operation::Caller => {
                self.stack.push(U256::from_be_slice(self.context.caller.as_slice()))?;
            }
            Operation::CallValue => {
                self.stack.push(U256::from(self.context.value))?;
            }
            Operation::CallDataLoad => {
                let i = self.stack.pop()?.to::<usize>();
                let mut result = [0u8; 32];

                if i < self.context.data.len() {
                    let slice_end: usize = (i + 32).min(self.context.data.len());
                    let dest_end = 32.min(slice_end);
                    result[..dest_end].copy_from_slice(&self.context.data[i..slice_end]);
                }

                self.stack.push(U256::from_be_slice(&result))?;
            }
            Operation::CallDataSize => {
                self.stack.push(U256::from(self.context.data.len()))?;
            }
            Operation::CallDataCopy => panic!("{}", not_impl_error),
            Operation::CodeSize => {
                let code_len = self.contract.code.len();
                self.stack.push(U256::from(code_len))?;
            }
            Operation::CodeCopy => {
                let dest_offset = self.stack.pop()?.to::<usize>();
                let offset = self.stack.pop()?.to::<usize>();
                let size = self.stack.pop()?.to::<usize>();

                let minimum_word_size = (size as u64 + 31) / 32;
                let static_gas = 3;
                let dynamic_gas = 3 * minimum_word_size + Memory::calc_memory_expansion_gas(size);

                if self.gas_available < gas_cost.base + static_gas + dynamic_gas {
                    return Err(VMError::OutOfGas);
                }

                self.memory.expand_memory(dest_offset, size, self.gas_available)?;

                // Get the raw bytecode slice
                for i in 0..size {
                    let byte = if offset + i < self.contract.code.len() {
                        self.contract.code[offset + i] // Copy raw byte directly
                    } else {
                        0 // For out-of-bound bytes, pad with 0
                    };
                    self.memory.set(dest_offset + i, byte)?;
                }

                return Ok(ExecutionResult::Success {
                    return_data: None,
                    gas_used: gas_cost.base + static_gas + dynamic_gas,
                    jump_dest: 0,
                    halt: false,
                });
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
                self.stack.pop()?; // Simply discard the value at the top of the stack
            }
            Operation::MLoad => panic!("{}", not_impl_error),
            Operation::MStore => {
                let offset = self.stack.pop()?.to::<usize>();
                let value = self.stack.pop()?;
                self.memory.write(offset, value, self.gas_available)?;
            }
            Operation::MStore8 => panic!("{}", not_impl_error),
            Operation::SLoad => {
                let key = self.stack.pop()?; // Get the storage key from the stack
                let value = self
                    .contract
                    .storage
                    .get(&key)
                    .cloned()
                    .unwrap_or(U256::ZERO);
                self.stack.push(value)?;
            }
            Operation::SStore => {
                let storage_key = self.stack.pop()?;
                let storage_value = self.stack.pop()?;

                let prev_value = self.contract.storage.insert(storage_key, storage_value);

                if prev_value.is_none() {
                    self.storage_revert
                        .insert(storage_key, (StorageChangeType::Delete, storage_value));
                } else {
                    self.storage_revert
                        .insert(storage_key, (StorageChangeType::Set, prev_value.unwrap()));
                }
            }
            Operation::Jump => {
                let offset = self.stack.pop()?.to::<usize>();
                return self.jump_to(offset);
            }
            Operation::JumpI => {
                let offset = self.stack.pop()?.to::<usize>();
                let jump = self.stack.pop()?;

                if !jump.is_zero() {
                    return self.jump_to(offset);
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
                self.stack.push(U256::ZERO)?;
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
                self.stack.push(*value)?;
            }
            Operation::Dup(item_num) => {
                self.stack.duplicate(*item_num as usize)?;
            }
            Operation::Swap(item_num) => {
                self.stack.swap(*item_num as usize)?;
            }
            Operation::Log0 => panic!("{}", not_impl_error),
            Operation::Log1 => panic!("{}", not_impl_error),
            Operation::Log2 => panic!("{}", not_impl_error),
            Operation::Log3 => panic!("{}", not_impl_error),
            Operation::Log4 => panic!("{}", not_impl_error),
            Operation::Create => panic!("{}", not_impl_error),
            Operation::Call => panic!("{}", not_impl_error),
            Operation::CallCode => panic!("{}", not_impl_error),
            Operation::Return => {
                let offset = self.stack.pop()?.to::<usize>();
                let size = self.stack.pop()?.to::<usize>();

                let return_data = self.memory.read(offset, size);

                return Ok(ExecutionResult::Success {
                    return_data: Some(return_data.to_vec()),
                    gas_used: gas_cost.base,
                    jump_dest: 0,
                    halt: false,
                });
            }
            Operation::DelegateCall => panic!("{}", not_impl_error),
            Operation::Create2 => panic!("{}", not_impl_error),
            Operation::StaticCall => panic!("{}", not_impl_error),
            Operation::Revert => {
                let length = self.stack.pop()?.to::<usize>();
                let offset = self.stack.pop()?.to::<usize>();

                self.revert_storage();
                let revert_data = self.memory.read(offset, length);

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
            jump_dest: 0,
            halt: false,
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
            1,
            Operation::Push1(U256::from(1)).opcode(),
            1,
            Operation::Add.opcode(),
        ];

        let mut vm = VM::new(
            Contract::new(code),
            ExecutionContext::new(
                Address::from_hex("0x169EE3A023A8D9fF2E0D94cf8220b1Ba40D59794").unwrap(),
                Address::from_hex("0x169EE3A023A8D9fF2E0D94cf8220b1Ba40D59794").unwrap(),
                0,
                vec![],
                21000,
            ),
            Arc::new(Mutex::new(State::new())),
        );
        vm.execute_operations().unwrap();
        assert_eq!(*vm.stack.top().unwrap(), U256::from(2));
    }

    #[test]
    fn test_contract_basics() {
        let sender = Wallet::generate();
        let receiver = Wallet::generate();

        let mut state = State::new();
        state.set_account(
            sender.address,
            Account::new(2 * ETH_TO_WEI, B256::ZERO, B256::ZERO),
        );

        let mut vm = VM::new(
            Contract::new(vec![]),
            ExecutionContext::default(),
            Arc::new(Mutex::new(state)),
        );

        let bytecode = BytecodeParser::read_bytecode_from_file("./test/Counter.evm").unwrap();
        let tx_create = Transaction::new(
            Address::ZERO,
            0,
            1 * ETH_TO_WEI,
            100,
            100,
            bytecode,
            Some(&sender.private_key),
        );

        vm.execute_transaction(tx_create).unwrap();

        assert_eq!(
            *vm.contract.storage.get(&U256::ZERO).unwrap(),
            U256::from(10)
        );

        let tx_inc = Transaction::new(
            receiver.address,
            GWEI_TO_WEI,
            30000,
            10000,
            10000,
            hash_string_to_u256("inc()").to_be_bytes::<32>()[..4].to_vec(),
            Some(&sender.private_key),
        );

        vm.execute_transaction(tx_inc).unwrap();

        assert_eq!(
            *vm.contract.storage.get(&U256::ZERO).unwrap(),
            U256::from(11)
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
        let result = vm.execute_operations().unwrap();

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
