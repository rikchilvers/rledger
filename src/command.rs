use crate::journal::Transaction;
use crate::reader::Error;
use std::cell::RefCell;
use std::rc::Rc;

pub trait Command {
    fn read_transactions<I>(&mut self, reader: I) -> Result<(), Error>
    where
        I: IntoIterator<Item = Result<Rc<RefCell<Transaction>>, Error>>;

    fn report(&self);
}
