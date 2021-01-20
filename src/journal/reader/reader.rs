use super::source::ParseResult;
use super::{error::ReaderError, source::Source};
use crate::journal::transaction::Transaction;
use std::cell::RefCell;
use std::path::PathBuf;
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
        if self.sources.len() == 0 {
            return None;
        }

        let mut source = self.sources.pop().unwrap();
        let parse_result = source.parse_line();
        match parse_result {
            Err(e) => return Some(Err(e)),
            Ok(parse_result) => match parse_result {
                ParseResult::SourceIncomplete => {
                    self.sources.push(source);
                    return self.next();
                }
                ParseResult::SourceComplete => {
                    self.sources.pop();
                    return self.next();
                }
                ParseResult::Transaction(transaction) => {
                    self.sources.push(source);
                    return Some(Ok(transaction));
                }
                ParseResult::IncludeDirective(path) => {
                    println!("would include file: {}\n", path);
                    match source.location.clone().parent() {
                        None => panic!("no parent"),
                        Some(parent) => {
                            self.sources.push(source);
                            self.sources.push(Source::new(parent.join(path)));
                        }
                    }
                    return self.next();
                }
            },
        }

        // return None;
        /*
        match self.sources.last_mut() {
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
                        match source.location.parent() {
                            None => panic!("no parent"),
                            Some(parent) => {
                                self.sources.push(Source::new(parent.join(path)));
                            }
                        }
                        return self.next();
                    }
                },
            },
        }
        */
    }
}

impl Reader {
    pub fn new(location: &str) -> Self {
        Self {
            sources: vec![Source::new(PathBuf::from(location))],
        }
    }
}
