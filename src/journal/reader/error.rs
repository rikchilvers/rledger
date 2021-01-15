pub enum ReaderError {
    UnexpectedItem(String, u64),
    MissingPosting(u64),
    MissingTransaction(u64),
    TwoPostingsWithElidedAmounts(u64),
    TransactionDoesNotBalance(u64),
}

impl std::fmt::Display for ReaderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReaderError::UnexpectedItem(item, line) => {
                write!(f, "Unexpected {} on line {}", item, line)
            }
            ReaderError::MissingPosting(line) => write!(f, "Missing posting on line {}", line),
            ReaderError::MissingTransaction(line) => {
                write!(f, "Missing transaction on line {}", line)
            }
            ReaderError::TwoPostingsWithElidedAmounts(line) => {
                write!(f, "Two postings with elided amounts on line {}", line)
            }
            ReaderError::TransactionDoesNotBalance(line) => {
                write!(f, "Transaction ending on line {} does not balance.", line)
            }
        }
    }
}
