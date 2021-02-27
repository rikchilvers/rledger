use journal::Transaction;
use reader::error::Error;

use reader::reader::{Config, Reader};

pub struct Printer {}

impl Printer {
    pub fn new() -> Self {
        Self {}
    }

    pub fn read(&mut self, file: String) -> Result<(), Error> {
        let mut reader = Reader::new();
        let mut config = Config::new();
        config.should_sort = true;

        let (transactions, postings) = reader.read(file, config)?;

        for transaction in transactions {
            transaction.display(&postings);
        }

        Ok(())
    }
}
