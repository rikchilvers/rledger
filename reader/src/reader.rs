use super::{error::Error, source::ParseResult, source::Source};

use journal::Transaction;

use std::{collections::HashSet, path::PathBuf, sync::Arc};

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
    type Item = Result<Arc<Transaction>, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.sources.len() == 0 {
            return None;
        }

        let mut source = self.sources.pop().unwrap();
        match source.parse_line() {
            Err(e) => return Some(Err(e)),
            Ok(parse_result) => match parse_result {
                ParseResult::SourceComplete => {
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
