use super::transaction::Transaction;
use super::transaction_status::TransactionStatus;

#[derive(Debug, Eq, PartialEq)]
pub struct TransactionHeader {
    pub date: time::Date,
    pub status: TransactionStatus,
    pub payee: String,
    pub comment: Option<String>,
}

impl Transaction {
    pub fn from_header(header: TransactionHeader) -> Self {
        Transaction {
            date: header.date,
            status: header.status,
            payee: header.payee,
            postings: vec![],
            comments: vec![],
            elided_amount_posting_index: None,
        }
    }
}
