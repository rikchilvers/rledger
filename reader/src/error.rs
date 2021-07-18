use std::path::PathBuf;

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

/// Packages an ErrorKind with location information
pub struct Error {
    /// The kind of error encountered
    pub kind: ErrorKind,
    /// The path of the file where the error was detected
    pub location: PathBuf,
    /// One-based line number where the error was detected
    pub line: u64,
}

/// Indicates an error during reading of a journal file
pub enum ErrorKind {
    IncorrectFormatting(String),
    DuplicateSource(PathBuf),
    UnexpectedItem(LineType),
    MissingPosting,
    MissingTransaction,
    TwoPostingsWithElidedAmounts,
    TransactionDoesNotBalance,
    IO(std::io::Error),
    Parse(LineType),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.kind {
            ErrorKind::UnexpectedItem(item) => {
                write!(f, "Unexpected {} on line {}", item, self.line)
            }
            ErrorKind::MissingPosting => {
                write!(f, "Missing posting on line {}", self.line)
            }
            ErrorKind::MissingTransaction => {
                write!(f, "Missing transaction on line {}", self.line)
            }
            ErrorKind::TwoPostingsWithElidedAmounts => {
                write!(f, "Two postings with elided amounts on line {}", self.line)
            }
            ErrorKind::TransactionDoesNotBalance => {
                write!(f, "Transaction ending on line {} does not balance.", self.line)
            }
            ErrorKind::IO(e) => {
                write!(f, "An IO error occurred on line {}: {:?}", self.line, e)
            }
            ErrorKind::Parse(item) => {
                write!(f, "Failed to parse {} at {:?}:{}", item, self.location, self.line)
            }
            ErrorKind::DuplicateSource(path) => {
                write!(f, "Found cyclic import of {:?}", path.as_path())
            }
            ErrorKind::IncorrectFormatting(desc) => {
                write!(f, "Incorrect formatting on line {}: {}", self.line, desc)
            }
        }
    }
}
