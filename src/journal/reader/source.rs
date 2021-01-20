use super::{
    comment::*, error::ReaderError, include::include, posting::posting, reader::ReaderState,
    transaction_header::transaction_header,
};
use crate::journal::{amount::Amount, posting::Posting, transaction::Transaction};
use std::cell::RefCell;
use std::io::BufRead;
use std::io::Lines;
use std::path::PathBuf;
use std::rc::Rc;

pub enum ParseResult {
    SourceIncomplete,
    SourceComplete,
    Transaction(Rc<RefCell<Transaction>>),
    IncludeDirective(String),
}

pub struct Source {
    pub location: PathBuf,
    lines: Lines<std::io::BufReader<std::fs::File>>,
    pub line_number: u64,
    state: ReaderState,
    transaction: Option<Rc<RefCell<Transaction>>>,
    /// We keep track of the current posting so we can add commennts to it
    posting: Option<Posting>,
}

impl Source {
    pub fn new(path: PathBuf) -> Self {
        let file = std::fs::File::open(path.clone()).expect(&format!("File not found"));

        Self {
            // location: PathBuf::from(path),
            location: path,
            lines: std::io::BufReader::new(file).lines(),
            line_number: 0,
            state: ReaderState::None,
            transaction: None,
            posting: None,
        }
    }

    // TODO: Make this an iterator?
    pub fn parse_line(&mut self) -> Result<ParseResult, ReaderError> {
        match self.lines.next() {
            None => return Ok(ParseResult::SourceComplete),
            Some(line) => match line {
                Err(e) => return Err(ReaderError::IO(e)),
                Ok(line) => {
                    self.line_number += 1;

                    if line.len() == 0 {
                        self.state = ReaderState::None;

                        match self.transaction.take() {
                            None => return Ok(ParseResult::SourceIncomplete),
                            Some(ref transaction) => {
                                if let Some(posting) = self.posting.take() {
                                    transaction.borrow_mut().add_posting(posting)?;
                                }
                                transaction.borrow_mut().close()?;
                                return Ok(ParseResult::Transaction(Rc::clone(transaction)));
                            }
                        }
                    }

                    // Check for include directive
                    if let Ok((_, include)) = include(&line) {
                        if self.state != ReaderState::None {
                            return Err(ReaderError::UnexpectedItem(
                                "include directive".to_owned(),
                                self.line_number,
                            ));
                        }

                        return Ok(ParseResult::IncludeDirective(include.to_owned()));
                    }

                    // Check for transaction header
                    if let Ok((_, transaction_header)) = transaction_header(&line) {
                        if self.state != ReaderState::None {
                            return Err(ReaderError::UnexpectedItem(
                                "transaction header".to_owned(),
                                self.line_number,
                            ));
                        }
                        self.state = ReaderState::InTransaction;

                        // We might have just read a posting, so add that to the previous transaction
                        if let Some((transaction, posting)) = self.transaction.as_ref().zip(self.posting.take()) {
                            transaction.borrow_mut().add_posting(posting)?;
                        }

                        // If had a previous transaction, we need to close it now we're starting a new one
                        if let Some(transaction) = &self.transaction {
                            transaction
                                .borrow_mut()
                                .close()
                                .map_err(|e| e.change_line_number(self.line_number - 1))?;
                            let completed_transaction = Rc::clone(transaction);

                            self.transaction =
                                Some(Rc::new(RefCell::new(Transaction::from_header(transaction_header))));

                            return Ok(ParseResult::Transaction(completed_transaction));
                        } else {
                            self.transaction =
                                Some(Rc::new(RefCell::new(Transaction::from_header(transaction_header))));

                            return Ok(ParseResult::SourceIncomplete);
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
                            _ => return Err(ReaderError::UnexpectedItem("comment".to_owned(), self.line_number)),
                        }

                        return Ok(ParseResult::SourceIncomplete);
                    }

                    // Check for postings
                    if let Ok((_, mut posting)) = posting(&line) {
                        if self.state == ReaderState::None {
                            return Err(ReaderError::UnexpectedItem("posting".to_owned(), self.line_number));
                        }
                        self.state = ReaderState::InPosting;

                        // If we're already in a posting, we need to add it to the current transaction
                        // We haven't done this already because we might need to add following comments first
                        if let Some((transaction, posting)) = self.transaction.as_ref().zip(self.posting.take()) {
                            transaction.borrow_mut().add_posting(posting)?;
                        }

                        posting.transaction = Some(Rc::downgrade(self.transaction.as_ref().unwrap()));
                        self.posting = Some(posting);

                        return Ok(ParseResult::SourceIncomplete);
                    }

                    Ok(ParseResult::SourceIncomplete)
                }
            },
        }
    }
}

impl Transaction {
    fn add_posting(&mut self, posting: Posting) -> Result<(), ReaderError> {
        if posting.amount.is_none() {
            if self.elided_amount_posting_index.is_some() {
                return Err(ReaderError::TwoPostingsWithElidedAmounts(0));
            }
            let index = self.postings.len();
            self.elided_amount_posting_index = Some(index);
        }

        self.postings.push(Rc::new(posting));

        return Ok(());
    }

    /// Returns true if the transaction was closed
    pub fn close(&mut self) -> Result<bool, ReaderError> {
        let mut sum = 0_i64;
        for p in self.postings.iter_mut() {
            if let Some(a) = &p.amount {
                sum += a.quantity;
            }
        }

        if sum == 0 {
            return Ok(true);
        }

        // If there is no posting with an elided amount, we can't balance the transaction
        if self.elided_amount_posting_index.is_none() {
            // we step up a line here because by this point we've moved past the transaction in question
            return Err(ReaderError::TransactionDoesNotBalance(0));
        }

        let index = self.elided_amount_posting_index.unwrap();

        match Rc::get_mut(&mut self.postings[index]) {
            None => return Ok(false), // TODO we should probably handle this case
            Some(posting) => {
                posting.amount = Some(Amount::new(-sum, ""));
                Ok(true)
            }
        }
    }
}