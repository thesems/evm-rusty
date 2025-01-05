use alloy_primitives::U256;
use crate::evm::errors::VMError;

const MAX_STACK_SIZE: usize = 1024;

#[derive(Default)]
pub struct Stack {
    stack_list: Vec<U256>
}

impl Stack {
    pub fn top(&self) -> Option<&U256> {
        self.stack_list.last()
    }

    pub fn stack_size(&self) -> usize {
        self.stack_list.len()
    }

    pub fn is_full(&self) -> bool {
        self.stack_size() >= MAX_STACK_SIZE
    }

    pub fn clear(&mut self) {
        self.stack_list.clear();
    }

    pub fn push(&mut self, value: U256) -> Result<(), VMError> {
        if self.is_full() {
            return Err(VMError::StackFull);
        }
        self.stack_list.push(value);
        Ok(())
    }

    pub fn pop(&mut self) -> Result<U256, VMError> {
        self.stack_list.pop().ok_or(VMError::NoItemsOnStack)
    }

    pub fn duplicate(&mut self, index: usize) -> Result<(), VMError> {
        if index == 0 || index > self.stack_list.len() {
            return Err(VMError::StackUnderflow);
        }
        let item_to_duplicate = self.stack_list[self.stack_list.len() - index].clone();
        self.push(item_to_duplicate)
    }

    pub fn swap(&mut self, item_num: usize) -> Result<(), VMError> {
        let stack_len = self.stack_size();
        if item_num == 0 || item_num > 16 || item_num > stack_len {
            return Err(VMError::StackUnderflow);
        }

        let temp = self.stack_list[stack_len - 1].clone(); // Assuming stack items need to be cloned
        self.stack_list[stack_len - 1] = self.stack_list[stack_len - item_num - 1].clone();
        self.stack_list[stack_len - item_num - 1] = temp;
        Ok(())
    }
}
