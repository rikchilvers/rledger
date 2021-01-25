#[derive(Debug, Clone, Eq, PartialEq)]
pub enum TransactionStatus {
    NoStatus,
    Cleared,
    Uncleared,
}

impl Default for TransactionStatus {
    fn default() -> Self {
        TransactionStatus::NoStatus
    }
}

impl std::fmt::Display for TransactionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TransactionStatus::NoStatus => write!(f, " "),
            TransactionStatus::Cleared => write!(f, "*"),
            TransactionStatus::Uncleared => write!(f, "!"),
        }
    }
}
