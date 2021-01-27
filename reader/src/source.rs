use super::{
    bufreader::BufReader, comment::*, error::LineType, error::ReaderError, include::parse_include,
    periodic_transaction::parse_periodic_transaction_header, posting::parse_posting,
    transaction_header::parse_transaction_header, transaction_header::transaction_from_header,
};
use journal::{Amount, PeriodicTransaction, Posting, Transaction};
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ReaderState {
    None, // once we have started parsing, we'll never re-enter this state
    InTransaction,
    InPeriodicTransaction,
    InPosting,
}

impl Default for ReaderState {
    fn default() -> Self {
        Self::None
    }
}

pub enum ParseResult {
    SourceComplete,
    Transaction(Rc<RefCell<Transaction>>),
    IncludeDirective(String),
}

pub struct Source {
    pub location: PathBuf,
    contents: BufReader,
    line_number: u64,
    state: ReaderState,
    periodic_transaction: Option<PeriodicTransaction>,
    transaction: Option<Rc<RefCell<Transaction>>>,
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

    pub fn parse_line(&mut self) -> Result<ParseResult, ReaderError> {
        match self.contents.next() {
            None => return Ok(ParseResult::SourceComplete),
            Some(line) => match line {
                Err(e) => panic!("{}", e),
                Ok(line) => {
                    // TODO: move this to contents
                    self.line_number += 1;

                    // println!("{}", line);

                    /*
                    if line.len() == 0 {
                        println!("empty");
                        self.state = ReaderState::None;

                        match self.transaction.take() {
                            None => return self.parse_line(),
                            Some(ref transaction) => {
                                if let Some(posting) = self.posting.take() {
                                    add_posting_to_transaction(
                                        &mut transaction.borrow_mut(),
                                        posting,
                                        self.line_number - 1,
                                    )?;
                                }
                                close_transaction(&mut transaction.borrow_mut(), self.line_number - 1)?;
                                return Ok(ParseResult::Transaction(Rc::clone(transaction)));
                            }
                        }
                    }
                    */

                    // Check for an include directive
                    if let Some(include) = parse_include(&line, self.line_number)? {
                        println!("!! include: {}", include);
                        // if self.state != ReaderState::None {
                        //     return Err(ReaderError::UnexpectedItem(
                        //         LineType::IncludeDirective,
                        //         self.line_number,
                        //     ));
                        // }

                        // return Ok(ParseResult::IncludeDirective(include.to_owned()));
                    }

                    // Check for a period transaction header
                    if let Some(period) = parse_periodic_transaction_header(&line, self.line_number)? {
                        println!("!! pth");
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
                            add_posting_to_transaction(&mut transaction.borrow_mut(), posting, self.line_number - 1)?;
                        }

                        // If we had a previous transaction, we need to close it now we're starting a new one
                        if let Some(transaction) = &self.transaction.take() {
                            close_transaction(&mut transaction.borrow_mut(), self.line_number - 1)?;
                            let completed_transaction = Rc::clone(transaction);
                            self.transaction = Some(Rc::new(RefCell::new(Transaction::new())));
                            return Ok(ParseResult::Transaction(completed_transaction));
                        } else {
                            self.transaction = Some(Rc::new(RefCell::new(Transaction::new())));
                            return self.parse_line();
                        }
                    }

                    // Check for a transaction header
                    if let Some(transaction_header) = parse_transaction_header(&line, self.line_number)? {
                        println!("!! th");
                        if self.state != ReaderState::None && self.state != ReaderState::InPosting {
                            return Err(ReaderError::UnexpectedItem(
                                LineType::TransactionHeader,
                                self.line_number,
                            ));
                        }
                        self.state = ReaderState::InTransaction;

                        // We might have just read a posting, so add that to the previous transaction
                        if let Some((transaction, posting)) = self.transaction.as_ref().zip(self.posting.take()) {
                            add_posting_to_transaction(&mut transaction.borrow_mut(), posting, self.line_number - 1)?;
                        }

                        // If we had a previous transaction, we need to close it now we're starting a new one
                        if let Some(transaction) = &self.transaction {
                            close_transaction(&mut transaction.borrow_mut(), self.line_number - 1)?;
                            let completed_transaction = Rc::clone(transaction);

                            self.transaction = Some(Rc::new(RefCell::new(transaction_from_header(transaction_header))));

                            return Ok(ParseResult::Transaction(completed_transaction));
                        } else {
                            self.transaction = Some(Rc::new(RefCell::new(transaction_from_header(transaction_header))));

                            return self.parse_line();
                        }
                    }

                    // Check for a comment
                    // This must come before postings because that lexer will match comments greedily
                    // TODO check the above is still true
                    if let Ok((_, comment)) = comment_min(2, &line) {
                        println!("!! comment");
                        match self.state {
                            ReaderState::InPosting => match &mut self.posting {
                                None => return Err(ReaderError::MissingPosting(self.line_number)),
                                Some(posting) => posting.add_comment(comment.to_owned()),
                            },
                            ReaderState::InTransaction => match &self.transaction {
                                None => return Err(ReaderError::MissingTransaction(self.line_number)),
                                Some(transaction) => transaction.borrow_mut().comments.push(comment.to_owned()),
                            },
                            // TODO handle comments in periodic transactions
                            _ => return Err(ReaderError::UnexpectedItem(LineType::Comment, self.line_number)),
                        }

                        return self.parse_line();
                    }

                    // Check for a posting
                    if let Some(mut posting) = parse_posting(&line, self.line_number)? {
                        println!("!! posting");
                        if self.state == ReaderState::None {
                            return Err(ReaderError::UnexpectedItem(LineType::Posting, self.line_number));
                        }
                        self.state = ReaderState::InPosting;

                        // If we're already in a posting, we need to add it to the current transaction
                        // We haven't done this already because we might need to add following comments first
                        if let Some((transaction, posting)) = self.transaction.as_ref().zip(self.posting.take()) {
                            add_posting_to_transaction(&mut transaction.borrow_mut(), posting, self.line_number - 1)?;
                        }

                        posting.transaction = Some(Rc::downgrade(self.transaction.as_ref().unwrap()));
                        self.posting = Some(posting);

                        return self.parse_line();
                    }

                    // We didn't match anything so move on
                    return self.parse_line();
                }
            },
        }
    }
}

fn add_posting_to_transaction(transaction: &mut Transaction, posting: Posting, line: u64) -> Result<(), ReaderError> {
    if posting.amount.is_none() {
        if transaction.elided_amount_posting_index.is_some() {
            return Err(ReaderError::TwoPostingsWithElidedAmounts(line));
        }
        let index = transaction.postings.len();
        transaction.elided_amount_posting_index = Some(index);
    }

    transaction.postings.push(Rc::new(posting));

    return Ok(());
}

/// Returns true if the transaction was closed
fn close_transaction(transaction: &mut Transaction, line: u64) -> Result<(), ReaderError> {
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
        return Err(ReaderError::TransactionDoesNotBalance(line));
    }

    let index = transaction.elided_amount_posting_index.unwrap();

    match Rc::get_mut(&mut transaction.postings[index]) {
        None => return Err(ReaderError::TransactionDoesNotBalance(line)),
        Some(posting) => posting.amount = Some(Amount::new(-sum, "")),
    }

    Ok(())
}
