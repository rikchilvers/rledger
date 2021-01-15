use super::reader::error::ReaderError;

pub enum Error {
    Reader(ReaderError),
    // TODO this should indicate the line it's on - maybe the reader should send this too?
    TransactionDoesNotBalance,
}
