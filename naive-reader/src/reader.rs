use super::{error::Error, error::ErrorKind, source::ItemKind, source::Source};

use journal::Posting;
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
    pub fn read(&mut self, location: String) -> Result<(Vec<Transaction>, Vec<Posting>), Error> {
        let (send, recv) = mpsc::channel();

        thread::spawn(move || {
            let mut source = Source::new(&location);
            source.parse(send);
        });

        let mut transactions = Vec::with_capacity(2046);
        let mut postings = Vec::with_capacity(2046);

        for t in recv {
            match t {
                Err(e) => return Err(e),
                Ok(r) => match r.kind {
                    ItemKind::Transaction(t, mut p) => {
                        transactions.push(t);
                        postings.append(&mut p);
                    }
                    ItemKind::IncludeDirective(include) => {
                        let to_insert = Arc::clone(&include);
                        if !self.visited_sources.insert(to_insert) {
                            let error = Error {
                                kind: ErrorKind::DuplicateSource(include),
                                line: 0,
                                location: r.location,
                            };
                            return Err(error);
                        }
                    }
                    _ => {}
                },
            }
        }

        // TODO add a flag for this
        &transactions.par_sort_unstable();

        return Ok((transactions, postings));
    }
}
