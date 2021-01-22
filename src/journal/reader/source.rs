use super::{
    comment::*, error::LineType, error::ReaderError, include::include,
    periodic_transaction::periodic_transaction_header, posting::posting, reader::ReaderState,
    transaction_header::transaction_header,
};
use crate::journal::{
    amount::Amount, periodic_transaction::PeriodicTransaction, posting::Posting, transaction::Transaction,
};
use std::cell::RefCell;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Lines;
use std::path::PathBuf;
use std::rc::Rc;

pub enum ParseResult {
    SourceComplete,
    Transaction(Rc<RefCell<Transaction>>),
    IncludeDirective(String),
}

pub struct Source {
    pub location: PathBuf,
    lines: Lines<BufReader<File>>,
    line_number: u64,
    state: ReaderState,
    periodic_transaction: Option<PeriodicTransaction>,
    transaction: Option<Rc<RefCell<Transaction>>>,
    posting: Option<Posting>,
}

impl Source {
    pub fn new(path: PathBuf) -> Self {
        let file = std::fs::File::open(path.clone()).expect(&format!("File not found"));

        Self {
            location: path,
            lines: std::io::BufReader::new(file).lines(),
            line_number: 0,
            state: ReaderState::None,
            periodic_transaction: None,
            transaction: None,
            posting: None,
        }
    }

    pub fn parse_line(&mut self) -> Result<ParseResult, ReaderError> {
        match self.lines.next() {
            None => return Ok(ParseResult::SourceComplete),
            Some(line) => match line {
                Err(e) => return Err(ReaderError::IO(e, self.line_number)),
                Ok(line) => {
                    self.line_number += 1;

                    if line.len() == 0 {
                        self.state = ReaderState::None;

                        match self.transaction.take() {
                            None => return self.parse_line(),
                            Some(ref transaction) => {
                                if let Some(posting) = self.posting.take() {
                                    transaction.borrow_mut().add_posting(posting, self.line_number - 1)?;
                                }
                                transaction.borrow_mut().close(self.line_number - 1)?;
                                return Ok(ParseResult::Transaction(Rc::clone(transaction)));
                            }
                        }
                    }

                    if let Ok((_, include)) = include(&line) {
                        if self.state != ReaderState::None {
                            return Err(ReaderError::UnexpectedItem(
                                LineType::IncludeDirective,
                                self.line_number,
                            ));
                        }

                        return Ok(ParseResult::IncludeDirective(include.to_owned()));
                    }

                    // Check for period transaction header
                    if let Ok((_, period)) = periodic_transaction_header(&line) {
                        if self.state != ReaderState::None {
                            return Err(ReaderError::UnexpectedItem(
                                LineType::PeriodidTransactionHeader,
                                self.line_number,
                            ));
                        }
                        self.state = ReaderState::InPeriodicTransaction;

                        println!("{:?}", period);

                        // We might have just read a posting, so add that to the previous transaction
                        // TODO this could be merged with the if let below
                        if let Some((transaction, posting)) = self.transaction.as_ref().zip(self.posting.take()) {
                            transaction.borrow_mut().add_posting(posting, self.line_number - 1)?;
                        }

                        // If we had a previous transaction, we need to close it now we're starting a new one
                        if let Some(transaction) = &self.transaction.take() {
                            transaction.borrow_mut().close(self.line_number - 1)?;
                            let completed_transaction = Rc::clone(transaction);
                            self.transaction = Some(Rc::new(RefCell::new(Transaction::new())));
                            return Ok(ParseResult::Transaction(completed_transaction));
                        } else {
                            self.transaction = Some(Rc::new(RefCell::new(Transaction::new())));
                            return self.parse_line();
                        }
                    }

                    // Check for transaction header
                    if let Ok((_, transaction_header)) = transaction_header(&line) {
                        if self.state != ReaderState::None {
                            return Err(ReaderError::UnexpectedItem(
                                LineType::TransactionHeader,
                                self.line_number,
                            ));
                        }
                        self.state = ReaderState::InTransaction;

                        // We might have just read a posting, so add that to the previous transaction
                        if let Some((transaction, posting)) = self.transaction.as_ref().zip(self.posting.take()) {
                            transaction.borrow_mut().add_posting(posting, self.line_number - 1)?;
                        }

                        // If we had a previous transaction, we need to close it now we're starting a new one
                        if let Some(transaction) = &self.transaction {
                            transaction.borrow_mut().close(self.line_number - 1)?;
                            let completed_transaction = Rc::clone(transaction);

                            self.transaction =
                                Some(Rc::new(RefCell::new(Transaction::from_header(transaction_header))));

                            return Ok(ParseResult::Transaction(completed_transaction));
                        } else {
                            self.transaction =
                                Some(Rc::new(RefCell::new(Transaction::from_header(transaction_header))));

                            return self.parse_line();
                        }
                    }

                    // Check for comments
                    // This must come before postings because that lexer will match comments greedily
                    if let Ok((_, comment)) = comment_min(2, &line) {
                        match self.state {
                            ReaderState::InPosting => match &mut self.posting {
                                None => return Err(ReaderError::MissingPosting(self.line_number)),
                                Some(posting) => posting.add_comment(comment.to_owned()),
                            },
                            ReaderState::InTransaction => match &self.transaction {
                                None => return Err(ReaderError::MissingTransaction(self.line_number)),
                                Some(transaction) => transaction.borrow_mut().comments.push(comment.to_owned()),
                            },
                            _ => return Err(ReaderError::UnexpectedItem(LineType::Comment, self.line_number)),
                        }

                        return self.parse_line();
                    }

                    // Check for postings
                    if let Ok((_, mut posting)) = posting(&line) {
                        if self.state == ReaderState::None {
                            return Err(ReaderError::UnexpectedItem(LineType::Posting, self.line_number));
                        }
                        self.state = ReaderState::InPosting;

                        // If we're already in a posting, we need to add it to the current transaction
                        // We haven't done this already because we might need to add following comments first
                        if let Some((transaction, posting)) = self.transaction.as_ref().zip(self.posting.take()) {
                            transaction.borrow_mut().add_posting(posting, self.line_number - 1)?;
                        }

                        posting.transaction = Some(Rc::downgrade(self.transaction.as_ref().unwrap()));
                        self.posting = Some(posting);

                        return self.parse_line();
                    }

                    return self.parse_line();
                }
            },
        }
    }
}

