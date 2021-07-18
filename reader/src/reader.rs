use super::{error::Error, error::ErrorKind, source::ItemKind, source::Source};

use journal::Posting;
use journal::Transaction;

use std::sync::mpsc;
use std::thread;
use std::{collections::HashSet, path::PathBuf};

use rayon::prelude::*;

const TRANSACTION_COUNT: usize = 1024;
const POSTING_COUNT: usize = TRANSACTION_COUNT * 2;

pub struct Config {
    pub should_sort: bool,
    pub read_postings: bool,
    pub read_transactions: bool,
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

/// Reader reads ledger files and returns the transactions, postings and included file paths it found
pub struct Reader {}

impl Reader {
    pub fn new() -> Self {
        Self {}
    }

    pub fn read<P: Into<PathBuf>>(
        &mut self,
        location: P,
        config: Config,
    ) -> Result<(Vec<Transaction>, Vec<Posting>, HashSet<PathBuf>), Error> {
        let location = location.into();
        let (send, recv) = mpsc::channel();

        let source_location = location.clone();
        thread::spawn(move || {
            let mut source = Source::new(source_location);
            source.parse(send);
        });

        // The Reader caches the txs and postings so that consumers don't have to worry
        // about errors occuring mid-stream.
        let mut transactions = Vec::with_capacity(if config.read_transactions { TRANSACTION_COUNT } else { 0 });
        let mut postings = Vec::with_capacity(if config.read_postings { POSTING_COUNT } else { 0 });
        let mut visited_sources = HashSet::new(); // HashSet<Arc<PathBuf>>,
        visited_sources.insert(location.into());

        for t in recv {
            match t {
                Err(e) => return Err(e),
                Ok(r) => match r.kind {
                    ItemKind::Transaction(mut t, mut p) => {
                        // Link the postings and txs by idx
                        let t_index = transactions.len();
                        let p_indices = (postings.len()..postings.len() + p.len()).collect();

                        if config.read_transactions {
                            if config.read_postings {
                                t.postings = p_indices;
                            }
                            transactions.push(t);
                        }

                        if config.read_postings {
                            if config.read_transactions && !config.should_sort {
                                for p in &mut p {
                                    p.transaction = Some(t_index);
                                }
                            }
                            postings.append(&mut p);
                        }
                    }
                    ItemKind::IncludeDirective(include) => {
                        if !visited_sources.insert(include.clone()) {
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
            for (t_idx, t) in transactions.iter().enumerate() {
                for p_idx in t.postings.iter() {
                    postings[*p_idx].transaction = Some(t_idx)
                }
            }
        }

        return Ok((transactions, postings, visited_sources));
    }
}
