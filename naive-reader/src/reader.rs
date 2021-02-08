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
    pub fn read(&mut self, location: String) -> Result<Vec<Arc<Transaction>>, Error> {
        let (send, recv) = mpsc::channel();

        thread::spawn(move || {
            let mut source = Source::new(&location);
            source.parse(send);
        });

        let mut transactions = Vec::with_capacity(2046);

        for t in recv {
            match t {
                Err(e) => return Err(e),
                Ok(r) => match r {
                    ParseResult::Transaction(t) => transactions.push(Arc::clone(&t)),
                    ParseResult::IncludeDirective(include) => {
                        let to_insert = Arc::clone(&include);
                        if !self.visited_sources.insert(to_insert) {
                            return Err(Error::DuplicateSource(include));
                        }
                    }
                    _ => {}
                },
            }
        }

        &transactions.par_sort_unstable();

        return Ok(transactions);
    }
}
