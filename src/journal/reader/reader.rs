use super::{error::ReaderError, source::ParseResult, source::Source};
use crate::journal::transaction::Transaction;
use std::{cell::RefCell, collections::HashSet, path::PathBuf, rc::Rc};

#[derive(Debug, PartialEq, Eq)]
pub enum ReaderState {
    None,
    InTransaction,
    InPeriodicTransaction,
    InPosting,
}

impl Default for ReaderState {
    fn default() -> Self {
        Self::None
    }
}

pub struct Reader {
    sources: Vec<Source>,
    visited_sources: HashSet<String>,
}

impl Reader {
    pub fn new(location: &str) -> Self {
        let mut set = HashSet::new();
        set.insert(location.to_owned());
        Self {
            sources: vec![Source::new(PathBuf::from(location))],
            visited_sources: set,
        }
    }
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
                ParseResult::SourceComplete => {
                    self.sources.pop();
                    return self.next();
                }
                ParseResult::Transaction(transaction) => {
                    self.sources.push(source);
                    return Some(Ok(transaction));
                }
                ParseResult::IncludeDirective(path) => {
                    if !self.visited_sources.insert(path.clone()) {
                        panic!("visited {} already", path);
                    }
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
    }
}
