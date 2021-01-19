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
    // The transaction needs to be RcMut so that multiple postings can hold onto it
    transaction: Rc<RefCell<Transaction>>,
    //The posting needs to be optional because in between transactions, we won't have one
    /// We keep track of the current posting so we can add commennts to it
    posting: Option<Posting>,
}

impl Source {
    pub fn new(path: &str) -> Self {
        let file = std::fs::File::open(path).expect(&format!("File not found: {}", path));

        Self {
            location: PathBuf::from(path),
            lines: std::io::BufReader::new(file).lines(),
            line_number: 0,
            state: ReaderState::None,
            transaction: Rc::new(RefCell::new(Transaction::new())),
            // posting: Rc::new(RefCell::new(Posting::new())),
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
                        self.add_posting()?;
                        self.close_current_transaction()?;
                        self.state = ReaderState::None;
                        return Ok(ParseResult::Transaction(Rc::clone(&self.transaction)));
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
                        // We might have just read a posting, so add that to the previous transaction
                        self.add_posting()?;

                        // If had a previous transaction, we need to close it now we're starting a new one
                        self.close_current_transaction()?;
                        let completed_transaction = self.transaction.clone();

                        self.state = ReaderState::InTransaction;
                        self.transaction =
                            Rc::new(RefCell::new(Transaction::from_header(transaction_header)));

                        return Ok(ParseResult::Transaction(completed_transaction));
                    }

                    // Check for comments
                    // This must come before postings because that lexer will match comments greedily
                    if let Ok((_, comment)) = comment_min(2, &line) {
                        match self.state {
                            ReaderState::InPosting => {
                                println!("");
                                // self.posting.add_comment(comment.to_owned())
                            }
                            ReaderState::InTransaction => {
                                self.transaction
                                    .borrow_mut()
                                    .comments
                                    .push(comment.to_owned());
                            }
                            _ => {
                                return Err(ReaderError::UnexpectedItem(
                                    "comment".to_owned(),
                                    self.line_number,
                                ))
                            }
                        }

                        return Ok(ParseResult::SourceIncomplete);
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
                        // We haven't done this already because we might need to add following comments first
                        self.add_posting()?;

                        self.state = ReaderState::InPosting;
                        self.posting = Some(posting);

                        return Ok(ParseResult::SourceIncomplete);
                    }

                    Ok(ParseResult::SourceIncomplete)
                }
            },
        }
    }

    fn add_posting(&mut self) -> Result<(), ReaderError> {
        match self.posting.take() {
            None => return Ok(()),
            Some(mut posting) => {
                if posting.amount.is_none() {
                    if self
                        .transaction
                        .borrow()
                        .elided_amount_posting_index
                        .is_some()
                    {
                        return Err(ReaderError::TwoPostingsWithElidedAmounts(self.line_number));
                    }
                    let index = self.transaction.borrow().postings.len();
                    self.transaction.borrow_mut().elided_amount_posting_index = Some(index);
                }

                posting.transaction = Some(Rc::downgrade(&self.transaction));
                self.transaction
                    .borrow_mut()
                    .postings
                    .push(Rc::new(posting));

                return Ok(());
            }
        }
    }

    pub fn close_current_transaction(&self) -> Result<(), ReaderError> {
        let mut sum = 0_i64;
        for p in self.transaction.borrow_mut().postings.iter_mut() {
            match &p.amount {
                Some(a) => sum += a.quantity,
                None => (),
            }
        }

        if sum == 0 {
            return Ok(());
        }

        // If there is no posting with an elided amount, we can't balance the transaction
        if self
            .transaction
            .borrow()
            .elided_amount_posting_index
            .is_none()
        {
            // we step up a line here because by this point we've moved past the transaction in
            // question
            return Err(ReaderError::TransactionDoesNotBalance(self.line_number - 1));
            // TODO this is the wrong line number
        }

        let index = self
            .transaction
            .borrow()
            .elided_amount_posting_index
            .unwrap();

        match Rc::get_mut(&mut self.transaction.borrow_mut().postings[index]) {
            None => return Ok(()), // TODO we should probably handle this case
            Some(posting) => {
                posting.amount = Some(Amount::new(-sum, ""));
                Ok(())
            }
        }
    }
}
