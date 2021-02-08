use std::path::PathBuf;
use std::sync::Arc;

/// Type of line found in a journal file
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
            LineType::Unknown => write!(f, "unknown line type"),
            LineType::Comment => write!(f, "comment"),
            LineType::IncludeDirective => write!(f, "include directive"),
            LineType::TransactionHeader => write!(f, "transaction header"),
            LineType::PeriodidTransactionHeader => write!(f, "periodic transaction header"),
            LineType::Posting => write!(f, "posting"),
        }
    }
}

/// Indicates an error during reading of a journal file
pub enum Error {
    IncorrectFormatting(String, u64),
    DuplicateSource(Arc<PathBuf>),
    UnexpectedItem(LineType, u64),
    MissingPosting(u64),
    MissingTransaction(u64),
    TwoPostingsWithElidedAmounts(u64),
    TransactionDoesNotBalance(u64),
    IO(std::io::Error, u64),
    Parse(LineType, u64),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::UnexpectedItem(item, line) => {
                write!(f, "Unexpected {} on line {}", item, line)
            }
            Error::MissingPosting(line) => {
                write!(f, "Missing posting on line {}", line)
            }
            Error::MissingTransaction(line) => {
                write!(f, "Missing transaction on line {}", line)
            }
            Error::TwoPostingsWithElidedAmounts(line) => {
                write!(f, "Two postings with elided amounts on line {}", line)
            }
            Error::TransactionDoesNotBalance(line) => {
                write!(f, "Transaction ending on line {} does not balance.", line)
            }
            Error::IO(e, line) => {
                write!(f, "An IO error occurred on line {}: {:?}", line, e)
            }
            Error::Parse(item, line) => {
                write!(f, "Failed to parse {} on line {}", item, line)
            }
            Error::DuplicateSource(path) => {
                write!(f, "Found cyclic import of {:?}", path.as_path())
            }
            Error::IncorrectFormatting(desc, line) => {
                write!(f, "Incorrect formatting on line {}: {}", line, desc)
            }
        }
    }
}
