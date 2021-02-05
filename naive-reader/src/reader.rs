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

        thread::spawn(move || {
            let mut source = Source::new(&location);
            source.parse(send);
        });

        let mut transactions = Vec::with_capacity(2046);

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
