use crate::command::Command;

use journal::Transaction;
use reader::Error;

use std::rc::Rc;

pub struct Statistics {
    transaction_count: usize,
}

impl Statistics {
    pub fn new() -> Self {
        Self { transaction_count: 0 }
    }
}

impl Command for Statistics {
    fn read_transactions<I>(&mut self, reader: I) -> Result<(), Error>
    where
        I: IntoIterator<Item = Result<Rc<Transaction>, Error>>,
    {
        for item in reader {
            match item {
                Err(e) => return Err(e),
                Ok(_) => self.transaction_count += 1,
            }
        }

        Ok(())
    }

    fn report(&self) {
        println!("Transactions:\t{} (X.X per day)", self.transaction_count);
    }
}
