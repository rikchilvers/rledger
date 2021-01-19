use super::source::ParseResult;
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
        match &mut self.sources.last_mut() {
            None => return None,
            Some(source) => match source.parse_line() {
                Err(e) => return Some(Err(e)),
                Ok(parse_result) => match parse_result {
                    ParseResult::SourceIncomplete => return self.next(),
                    ParseResult::SourceComplete => {
                        self.sources.pop();
                        return self.next();
                    }
                    ParseResult::Transaction(transaction) => return Some(Ok(transaction)),
                    ParseResult::IncludeDirective(path) => {
                        println!("would include file: {}\n", path);
                        // self.sources.push(Source::new(&path));
                        return self.next();
                    }
                },
            },
        }
    }
}

impl Reader {
    pub fn new(location: &str) -> Self {
        Self {
            sources: vec![Source::new(location)],
        }
    }
}
