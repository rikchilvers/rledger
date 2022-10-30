use journal::Amount;
use journal::Date;
use reader::error::Error;
use reader::reader::{Config, Reader};
use std::collections::HashMap;

struct BudgetMonth {
    overspending: Amount,
    future: Amount,
}

pub struct Budget {
    months: HashMap<Date, BudgetMonth>,
}

impl Budget {
    pub fn new() -> Self {
        Self { months: HashMap::new() }
    }

    pub fn read(&mut self, file: String) -> Result<(), Error> {
        let mut reader = Reader::new();
        let config = Config::new();

        let (transactions, postings, _) = reader.read(file, config)?;
        // self.postings = postings;

        for tx in transactions {}

        // for posting in &self.postings {
        //     let mut path: Vec<&str> = posting.path.split(':').collect();
        // self.tree.add_path(&mut path);
        // }

        // self.tree.display(&None, |_| None);

        Ok(())
    }
}
