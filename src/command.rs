use crate::journal::Transaction;
use crate::reader::error::Error;

use std::sync::Arc;

pub trait Command {
    fn read_transactions<I>(&mut self, reader: I) -> Result<(), Error>
    where
        I: IntoIterator<Item = Result<Arc<Transaction>, Error>>;

    fn report(&self);
}
