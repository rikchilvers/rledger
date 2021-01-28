use crate::command::Command;

use journal::Transaction;
use reader::Error;

use std::rc::Rc;

pub struct Printer {
    transactions: Vec<Rc<Transaction>>,
}

impl Printer {
    pub fn new() -> Self {
        Self { transactions: vec![] }
    }
}

impl Command for Printer {
    fn read_transactions<I>(&mut self, reader: I) -> Result<(), Error>
    where
        I: IntoIterator<Item = Result<Rc<Transaction>, Error>>,
    {
        for item in reader {
            match item {
                Err(e) => return Err(e),
                Ok(transaction) => self.transactions.push(transaction),
            }
        }

        Ok(())
    }

    fn report(&self) {
        for transaction in self.transactions.iter() {
            println!("{}", transaction);
        }
    }
}
