use super::{error::Error, source::ParseResult, source::Source};

use journal::Transaction;

use std::sync::mpsc;
use std::thread;
use std::{collections::HashSet, path::PathBuf, sync::Arc};

use rayon::prelude::*;

pub struct Reader {
    visited_sources: HashSet<Arc<PathBuf>>,
}

impl Reader {
    pub fn new() -> Self {
        Self {
            visited_sources: HashSet::new(),
        }
    }

    // TODO: make this take things that are into<pathbuf>
    pub fn read(&mut self, location: String) -> Vec<Arc<Transaction>> {
        let (send, recv) = mpsc::channel();
        let mut transactions = Vec::with_capacity(2046);

        thread::spawn(move || {
            let mut source = Source::new(PathBuf::from(location));
            source.parse(send);
        });

        for t in recv {
            match t {
                Err(_) => panic!("err"),
                Ok(r) => match r {
                    ParseResult::Transaction(t) => transactions.push(Arc::clone(&t)),
                    ParseResult::IncludeDirective(l) => {
                        if !self.visited_sources.insert(l) {
                            panic!("already had that file")
                        }
                    }
                    _ => {}
                },
            }
        }

        &transactions.par_sort_unstable();

        return transactions;
    }
}

impl Iterator for Reader {
    type Item = Result<Arc<Transaction>, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        return None;
    }
}
