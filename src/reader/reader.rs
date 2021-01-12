use super::{
    comment::*, include::include, posting::posting, transaction_header::transaction_header,
};
use crate::journal::{posting::Posting, transaction::Transaction};
use std::io::BufRead;

#[derive(Debug, PartialEq, Eq)]
enum ReaderState {
    None,
    InTransaction,
    InPosting,
}

impl Default for ReaderState {
    fn default() -> Self {
        Self::None
    }
}

#[derive(Default)]
pub struct Reader {
    state: ReaderState,
    line_number: u64,
    current_transaction: Option<Transaction>,
    current_posting: Option<Posting>,
}

impl Reader {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn read(&mut self, path: &str) -> bool {
        let file = std::fs::File::open(path).expect(&format!("file not found: {}", path));
        let reader = std::io::BufReader::new(file);

        println!("Reading {}", path);

        for line in reader.lines() {
            self.line_number += 1;

            match line {
                Ok(l) => {
                    if !self.read_line(l) {
                        return false;
                    }
                }
                Err(e) => {
                    println!("{}", e);
                    return false;
                }
            }
        }

        println!("There were {} lines", self.line_number);

        return true;
    }

    fn read_line(&mut self, line: String) -> bool {
        if line.len() == 0 {
            self.add_posting();
            if let Some(t) = &mut self.current_transaction {
                t.close();
                println!("{}", t);
            }
            self.current_transaction = None;
            self.state = ReaderState::None;
            return true;
        }

        if let Ok((_, include)) = include(&line) {
            println!(">> include: {}", include);
            return true;
        }

        if let Ok((_, t)) = transaction_header(&line) {
            if self.state != ReaderState::None {
                println!("unexpected transaction header");
                return false;
            }

            self.add_posting();
            if let Some(t) = &mut self.current_transaction {
                t.close();
                println!("{}", t);
            }
            self.current_transaction = None;

            self.state = ReaderState::InTransaction;
            self.current_transaction = Some(Transaction::from_header(t));

            return true;
        }

        // Currently, this must come before postings because that lexer will match comments greedily
        if let Ok((_, c)) = comment_min(2, &line) {
            match &mut self.state {
                ReaderState::InPosting => {
                    if let Some(p) = &mut self.current_posting {
                        p.add_comment(c.to_owned());
                    }
                }
                ReaderState::InTransaction => {
                    if let Some(transaction) = &mut self.current_transaction {
                        transaction.add_comment(c.to_owned())
                    } else {
                        println!("couldn't add transaction comment");
                    }
                }
                _ => {
                    println!("unexpected comment");
                    return false;
                }
            }

            return true;
        }

        if let Ok((_, posting)) = posting(&line) {
            if self.state == ReaderState::None {
                println!("unexpected posting");
                return false;
            }

            // If we're already in a posting, we need to add it to the current transaction
            self.add_posting();

            self.state = ReaderState::InPosting;
            self.current_posting = Some(posting);
        }

        true
    }

    fn add_posting(&mut self) -> bool {
        if let Some(current_posting) = self.current_posting.take() {
            if let Some(t) = &mut self.current_transaction {
                t.add_posting(current_posting);
                return true;
            } else {
                println!("no transaction to add posting to");
                return false;
            }
        }

        return true;
    }
}
