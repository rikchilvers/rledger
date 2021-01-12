use super::{
    comment::*, include::include, posting::posting, transaction_header::transaction_header,
};
use crate::journal::{posting::Posting, transaction::Transaction};
use std::cell::RefCell;
use std::io::BufRead;
use std::rc::Rc;

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

pub struct Reader<'a> {
    state: ReaderState,
    line_number: u64,
    current_transaction: Option<Rc<RefCell<Transaction>>>,
    current_posting: Option<Posting>,
    pub transaction_handler: Box<dyn FnMut() + 'a>,
}

impl<'a> Reader<'a> {
    pub fn new(handler: Box<dyn FnMut() + 'a>) -> Self {
        Self {
            state: ReaderState::None,
            line_number: 0,
            current_transaction: None,
            current_posting: None,
            transaction_handler: handler,
        }
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
                t.borrow_mut().close();
                (self.transaction_handler)();
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
            if let Some(ref mut t) = &mut self.current_transaction {
                t.borrow_mut().close();
            }
            self.current_transaction = None;

            self.state = ReaderState::InTransaction;
            self.current_transaction = Some(Rc::new(RefCell::new(Transaction::from_header(t))));

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
                        transaction.borrow_mut().add_comment(c.to_owned());
                    // transaction.add_comment(c.to_owned())
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
        if let Some(mut current_posting) = self.current_posting.take() {
            if let Some(t) = &mut self.current_transaction {
                current_posting.transaction = Some(Rc::downgrade(t));
                t.borrow_mut().add_posting(current_posting);
                return true;
            } else {
                println!("no transaction to add posting to");
                return false;
            }
        }

        return true;
    }
}
