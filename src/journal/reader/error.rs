pub enum LineType {
    Unknown,
    Comment,
    IncludeDirective,
    TransactionHeader,
    PeriodidTransactionHeader,
    Posting,
}

impl std::fmt::Display for LineType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LineType::Unknown => write!(f, "an unknown line type"),
            LineType::Comment => write!(f, "a comment"),
            LineType::IncludeDirective => write!(f, "an include directive"),
            LineType::TransactionHeader => write!(f, "a transaction header"),
            LineType::PeriodidTransactionHeader => write!(f, "a periodic transaction Header"),
            LineType::Posting => write!(f, "a posting"),
        }
    }
}

pub enum ReaderError {
    UnexpectedItem(LineType, u64),
    MissingPosting(u64),
    MissingTransaction(u64),
    TwoPostingsWithElidedAmounts(u64),
    TransactionDoesNotBalance(u64),
    IO(std::io::Error, u64),
    Parse(LineType, u64),
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
            ReaderError::IO(e, line) => {
                write!(f, "An IO error occurred on line {}: {:?}", line, e)
            }
            ReaderError::Parse(item, line) => {
                write!(f, "Failed to parse {} on line {}", item, line)
            }
        }
    }
}
