use super::*;
use crate::crypto::hash::hash_string_to_u256;
use crate::crypto::wallet::Wallet;
use crate::evm::bytecode_parser::BytecodeParser;
use crate::transaction::transaction::{ETH_TO_WEI, GWEI_TO_WEI};
use alloy_primitives::hex::FromHex;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

#[test]
fn test_add_operation() {
    let code = vec![
        Operation::Push1(U256::from(1)).opcode(),
        1,
        Operation::Push1(U256::from(1)).opcode(),
        1,
        Operation::Add.opcode(),
    ];

    let to = Address::from_hex("0x169EE3A023A8D9fF2E0D94cf8220b1Ba40D59794").unwrap();
    let sender = to.clone();

    let state = Arc::new(Mutex::new(State::new()));
    state
        .lock()
        .unwrap()
        .set_contract(to, Rc::new(RefCell::new(Contract::new(code))));

    let mut vm = VM::new(ExecutionContext::new(sender, to, 0, vec![], 21000), state).unwrap();
    vm.execute_operations().unwrap();
    assert_eq!(*vm.stack.top().unwrap(), U256::from(2));
}

#[test]
fn test_contract_basics() {
    let sender = Wallet::generate();

    let mut state = Arc::new(Mutex::new(State::new()));
    state.lock().unwrap().set_account(
        sender.address,
        Account::new(2 * ETH_TO_WEI, B256::ZERO, B256::ZERO),
    );

    let mut vm = VM::new(ExecutionContext::default(), state.clone()).unwrap();

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

    let exec_result = vm.execute_transaction(tx_create).unwrap();

    assert_eq!(
        *vm.contract()
            .unwrap()
            .borrow()
            .storage
            .get(&U256::ZERO)
            .unwrap(),
        U256::from(10)
    );

    if let ExecutionResult::Success {
        new_contract_address,
        ..
    } = exec_result
    {
        assert!(new_contract_address.is_some());

        let tx_inc = Transaction::new(
            new_contract_address.unwrap(),
            GWEI_TO_WEI,
            30000,
            10000,
            10000,
            hash_string_to_u256("inc()").to_be_bytes::<32>()[..4].to_vec(),
            Some(&sender.private_key),
        );

        vm.execute_transaction(tx_inc).unwrap();

        assert_eq!(
            *vm.contract()
                .unwrap()
                .borrow()
                .storage
                .get(&U256::ZERO)
                .unwrap(),
            U256::from(11)
        );
    } else {
        panic!("Transaction execution failed");
    }
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

    let to = Address::from_hex("0x169EE3A023A8D9fF2E0D94cf8220b1Ba40D59794").unwrap();
    let sender = to.clone();

    let state = Arc::new(Mutex::new(State::new()));
    state
        .lock()
        .unwrap()
        .set_contract(to, Rc::new(RefCell::new(Contract::new(code))));

    let mut vm = VM::new(
        ExecutionContext::new(sender, to, ETH_TO_WEI, vec![], ETH_TO_WEI),
        state,
    )
    .unwrap();

    let result = vm.execute_operations().unwrap();

    let key = U256::from(0);
    assert_eq!(vm.contract().unwrap().borrow().storage.get(&key), None);
    assert!(
        matches!(result, ExecutionResult::Revert { .. }),
        "Expected a revert operation."
    );
}

#[test]
fn test_mload_operation() {
    let code = vec![
        Operation::Push1(U256::from(0xab)).opcode(), // Value to store
        0xab,
        Operation::Push1(U256::from(0)).opcode(), // Memory offset
        0,
        Operation::MStore.opcode(),
        Operation::Push1(U256::from(0)).opcode(), // Memory offset
        0,
        Operation::MLoad.opcode(),
    ];

    let to = Address::from_hex("0x169EE3A023A8D9fF2E0D94cf8220b1Ba40D59794").unwrap();
    let sender = to.clone();

    let state = Arc::new(Mutex::new(State::new()));
    state
        .lock()
        .unwrap()
        .set_contract(to, Rc::new(RefCell::new(Contract::new(code))));

    let mut vm = VM::new(ExecutionContext::new(sender, to, 0, vec![], 21000), state).unwrap();
    vm.execute_operations().unwrap();

    assert_eq!(*vm.stack.top().unwrap(), U256::from(0xab));
}

