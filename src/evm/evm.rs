use crate::evm::evm::VMError::{NoItemsOnStack, NotEnoughItemsOnStack, StackFull};
use crate::evm::operation::Operation;
use alloy_primitives::U256;

const MAX_STACK_SIZE: u32 = 1024;

#[derive(Debug, Clone)]
pub enum VMError {
    StackFull,
    NotEnoughItemsOnStack,
    NoItemsOnStack,
}

struct VM {
    stack: Vec<U256>,
    code: Vec<Operation>,
    available_gas: u64,
}

impl VM {
    pub fn new(code: Vec<Operation>, available_gas: u64) -> Self {
        Self {
            stack: Vec::new(),
            code,
            available_gas,
        }
    }

    pub fn execute(&mut self) -> Result<(), VMError> {
        let len = self.code.len();
        for i in 0..len {
            self.process_operation(i)?;
        }
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
        let operation = &self.code[op_idx];
        let stack_req = operation.stack_req();
        let gas_cost = operation.gas_cost();

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

    #[test]
    fn test_add_operation() {
        let code = vec![
            Operation::Push1(U256::from(1)),
            Operation::Push1(U256::from(1)),
            Operation::Add,
        ];
        let mut vm = VM::new(code, 10);
        vm.execute().unwrap();
        assert_eq!(*vm.stack.last().unwrap(), U256::from(2));
    }
}
