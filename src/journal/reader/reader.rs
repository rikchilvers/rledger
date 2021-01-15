use super::{
    comment::*, error::ReaderError, include::include, posting::posting,
    transaction_header::transaction_header,
};
use crate::journal::{posting::Posting, transaction::Transaction};
use std::cell::RefCell;
use std::io::BufRead;
use std::io::Lines;
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

pub struct Reader {
    state: ReaderState,
    line_number: u64,
    lines: Lines<std::io::BufReader<std::fs::File>>,

    current_transaction: Option<Rc<RefCell<Transaction>>>,
    current_posting: Option<Posting>,
}

impl Iterator for Reader {
    type Item = Rc<RefCell<Transaction>>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.lines.next() {
            None => return None,
            Some(line) => match line {
                Ok(line) => {
                    self.line_number += 1;
                    match self.read_line(&line) {
                        Ok(transaction) => match transaction {
                            None => return self.next(),
                            Some(transaction) => return Some(transaction),
                        },
                        Err(e) => {
                            println!("{}", e);
                            return None;
                        }
                    }
                }
                Err(e) => {
                    println!("{}", e);
                    return None;
                }
            },
        }
    }
}

impl Reader {
    pub fn new(path: &str) -> Self {
        let file = std::fs::File::open(path).expect(&format!("file not found: {}", path));
        let reader = std::io::BufReader::new(file);

        Self {
            state: ReaderState::None,
            line_number: 0,
            lines: reader.lines(),
            current_transaction: None,
            current_posting: None,
        }
    }

    fn read_line(&mut self, line: &str) -> Result<Option<Rc<RefCell<Transaction>>>, ReaderError> {
        let mut completed_transaction: Option<Rc<RefCell<Transaction>>> = None;

        if line.len() == 0 {
            self.add_posting()?;
            // We can take here as we want the transaction to be empty afterwards
            if let Some(t) = &mut self.current_transaction.take() {
                t.borrow_mut().close();
                completed_transaction = Some(t.clone());
            }
            self.state = ReaderState::None;
            return Ok(completed_transaction);
        }

        // Check for include directive
        if let Ok((_, include)) = include(&line) {
            println!(">> include: {}", include);
            return Ok(completed_transaction);
        }

        // Check for transaction header
        if let Ok((_, t)) = transaction_header(&line) {
            if self.state != ReaderState::None {
                return Err(ReaderError::UnexpectedItem(
                    "transaction header".to_owned(),
                    self.line_number,
                ));
            }

            // We might have just read a transaction, so add that to the previous transaction
            self.add_posting()?;

            // If had a previous transaction, we need to close it now we're starting a new one
            if let Some(ref t) = self.current_transaction {
                t.borrow_mut().close();
                completed_transaction = Some(t.clone());
            }

            self.state = ReaderState::InTransaction;
            self.current_transaction = Some(Rc::new(RefCell::new(Transaction::from_header(t))));

            return Ok(completed_transaction);
        }

        // Check for comments
        // This must come before postings because that lexer will match comments greedily
        if let Ok((_, c)) = comment_min(2, &line) {
            match self.state {
                ReaderState::InPosting => match self.current_posting {
                    Some(ref mut p) => p.add_comment(c.to_owned()),
                    None => return Err(ReaderError::MissingPosting(self.line_number)),
                },
                ReaderState::InTransaction => match self.current_transaction {
                    Some(ref mut transaction) => {
                        transaction.borrow_mut().comments.push(c.to_owned())
                    }
                    None => return Err(ReaderError::MissingTransaction(self.line_number)),
                },
                _ => {
                    return Err(ReaderError::UnexpectedItem(
                        "comment".to_owned(),
                        self.line_number,
                    ))
                }
            }

            return Ok(completed_transaction);
        }

        // Check for postings
        if let Ok((_, posting)) = posting(&line) {
            if self.state == ReaderState::None {
                return Err(ReaderError::UnexpectedItem(
                    "posting".to_owned(),
                    self.line_number,
                ));
            }

            // If we're already in a posting, we need to add it to the current transaction
            self.add_posting()?;

            self.state = ReaderState::InPosting;
            self.current_posting = Some(posting);
        }

        Ok(completed_transaction)
    }

    fn add_posting(&mut self) -> Result<(), ReaderError> {
        match self.current_posting.take() {
            None => return Ok(()),
            Some(mut posting) => match self.current_transaction {
                None => return Err(ReaderError::MissingTransaction(self.line_number)),
                Some(ref transaction) => {
                    if posting.amount.is_none() {
                        if transaction.borrow().elided_amount_posting_index.is_some() {
                            return Err(ReaderError::TwoPostingsWithElidedAmounts(
                                self.line_number,
                            ));
                        }
                        let index = transaction.borrow().postings.len();
                        transaction.borrow_mut().elided_amount_posting_index = Some(index);
                    }

                    posting.transaction = Some(Rc::downgrade(transaction));
                    transaction.borrow_mut().postings.push(Rc::new(posting));

                    return Ok(());
                }
            },
        }
    }
}
