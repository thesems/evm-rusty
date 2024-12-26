use thiserror::Error;

#[derive(Error, Debug)]
pub enum TransactionError {
    #[error("sender account doesn't exist")]
    SenderAccountDoesNotExist,
    #[error("insufficient balance")]
    InsufficientBalance,
    #[error("maximum gas limit exceeded")]
    InsufficientGas,
    #[error("maximum gas fee below base fee")]
    MaximumGasFeeBelowBaseFee,
}