use journal::Transaction;
use reader::error::Error;
use reader::Date;

use reader::reader::{Config, Reader};

pub struct Statistics {
    start_date: Date,
    end_date: Date,
    transaction_count: usize,
    posting_count: usize,
}

impl Statistics {
    pub fn new() -> Self {
        Self {
            start_date: Date::try_from_ymd(100000, 1, 1).unwrap(),
            end_date: Date::try_from_ymd(-100000, 1, 1).unwrap(),
            transaction_count: 0,
            posting_count: 0,
        }
    }

    pub fn read(&mut self, file: String) -> Result<(), Error> {
        let mut reader = Reader::new();
        let mut config = Config::new();

        let (transactions, postings) = reader.read(file, config)?;

        for t in transactions {
            self.process_transaction(t);
        }

        for p in postings {
            self.posting_count += 1;
        }

        self.report();

        Ok(())
    }

    fn process_transaction(&mut self, transaction: Transaction) {
        self.transaction_count += 1;

        if transaction.date.lt(&self.start_date) {
            self.start_date = transaction.date;
        }

        if transaction.date.gt(&self.end_date) {
            self.end_date = transaction.date;
        }
    }

    fn report(&self) {
        println!("First transaction:\t{} (X time ago)", self.start_date);
        println!("Last transaction:\t{} (X time ago)", self.end_date);
        println!("Time period:\t\tXXXX days");
        println!("Transactions:\t\t{} (X.X per day)", self.transaction_count);
        println!("Postings:\t\t{}", self.posting_count);
    }
}
