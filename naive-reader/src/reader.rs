use super::{error::Error, error::ErrorKind, source::ItemKind, source::Source};

use journal::Posting;
use journal::Transaction;

use std::sync::mpsc;
use std::thread;
use std::{collections::HashSet, path::PathBuf, sync::Arc};

use rayon::prelude::*;

const TRANSACTION_COUNT: usize = 1024;
const POSTING_COUNT: usize = TRANSACTION_COUNT * 2;

pub struct Config {
    should_sort: bool,
    read_postings: bool,
    read_transactions: bool,
}

impl Config {
    pub fn new() -> Self {
        Self {
            should_sort: false,
            read_postings: true,
            read_transactions: true,
        }
    }
}

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
    pub fn read(&mut self, location: String, config: Config) -> Result<(Vec<Transaction>, Vec<Posting>), Error> {
        let (send, recv) = mpsc::channel();

        thread::spawn(move || {
            let mut source = Source::new(&location);
            source.parse(send);
        });

        // The Reader caches the txs and postings so that consumers don't have to worry
        // about errors occuring mid-stream.
        let mut transactions = Vec::with_capacity(if config.read_transactions { TRANSACTION_COUNT } else { 0 });
        let mut postings = Vec::with_capacity(if config.read_postings { POSTING_COUNT } else { 0 });

        for t in recv {
            match t {
                Err(e) => return Err(e),
                Ok(r) => match r.kind {
                    ItemKind::Transaction(t, mut p) => {
                        if config.read_transactions {
                            transactions.push(t);
                        }
                        if config.read_postings {
                            postings.append(&mut p);
                        }
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

        if config.should_sort {
            &transactions.par_sort_unstable();
        }

        return Ok((transactions, postings));
    }
}
