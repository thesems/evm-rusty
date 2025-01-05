use crate::evm::bytecode_parser::ParserError;

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
}
impl From<ParserError> for VMError {
    fn from(value: ParserError) -> Self {
        match value {
            ParserError::IncompletePush => VMError::InvalidBytecode,
            _ => VMError::InvalidTransaction,
        }
    }
}
