use alloy_primitives::U256;
use crate::evm::memory::MemoryError::OutOfBounds;

#[derive(Debug, Clone)]
pub enum MemoryError {
    OutOfBounds,
    OutOfGas
}

#[derive(Default)]
pub struct Memory {
    memory: Vec<u8>,
}

impl Memory {
    pub fn clear(&mut self) {
        self.memory.clear();
    }

    pub fn get(&self, index: usize) -> Option<&u8> {
        self.memory.get(index)
    }

    pub fn set(&mut self, index: usize, value: u8) -> Result<(), MemoryError> {
        if index >= self.memory.len() {
            return Err(OutOfBounds);
        }
        self.memory[index] = value;
        Ok(())
    }

    pub fn read(&mut self, offset: usize, length: usize) -> &[u8] {
        if self.memory.len() < offset + length {
            self.memory.resize(offset + length, 0);
        }
        &self.memory[offset..offset + length]
    }

    pub fn write(&mut self, offset: usize, value: U256, gas_available: u64) -> Result<(), MemoryError> {
        let bytes = value.to_be_bytes::<32>();
        self.expand_memory(offset, 32, gas_available)?;
        self.memory[offset..offset + 32].copy_from_slice(&bytes);
        Ok(())
    }

    pub fn expand_memory(
        &mut self,
        offset: usize,
        required_size: usize,
        gas_available: u64,
    ) -> Result<(), MemoryError> {
        let new_size = offset + required_size;
        if self.memory.len() < new_size {
            if Self::calc_memory_expansion_gas(offset + 32) < gas_available {
                self.memory.resize(new_size, 0);
            } else {
                return Err(MemoryError::OutOfGas);
            }
        }
        Ok(())
    }

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
    pub fn calc_memory_expansion_gas(memory_byte_size: usize) -> u64 {
        let memory_size_word = (memory_byte_size + 31) / 32;
        let memory_cost = (memory_size_word * memory_size_word / 512) + (3 * memory_size_word);
        memory_cost as u64
    }
}
