use crate::journal::transaction::Transaction;
use std::cell::RefCell;
use std::rc::Rc;

pub trait Command {
    fn handle_transaction(&mut self, transaction: Rc<RefCell<Transaction>>);
    fn handle_posting(&mut self);
    fn report(&self);
}

pub struct Printer {
    transactions: Vec<Rc<RefCell<Transaction>>>,
}

impl Printer {
    pub fn new() -> Self {
        Self {
            transactions: vec![],
        }
    }
}

impl Command for Printer {
    fn handle_transaction(&mut self, transaction: Rc<RefCell<Transaction>>) {
        println!("printer handle transaction")
    }
    fn handle_posting(&mut self) {
        println!("printer handle posting")
    }
    fn report(&self) {
        println!("printer report")
    }
}
