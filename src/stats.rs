use std::collections::HashSet;
use std::path::PathBuf;

use journal::Transaction;
use reader::error::Error;
use reader::Date;

use reader::reader::{Config, Reader};

pub struct Statistics {
    start_date: Date,
    end_date: Date,
    sources: HashSet<PathBuf>,
    transaction_count: usize,
    posting_count: usize,
    unique_accounts: HashSet<String>,
    unique_payees: HashSet<String>,
}

impl Statistics {
    pub fn new() -> Self {
        Self {
            start_date: Date::try_from_ymd(100000, 1, 1).unwrap(),
            end_date: Date::try_from_ymd(-100000, 1, 1).unwrap(),
            sources: HashSet::new(),
            transaction_count: 0,
            posting_count: 0,
            unique_accounts: HashSet::new(),
            unique_payees: HashSet::new(),
        }
    }

    pub fn read(&mut self, file: String) -> Result<(), Error> {
        let mut reader = Reader::new();
        let config = Config::new();

        let (transactions, postings, sources) = reader.read(file, config)?;

        self.sources = sources;

        for t in transactions {
            self.process_transaction(&t);
            self.unique_payees.insert(t.payee);
        }

        for p in postings {
            self.posting_count += 1;
            self.unique_accounts.insert(p.path);
        }

        self.report();

        Ok(())
    }

    fn process_transaction(&mut self, transaction: &Transaction) {
        self.transaction_count += 1;

        if transaction.date.lt(&self.start_date) {
            self.start_date = transaction.date;
        }

        if transaction.date.gt(&self.end_date) {
            self.end_date = transaction.date;
        }
    }

    fn report(&self) {
        let days = (self.end_date - self.start_date).as_seconds_f64() / 60. / 60. / 24.;
        let txs_per_day = (self.transaction_count as f64) / days;

        println!("Transactions found in {} files", self.sources.len());
        let mut sources: Vec<&PathBuf> = self.sources.iter().collect();
        sources.sort();
        for s in sources {
            println!("  {:?}", s)
        }
        println!("First transaction:\t{}", self.start_date);
        println!("Last transaction:\t{}", self.end_date);
        println!("Time period:\t\t{:.0} days", days);
        println!(
            "Transactions:\t\t{} ({:.1} per day)",
            self.transaction_count, txs_per_day
        );
        println!("Postings:\t\t{}", self.posting_count);
        println!("Unique accounts:\t{}", self.unique_accounts.len());
        println!("Unique payees:\t\t{}", self.unique_payees.len());
    }
}
