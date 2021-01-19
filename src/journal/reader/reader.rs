use super::{error::ReaderError, source::Source};
use crate::journal::transaction::Transaction;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug, PartialEq, Eq)]
pub enum ReaderState {
    None,
    InTransaction,
    InPosting,
}

impl Default for ReaderState {
    fn default() -> Self {
        Self::None
    }
}

pub struct Reader {
    sources: Vec<Source>,
}

impl Iterator for Reader {
    type Item = Result<Rc<RefCell<Transaction>>, ReaderError>;

    fn next(&mut self) -> Option<Self::Item> {
        return None;

        /*
        let mut source = self.sources.last().unwrap();

        match source.parse_line(&self.state) {
            None => match self.sources.pop() {
                None => return None,
                Some(source) => {
                    self.current_source = source;
                    return self.next();
                }
            },
            Some(line) => match line {
                Err(e) => return Some(Err(e)),
                Ok(line) => match self.process_line(line) {
                    Err(e) => return Some(Err(e)),
                    Ok(transaction) => match transaction {
                        None => return self.next(),
                        Some(transaction) => return Some(Ok(transaction)),
                    },
                },
            },
        }
        */
    }
}

impl Reader {
    pub fn new(location: &str) -> Self {
        Self {
            sources: vec![Source::new(location)],
        }
    }
}
