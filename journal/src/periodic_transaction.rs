use super::Transaction;
use std::sync::Arc;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PeriodInterval {
    Budget,
    Daily,
    Weekly,
    Monthly,
    Quarterly,
    Yearly,
}

#[derive(Default, Debug, PartialEq, Eq)]
pub struct Period {
    // TODO: which of these should be optional?
    pub start_date: Option<time::Date>,
    pub end_date: Option<time::Date>,
    pub interval: Option<PeriodInterval>,
    pub frequency: u8,
}

#[derive(Debug)]
pub struct PeriodicTransaction {
    period: Period,
    transaction: Arc<Transaction>,
}

impl PeriodicTransaction {
    pub fn run(&self, start_date: time::Date) -> Vec<Arc<Transaction>> {
        unimplemented!();
        /*
        match self.period.interval {
            PeriodInterval::Budget => {
                self.transaction.borrow_mut().date = start_date;
                return vec![Arc::clone(&self.transaction)];
            }
            _ => return vec![],
        }
        */
    }
}
