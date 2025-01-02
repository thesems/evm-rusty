use crate::block::account::Account;
use crate::block::state::State;
use crate::evm::bytecode_parser::{BytecodeParser, ParserError};
use crate::evm::evm::VMError::{NoItemsOnStack, NotEnoughItemsOnStack, StackFull};
use crate::evm::operation::Operation;
use crate::transaction::transaction::Transaction;
use alloy_rlp::{Decodable, Encodable, RlpDecodable, RlpEncodable};

use alloy_primitives::{keccak256, Address, B256, U256};
use clap::builder::TypedValueParser;
use k256::pkcs8::der::Encode;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use crate::crypto::hash::{hash_slice_to_b256, hash_string_to_u256};

const MAX_STACK_SIZE: u32 = 1024;

#[derive(Debug, RlpEncodable, RlpDecodable, PartialEq)]
pub struct AddressNonce {
    pub address: Vec<u8>,
    pub nonce: u64,
}

#[derive(Debug)]
pub enum VMError {
    StackFull,
    NotEnoughItemsOnStack,
    NoItemsOnStack,
    NotImplemented,
    ContractNotFound,
    InvalidTransaction,
    InvalidBytecode,
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
    pub code: Vec<Operation>,
    pub storage: HashMap<U256, U256>,
}

