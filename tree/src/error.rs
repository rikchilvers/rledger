#[derive(Debug)]
pub enum Error {
    NodeOutOfBounds,
    NodeNotPresent,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::NodeOutOfBounds => write!(f, "node out of bounds"),
            Error::NodeNotPresent => write!(f, "node not present"),
        }
    }
}

impl std::error::Error for Error {}
