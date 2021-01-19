pub enum ReaderError {
    UnexpectedItem(String, u64),
    MissingPosting(u64),
    MissingTransaction(u64),
    TwoPostingsWithElidedAmounts(u64),
    TransactionDoesNotBalance(u64),
    General,
    // TODO: remove this
    IO(std::io::Error),
}

impl ReaderError {
    pub fn change_line_number(&self, new: u64) -> Self {
        match self {
            Self::UnexpectedItem(item, _) => ReaderError::UnexpectedItem(item.to_owned(), new),
            Self::MissingPosting(_) => Self::MissingPosting(new),
            Self::MissingTransaction(_) => Self::MissingTransaction(new),
            Self::TwoPostingsWithElidedAmounts(_) => Self::TwoPostingsWithElidedAmounts(new),
            Self::TransactionDoesNotBalance(_) => Self::TransactionDoesNotBalance(new),
            Self::General => Self::General,
            Self::IO(_) => Self::General,
        }
    }
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
            ReaderError::General => {
                write!(f, "An error occurred")
            }
            ReaderError::IO(e) => {
                write!(f, "An IO error occurred: {:?}", e)
            }
        }
    }
}
