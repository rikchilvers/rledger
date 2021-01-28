use super::{
    bufreader::BufReader,
    comment::parse_comment,
    error::Error,
    error::LineType,
    include::parse_include,
    periodic_transaction::parse_periodic_transaction_header,
    posting::parse_posting,
    transaction_header::{parse_transaction_header, transaction_from_header, TransactionHeader},
};

use journal::{Amount, PeriodicTransaction, Posting, Transaction};

use std::path::PathBuf;
use std::rc::Rc;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ReaderState {
    None, // once we have started parsing, we'll never re-enter this state
    InTransaction,
    InPeriodicTransaction,
    InPosting,
}

pub enum ParseResult {
    SourceComplete,
    Transaction(Rc<Transaction>),
    IncludeDirective(String),
}

pub struct Source {
    pub location: PathBuf,
    contents: BufReader,
    line_number: u64,
    state: ReaderState,
    periodic_transaction: Option<PeriodicTransaction>,
    transaction: Option<Rc<Transaction>>,
    posting: Option<Posting>,
}

impl Source {
    pub fn new(path: PathBuf) -> Self {
        Self {
            location: path.clone(),
            contents: BufReader::open(path).unwrap(),
            line_number: 0,
            state: ReaderState::None,
            periodic_transaction: None,
            transaction: None,
            posting: None,
        }
    }

    pub fn parse_line(&mut self) -> Result<ParseResult, Error> {
        match self.contents.next() {
            None => {
                // If the source is complete, we need to finish the last transactions
                if let Some(transaction) = &mut self.transaction.take() {
                    // We might have just read a posting, so add that to the previous transaction
                    if let Some(posting) = self.posting.take() {
                        add_posting_to_transaction(transaction, posting, self.line_number - 1)?;
                    }

                    close_transaction(transaction, self.line_number - 1)?;

                    return Ok(ParseResult::Transaction(Rc::clone(&transaction)));
                }

                return Ok(ParseResult::SourceComplete);
            }
            Some(line) => match line {
                // TODO: handle this error
                Err(e) => panic!("{}", e),
                Ok(line) => {
                    // TODO: move this to contents
                    self.line_number += 1;

                    // Check for a comment
                    // FIXME: this must come before postings because it is more discerning than the
                    // account parser (which takes everything up to markers)
                    // Ideally, we want the parsers to run most to least prevalent
                    if let Some(comment) = parse_comment(&line, self.line_number)? {
                        match self.state {
                            ReaderState::InPosting => match &mut self.posting {
                                None => return Err(Error::MissingPosting(self.line_number)),
                                Some(posting) => posting.add_comment(comment.to_owned()),
                            },
                            ReaderState::InTransaction => match &mut self.transaction {
                                None => return Err(Error::MissingTransaction(self.line_number)),
                                Some(transaction) => {
                                    if let Some(transaction) = Rc::get_mut(transaction) {
                                        transaction.comments.push(comment.to_owned())
                                    }
                                }
                            },
                            // TODO handle comments in periodic transactions
                            _ => return Err(Error::UnexpectedItem(LineType::Comment, self.line_number)),
                        }

                        return self.parse_line();
                    }

                    // Check for a posting
                    if let Some(posting) = parse_posting(&line, self.line_number)? {
                        if self.state == ReaderState::None {
                            return Err(Error::UnexpectedItem(LineType::Posting, self.line_number));
                        }
                        self.state = ReaderState::InPosting;

                        // If we're already in a posting, we need to add it to the current transaction
                        // We haven't done this already because we might need to add following comments first
                        if let Some((transaction, posting)) = self.transaction.as_mut().zip(self.posting.take()) {
                            add_posting_to_transaction(transaction, posting, self.line_number - 1)?;
                        }

                        // posting.transaction = Some(Rc::downgrade(self.transaction));
                        self.posting = Some(posting);

                        return self.parse_line();
                    }

                    // Check for a transaction header
                    if let Some(transaction_header) = parse_transaction_header(&line, self.line_number)? {
                        if self.state != ReaderState::None && self.state != ReaderState::InPosting {
                            return Err(Error::UnexpectedItem(LineType::TransactionHeader, self.line_number));
                        }
                        self.state = ReaderState::InTransaction;

                        return self.finish_transaction(Some(transaction_header));
                    }

                    // Check for an include directive
                    if let Some(include) = parse_include(&line, self.line_number)? {
                        return Ok(ParseResult::IncludeDirective(include.to_owned()));
                    }

                    // Check for a period transaction header
                    if let Some(period) = parse_periodic_transaction_header(&line, self.line_number)? {
                        if self.state != ReaderState::None {
                            return Err(Error::UnexpectedItem(
                                LineType::PeriodidTransactionHeader,
                                self.line_number,
                            ));
                        }
                        self.state = ReaderState::InPeriodicTransaction;

                        return self.finish_transaction(None);
                    }

                    // We didn't match anything so move on
                    return self.parse_line();
                }
            },
        }
    }

    fn finish_transaction(&mut self, transaction_header: Option<TransactionHeader>) -> Result<ParseResult, Error> {
        // If we had a previous transaction, we need to close it now we're starting a new one
        if let Some(transaction) = &mut self.transaction {
            // We might have just read a posting, so add that to the previous transaction
            if let Some(posting) = self.posting.take() {
                add_posting_to_transaction(transaction, posting, self.line_number - 1)?;
            }

            close_transaction(transaction, self.line_number - 1)?;

            let completed_transaction = Rc::clone(transaction);

            match transaction_header {
                Some(transaction_header) => {
                    self.transaction = Some(Rc::new(transaction_from_header(transaction_header)))
                }
                None => self.transaction = Some(Rc::new(Transaction::new())),
            }

            return Ok(ParseResult::Transaction(completed_transaction));
        } else {
            match transaction_header {
                Some(transaction_header) => {
                    self.transaction = Some(Rc::new(transaction_from_header(transaction_header)))
                }
                None => self.transaction = Some(Rc::new(Transaction::new())),
            }

            return self.parse_line();
        }
    }
}

fn add_posting_to_transaction(transaction: &mut Rc<Transaction>, posting: Posting, line: u64) -> Result<(), Error> {
    match Rc::get_mut(transaction) {
        Some(transaction) => {
            if posting.amount.is_none() {
                if transaction.elided_amount_posting_index.is_some() {
                    return Err(Error::TwoPostingsWithElidedAmounts(line));
                }
                let index = transaction.postings.len();
                transaction.elided_amount_posting_index = Some(index);
            }

            transaction.postings.push(Rc::new(posting));

            return Ok(());
        }
        None => unimplemented!(),
    }
}

/// Returns true if the transaction was closed
fn close_transaction(transaction: &mut Rc<Transaction>, line: u64) -> Result<(), Error> {
    match Rc::get_mut(transaction) {
        None => unimplemented!(),
        Some(transaction) => {
            let mut sum = 0_i64;
            for p in transaction.postings.iter_mut() {
                if let Some(a) = &p.amount {
                    sum += a.quantity;
                }
            }

            if sum == 0 {
                return Ok(());
            }

            // If there is no posting with an elided amount, we can't balance the transaction
            if transaction.elided_amount_posting_index.is_none() {
                // we step up a line here because by this point we've moved past the transaction in question
                return Err(Error::TransactionDoesNotBalance(line));
            }

            let index = transaction.elided_amount_posting_index.unwrap();

            match Rc::get_mut(&mut transaction.postings[index]) {
                None => return Err(Error::TransactionDoesNotBalance(line)),
                Some(posting) => posting.amount = Some(Amount::new(-sum, "")),
            }

            Ok(())
        }
    }
}