impl Transaction {
    fn add_posting(&mut self, posting: Posting, line: u64) -> Result<(), ReaderError> {
        if posting.amount.is_none() {
            if self.elided_amount_posting_index.is_some() {
                return Err(ReaderError::TwoPostingsWithElidedAmounts(line));
            }
            let index = self.postings.len();
            self.elided_amount_posting_index = Some(index);
        }

        self.postings.push(Rc::new(posting));

        return Ok(());
    }

    /// Returns true if the transaction was closed
    pub fn close(&mut self, line: u64) -> Result<(), ReaderError> {
        let mut sum = 0_i64;
        for p in self.postings.iter_mut() {
            if let Some(a) = &p.amount {
                sum += a.quantity;
            }
        }

        if sum == 0 {
            return Ok(());
        }

        // If there is no posting with an elided amount, we can't balance the transaction
        if self.elided_amount_posting_index.is_none() {
            // we step up a line here because by this point we've moved past the transaction in question
            return Err(ReaderError::TransactionDoesNotBalance(line));
        }

        let index = self.elided_amount_posting_index.unwrap();

        match Rc::get_mut(&mut self.postings[index]) {
            None => return Err(ReaderError::TransactionDoesNotBalance(line)),
            Some(posting) => posting.amount = Some(Amount::new(-sum, "")),
        }

        Ok(())
    }
}
