#[derive(Debug, Clone, Eq, PartialEq)]
// TODO rename TransactionStatus ?
pub enum Status {
    // TODO: change to None
    NoStatus,
    Cleared,
    Uncleared,
}

impl Default for Status {
    fn default() -> Self {
        Status::NoStatus
    }
}

impl std::fmt::Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Status::NoStatus => write!(f, " "),
            Status::Cleared => write!(f, "*"),
            Status::Uncleared => write!(f, "!"),
        }
    }
}
