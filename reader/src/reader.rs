use super::{error::Error, source::ParseResult, source::Source};

use journal::Transaction;

use std::{collections::HashSet, path::PathBuf, sync::Arc};

use std::sync::mpsc;
use std::thread;

pub struct Reader {
    handles: Vec<thread::JoinHandle<()>>,
    visited_sources: HashSet<String>,
    // receiver: mpsc::Receiver<Result<ParseResult, Error>>,
    // sender: mpsc::Sender<Result<ParseResult, Error>>,
}

impl Reader {
    pub fn new() -> Self {
        Self {
            handles: vec![],
            visited_sources: HashSet::new(),
            // receiver: recv,
            // sender: send,
        }
    }

    pub fn go(&mut self, location: &str) {
        use rayon::prelude::*;

        self.visited_sources.insert(location.to_owned());

        let (send, recv) = mpsc::channel();

        let location = location.to_owned();
        // let send1 = send.clone();
        thread::spawn(move || {
            let mut source = Source::new(PathBuf::from(location));
            source.parse(send);
        });

        let mut transactions = vec![];
        for t in recv {
            match t {
                Err(_) => panic!("err"),
                Ok(r) => match r {
                    ParseResult::SourceComplete => {}
                    ParseResult::IncludeDirective(_) => {}
                    ParseResult::Transaction(t) => {
                        transactions.push(Arc::clone(&t));
                    }
                },
            }
        }

        &transactions.par_sort_unstable();

        for t in &transactions {
            println!("{}", t);
        }
        println!("got {} transactions", transactions.len());
    }
}

impl Iterator for Reader {
    type Item = Result<Arc<Transaction>, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        return None;
    }
}

/*
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
                            self.sources.push(Source::new(parent.join(path), self.sender.clone()));
                        }
                    }
                    return self.next();
                }
            },
        }
    }
}
*/
