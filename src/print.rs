use crate::command::Command;
use crate::journal::transaction::Transaction;
use std::cell::RefCell;
use std::rc::Rc;

pub struct Printer {
    transactions: Vec<Rc<RefCell<Transaction>>>,
}

impl Printer {
    pub fn new() -> Self {
        Self {
            transactions: vec![],
        }
    }

    pub fn report_take(&self, transactions: &Vec<Rc<RefCell<Transaction>>>) {
        for transaction in transactions.iter() {
            println!("{}", transaction.borrow());
        }
    }
}

impl Command for Printer {
    fn handle_transaction(&mut self, transaction: Rc<RefCell<Transaction>>) {
        // self.transactions.push(transaction);
    }

    fn report(&self) {
        for transaction in self.transactions.iter() {
            println!("{}", transaction.borrow());
        }
    }
}
