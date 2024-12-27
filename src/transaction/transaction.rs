use crate::transaction::transaction_eip1559::TransactionEip1559;

pub enum Transaction {
    Eip1559(TransactionEip1559),
}