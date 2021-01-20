use crate::journal::ReaderError;
use crate::journal::Transaction;
use std::cell::RefCell;
use std::rc::Rc;

pub trait Command {
    fn read_transactions<I>(&mut self, reader: I) -> Result<(), ReaderError>
    where
        I: IntoIterator<Item = Result<Rc<RefCell<Transaction>>, ReaderError>>;

    fn report(&self);
}
