use crate::command::Command;

use journal::Transaction;
use reader::error::Error;
use reader::Date;

use std::sync::Arc;

pub struct Statistics {
    start_date: Date,
    end_date: Date,
    transaction_count: usize,
}

impl Statistics {
    pub fn new() -> Self {
        Self {
            start_date: Date::try_from_ymd(100000, 1, 1).unwrap(),
            end_date: Date::try_from_ymd(-100000, 1, 1).unwrap(),
            transaction_count: 0,
        }
    }

    fn process_transaction(&mut self, transaction: Arc<Transaction>) {
        self.transaction_count += 1;

        if transaction.date.lt(&self.start_date) {
            self.start_date = transaction.date;
        }

        if transaction.date.gt(&self.end_date) {
            self.end_date = transaction.date;
        }
    }
}

impl Command for Statistics {
    fn read_transactions<I>(&mut self, reader: I) -> Result<(), Error>
    where
        I: IntoIterator<Item = Result<Arc<Transaction>, Error>>,
    {
        for item in reader {
            match item {
                Err(e) => return Err(e),
                Ok(transaction) => self.process_transaction(transaction),
            }
        }

        Ok(())
    }

    fn report(&self) {
        println!("First transaction:\t{} (X time ago)", self.start_date);
        println!("Last transaction:\t{} (X time ago)", self.end_date);
        println!("Time period:\t\tXXXX days");
        println!("Transactions:\t\t{} (X.X per day)", self.transaction_count);
    }
}