#[test]
fn test_mstore8_operation() {
    let code = vec![
        Operation::Push1(U256::from(0xab)).opcode(), // Value to store
        0xab,
        Operation::Push1(U256::from(31)).opcode(), // Memory offset
        31,
        Operation::MStore8.opcode(),
        Operation::Push1(U256::from(0)).opcode(), // Memory offset for MLOAD
        0,
        Operation::MLoad.opcode(),
    ];

    let to = Address::from_hex("0x169EE3A023A8D9fF2E0D94cf8220b1Ba40D59794").unwrap();
    let sender = to.clone();

    let state = Arc::new(Mutex::new(State::new()));
    state
        .lock()
        .unwrap()
        .set_contract(to, Rc::new(RefCell::new(Contract::new(code))));

    let mut vm = VM::new(ExecutionContext::new(sender, to, 0, vec![], 21000), state).unwrap();
    vm.execute_operations().unwrap();

    assert_eq!(*vm.stack.top().unwrap(), U256::from(0xab));
}

#[test]
fn test_calldatacopy_operation() {
    let code = vec![
        Operation::Push1(U256::from(4)).opcode(), // size
        4,
        Operation::Push1(U256::from(0)).opcode(), // calldata offset
        0,
        Operation::Push1(U256::from(0)).opcode(), // memory offset
        0,
        Operation::CallDataCopy.opcode(),
        Operation::Push1(U256::from(0)).opcode(), // memory offset for MLOAD
        0,
        Operation::MLoad.opcode(),
    ];

    let to = Address::from_hex("0x169EE3A023A8D9fF2E0D94cf8220b1Ba40D59794").unwrap();
    let sender = to.clone();

    let state = Arc::new(Mutex::new(State::new()));
    state
        .lock()
        .unwrap()
        .set_contract(to, Rc::new(RefCell::new(Contract::new(code))));

    let mut vm = VM::new(
        ExecutionContext::new(sender, to, 0, vec![0x11, 0x22, 0x33, 0x44], 21000),
        state,
    )
    .unwrap();
    vm.execute_operations().unwrap();

    assert_eq!(
        *vm.stack.top().unwrap(),
        U256::from_be_slice(&[
            0x11, 0x22, 0x33, 0x44, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0
        ])
    );
}

#[test]
fn test_keccak256_operation() {
    let code = vec![
        Operation::Push1(U256::from(0x44)).opcode(), // value byte
        0x44,
        Operation::Push1(U256::from(3)).opcode(), // memory offset
        3,
        Operation::MStore8.opcode(),
        Operation::Push1(U256::from(0x33)).opcode(), // value byte
        0x33,
        Operation::Push1(U256::from(2)).opcode(), // memory offset
        2,
        Operation::MStore8.opcode(),
        Operation::Push1(U256::from(0x22)).opcode(), // value byte
        0x22,
        Operation::Push1(U256::from(1)).opcode(), // memory offset
        1,
        Operation::MStore8.opcode(),
        Operation::Push1(U256::from(0x11)).opcode(), // value byte
        0x11,
        Operation::Push1(U256::from(0)).opcode(), // memory offset
        0,
        Operation::MStore8.opcode(),
        Operation::Push1(U256::from(4)).opcode(), // size
        4,
        Operation::Push1(U256::from(0)).opcode(), // memory offset
        0,
        Operation::Keccak256.opcode(),
    ];

    let to = Address::from_hex("0x169EE3A023A8D9fF2E0D94cf8220b1Ba40D59794").unwrap();
    let sender = to.clone();

    let state = Arc::new(Mutex::new(State::new()));
    state
        .lock()
        .unwrap()
        .set_contract(to, Rc::new(RefCell::new(Contract::new(code))));

    let mut vm = VM::new(ExecutionContext::new(sender, to, 0, vec![], 21000), state).unwrap();
    vm.execute_operations().unwrap();

    let expected = keccak256([0x11, 0x22, 0x33, 0x44]);
    assert_eq!(
        *vm.stack.top().unwrap(),
        U256::from_be_slice(expected.as_slice())
    );
}