impl Contract {
    pub fn new(code: Vec<Operation>) -> Self {
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

pub struct VM {
    stack: Vec<U256>,
    memory: Vec<u8>,
    contract: Contract,
    available_gas: u64,
    context: ExecutionContext,
    pc: usize,
    state: Arc<Mutex<State>>,
}

impl VM {
    pub fn new(contract: Contract, context: ExecutionContext, state: Arc<Mutex<State>>) -> Self {
        Self {
            stack: Vec::new(),
            memory: vec![],
            contract,
            available_gas: context.gas,
            context,
            pc: 0,
            state,
        }
    }

    pub fn execute_operations(&mut self, code: Vec<Operation>) -> Result<(), VMError> {
        for i in 0..code.len() {
            self.process_operation(i)?;
        }
        Ok(())
    }

    pub fn execute_transaction(&mut self, transaction: Transaction) -> Result<(), VMError> {
        // differentiate contract creation
        if transaction.to.is_zero() {
            self.call_contract_create(transaction)
        } else {
            self.call_contract(transaction)
        }
    }

    fn generate_contract_address(&self, address: Address, nonce: u64) -> Address {
        let mut buffer = Vec::<u8>::new();
        AddressNonce { address: address.0.as_slice().to_vec(), nonce }.encode(&mut buffer);
        let hash = keccak256(&buffer);
        Address::from_slice(&hash[12..])
    }

    pub fn call_contract_create(&mut self, transaction: Transaction) -> Result<(), VMError> {
        let sender = transaction
            .get_sender_address()
            .ok_or(VMError::InvalidTransaction)?;
        let contract_address = self.generate_contract_address(sender, transaction.nonce);

        self.state.lock().unwrap().accounts.insert(
            contract_address,
            Account::new(
                transaction.value,
                hash_slice_to_b256(transaction.input_data.as_slice()),
                B256::ZERO, // TODO: storage root hash?
            ),
        );

        self.context = ExecutionContext::new(
            transaction.get_sender_address().unwrap(),
            contract_address,
            transaction.value,
            transaction.input_data.clone(),
            transaction.gas_limit,
        );

        let mut parser = BytecodeParser::new(transaction.input_data.clone());
        match parser.compile() {
            Ok(code) => self.execute_operations(code),
            Err(_) => Err(VMError::InvalidBytecode),
        }
    }

    pub fn call_contract(&mut self, transaction: Transaction) -> Result<(), VMError> {
        self.context = ExecutionContext::new(
            transaction.get_sender_address().unwrap(),
            transaction.to,
            transaction.value,
            transaction.input_data.clone(),
            transaction.gas_limit,
        );

        // extract function selector
        let selector = &transaction.input_data[0..4];

        self.pc = 0;
        self.stack.clear();
        self.memory.clear();

        Ok(())
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

    fn process_operation(&mut self, op_idx: usize) -> Result<(), VMError> {
        let operation = &self.contract.code[op_idx];
        let stack_req = operation.stack_req();
        let gas_cost = operation.gas_cost();

        let operation_name = format!("{:?}", operation);

        if self.stack_size() < stack_req.min_stack_height {
            return Err(NotEnoughItemsOnStack);
        }

        match operation {
            Operation::Stop => {}
            Operation::Add => {
                self.add()?;
            }
            Operation::Mul => {}
            Operation::Sub => {}
            Operation::Div => {}
            Operation::SDiv => {}
            Operation::Mod => {}
            Operation::SMod => {}
            Operation::AddMod => {}
            Operation::MulMod => {}
            Operation::Exp => {}
            Operation::SignExtend => {}
            Operation::Lt => {}
            Operation::Gt => {}
            Operation::Slt => {}
            Operation::Sgt => {}
            Operation::Eq => {}
            Operation::IsZero => {}
            Operation::And => {}
            Operation::Or => {}
            Operation::Xor => {}
            Operation::Not => {}
            Operation::Byte => {}
            Operation::Shl => {}
            Operation::Shr => {}
            Operation::Sar => {}
            Operation::Address => {}
            Operation::Balance => {}
            Operation::Origin => {}
            Operation::Caller => {}
            Operation::CallValue => {}
            Operation::CallDataLoad => {}
            Operation::CallDataSize => {}
            Operation::CallDataCopy => {}
            Operation::CodeSize => {}
            Operation::CodeCopy => {}
            Operation::GasPrice => {}
            Operation::ExtCodeSize => {}
            Operation::ExtCodeCopy => {}
            Operation::ReturnDataSize => {}
            Operation::ReturnDataCopy => {}
            Operation::ExtCodeHash => {}
            Operation::BlockHash => {}
            Operation::Coinbase => {}
            Operation::Timestamp => {}
            Operation::Number => {}
            Operation::Difficulty => {}
            Operation::GasLimit => {}
            Operation::ChainId => {}
            Operation::SelfBalance => {}
            Operation::BaseFee => {}
            Operation::Pop => {}
            Operation::MLoad => {}
            Operation::MStore => {}
            Operation::MStore8 => {}
            Operation::SLoad => {}
            Operation::SStore => {}
            Operation::Jump => {}
            Operation::JumpI => {}
            Operation::PC => {}
            Operation::MSize => {}
            Operation::Gas => {}
            Operation::JumpDest => {}
            Operation::Push1(_) => {
                self.push(U256::from(1))?;
            }
            Operation::Push2(_) => {}
            Operation::Push3(_) => {}
            Operation::Push4(_) => {}
            Operation::Push5(_) => {}
            Operation::Push6(_) => {}
            Operation::Push7(_) => {}
            Operation::Push8(_) => {}
            Operation::Push9(_) => {}
            Operation::Push10(_) => {}
            Operation::Push11(_) => {}
            Operation::Push12(_) => {}
            Operation::Push13(_) => {}
            Operation::Push14(_) => {}
            Operation::Push15(_) => {}
            Operation::Push16(_) => {}
            Operation::Push17(_) => {}
            Operation::Push18(_) => {}
            Operation::Push19(_) => {}
            Operation::Push20(_) => {}
            Operation::Push21(_) => {}
            Operation::Push22(_) => {}
            Operation::Push23(_) => {}
            Operation::Push24(_) => {}
            Operation::Push25(_) => {}
            Operation::Push26(_) => {}
            Operation::Push27(_) => {}
            Operation::Push28(_) => {}
            Operation::Push29(_) => {}
            Operation::Push30(_) => {}
            Operation::Push31(_) => {}
            Operation::Push32(_) => {}
            Operation::Dup(_) => {}
            Operation::Swap1 => {}
            Operation::Swap2 => {}
            Operation::Swap3 => {}
            Operation::Swap4 => {}
            Operation::Swap5 => {}
            Operation::Swap6 => {}
            Operation::Swap7 => {}
            Operation::Swap8 => {}
            Operation::Swap9 => {}
            Operation::Swap10 => {}
            Operation::Swap11 => {}
            Operation::Swap12 => {}
            Operation::Swap13 => {}
            Operation::Swap14 => {}
            Operation::Swap15 => {}
            Operation::Swap16 => {}
            Operation::Log0 => {}
            Operation::Log1 => {}
            Operation::Log2 => {}
            Operation::Log3 => {}
            Operation::Log4 => {}
            Operation::Create => {}
            Operation::Call => {}
            Operation::CallCode => {}
            Operation::Return => {}
            Operation::DelegateCall => {}
            Operation::Create2 => {}
            Operation::StaticCall => {}
            Operation::Revert => {}
            Operation::Invalid => {}
            Operation::SelfDestruct => {}
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::hash::hash_string_to_u256;
    use crate::crypto::wallet::Wallet;
    use crate::evm::bytecode_parser::BytecodeParser;
    use alloy_primitives::hex::FromHex;

    #[test]
    fn test_add_operation() {
        let code = vec![
            Operation::Push1(U256::from(1)),
            Operation::Push1(U256::from(1)),
            Operation::Add,
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

        let mut vm = VM::new(
            Contract::new(parser.compile().unwrap()),
            ExecutionContext::new(
                Address::from_hex("0x169EE3A023A8D9fF2E0D94cf8220b1Ba40D59794").unwrap(),
                Address::from_hex("0x169EE3A023A8D9fF2E0D94cf8220b1Ba40D59794").unwrap(),
                0,
                vec![],
                0,
            ),
            Arc::new(Mutex::new(State::new())),
        );

        let eth_wallet = Wallet::generate();

        let tx_create = Transaction::new(
            Address::ZERO,
            100,
            100,
            100,
            100,
            parser.bytecode,
            Some(&eth_wallet.private_key),
        );

        vm.execute_transaction(tx_create).unwrap();

        assert_eq!(
            *vm.contract.storage.get(&hash_string_to_u256("counter")).unwrap(),
            U256::from(0)
        );

        let tx_inc = Transaction::new(
            eth_wallet.address,
            100,
            100,
            100,
            100,
            hash_string_to_u256("inc()").as_le_slice()[..4].to_vec(), // function to call: inc()
            Some(&eth_wallet.private_key),
        );

        vm.execute_transaction(tx_inc).unwrap();

        assert_eq!(
            *vm.contract.storage.get(&hash_string_to_u256("counter")).unwrap(),
            U256::from(1)
        );
    }
}
