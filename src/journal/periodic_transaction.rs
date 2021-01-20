use super::Transaction;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PeriodInterval {
    Budget,
    Daily,
    Weekly,
    Monthly,
    Quarterly,
    Yearly,
}

#[derive(Debug)]
pub struct Period {
    start_date: time::Date,
    end_date: time::Date,
    interval: PeriodInterval,
    frequency: u8,
}

#[derive(Debug)]
pub struct PeriodicTransaction {
    period: Period,
    transaction: Rc<RefCell<Transaction>>,
}

impl PeriodicTransaction {
    pub fn run(&self, start_date: time::Date) -> Vec<Rc<RefCell<Transaction>>> {
        match self.period.interval {
            PeriodInterval::Budget => {
                self.transaction.borrow_mut().date = start_date;
                return vec![Rc::clone(&self.transaction)];
            }
            _ => return vec![],
        }
    }
}
