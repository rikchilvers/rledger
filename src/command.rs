use crate::journal::transaction::Transaction;
use std::cell::RefCell;
use std::rc::Rc;

pub trait Command {
    fn handle_transaction(&mut self, transaction: Rc<RefCell<Transaction>>);
    fn report(&self);
}
