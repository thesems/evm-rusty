use std::fs;
use crate::evm::operation::{Operation, OperationError};
use alloy_primitives::U256;

#[derive(Debug)]
pub enum ParserError {
    IncompletePush,
    InvalidOpcode,
}

impl From<OperationError> for ParserError {
    fn from(err: OperationError) -> Self {
        match err {
            _ => ParserError::InvalidOpcode,
        }
    }
}


// Structure to handle bytecode parsing
pub struct BytecodeParser {
    pub bytecode: Vec<u8>,
    pc: usize,
}

impl BytecodeParser {
    pub fn new(bytecode: Vec<u8>) -> Self {
        Self { bytecode, pc: 0 }
    }

    pub fn from(filepath: &str) -> Result<Self, std::io::Error> {
        let bytecode = fs::read(filepath)?;
        Ok(Self { bytecode, pc: 0 })
    }

    pub fn compile(&mut self) -> Result<Vec<Operation>, ParserError> {
        let mut operations = Vec::new();
        while let Some(operation) = self.next_operation()? {
            operations.push(operation);
        }
        Ok(operations)
    }

    fn next_operation(&mut self) -> Result<Option<Operation>, ParserError> {
        if self.pc >= self.bytecode.len() {
            return Ok(None);
        }

        let opcode = self.bytecode[self.pc];

        let operation = match opcode {
            // Handle push operations specially
            n @ 0x60..=0x7f => {
                let bytes_to_read = (n - 0x60 + 1) as usize;
                if self.pc + bytes_to_read >= self.bytecode.len() {
                    return Err(ParserError::IncompletePush);
                }

                // Read the specified number of bytes
                let mut value = U256::from(0);
                for i in 0..bytes_to_read {
                    value = value << 8;
                    value = value + U256::from(self.bytecode[self.pc + 1 + i]);
                }

                self.pc += bytes_to_read + 1;
                Operation::from_byte(opcode, Some(value))
            }
            // Handle all other operations
            _ => {
                self.pc += 1;
                Operation::from_byte(opcode, None)
            }
        }?;

        Ok(Some(operation))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_compile() {
        let file_path = "./test/Add.evm";
        let bytecode = fs::read(file_path)
            .expect("Failed to read EVM bytecode file");

        let mut parser = BytecodeParser::new(bytecode);

        let operations = parser.compile()
            .expect("Compilation of EVM bytecode failed");

        assert!(!operations.is_empty(), "The operations vector should not be empty");
    }
}