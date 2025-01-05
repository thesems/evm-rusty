use crate::evm::bytecode_parser::ParserError;
use crate::evm::memory::MemoryError;

#[derive(Debug, Clone)]
pub enum VMError {
    StackFull,
    NotEnoughItemsOnStack(String),
    NoItemsOnStack,
    NotImplemented,
    ContractNotFound,
    InvalidTransaction,
    InvalidBytecode,
    InvalidContractCreationResponse,
    OutOfGas,
    StackUnderflow,
    NoOperationExecuted,
    InvalidJumpDest,
    DivisionByZero,
    MemoryError,
}
impl From<ParserError> for VMError {
    fn from(value: ParserError) -> Self {
        match value {
            ParserError::IncompletePush => VMError::InvalidBytecode,
            ParserError::InvalidOpcode => VMError::InvalidBytecode,
        }
    }
}
impl From<MemoryError> for VMError {
    fn from(value: MemoryError) -> Self {
        match value {
            MemoryError::OutOfBounds => VMError::MemoryError,
            MemoryError::OutOfGas => VMError::OutOfGas,
        }
    }
}
