use journal::transaction::Status;
use journal::Transaction;

#[derive(Debug, Eq, PartialEq)]
pub struct TransactionHeader {
    pub date: time::Date,
    pub status: Status,
    // TODO this should be optional
    pub payee: String,
    pub comment: Option<String>,
}
