#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ReaderState {
    None,
    InTransaction,
    InPeriodicTransaction,
    InPosting,
}

impl Default for ReaderState {
    fn default() -> Self {
        Self::None
    }
}
