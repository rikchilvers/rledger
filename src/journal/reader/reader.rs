use super::{error::ReaderError, source::LineType, source::Source};
use crate::journal::{amount::Amount, posting::Posting, transaction::Transaction};
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug, PartialEq, Eq)]
pub enum ReaderState {
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
    sources: Vec<Source>,
    current_source: Option<Source>,

    current_transaction: Option<Rc<RefCell<Transaction>>>,
    // We keep track of the current posting so we can add commennts to it
    current_posting: Option<Posting>,
}

impl Iterator for Reader {
    type Item = Result<Rc<RefCell<Transaction>>, ReaderError>;

    fn next(&mut self) -> Option<Self::Item> {
        // Ensure we have a source to work on
        if self.current_source.is_none() {
            if self.sources.last().is_none() {
                return None;
            }
            self.current_source = self.sources.pop();
            return self.next();
        }

        // If we get to this point, we know we have a source, so we can unwrap
        let source = self.current_source.as_mut().unwrap();
        let parsed_line = source.parse_line(&self.state);

        // If the source had no more lines, drop it and start again
        if parsed_line.is_none() {
            self.current_source = None;
            return self.next();
        }

        // If we get to this point, we know we have a parsed_line, so we can unwrap
        match parsed_line.unwrap() {
            Err(e) => return Some(Err(e)),
            Ok(line) => match self.process_line(line) {
                Err(e) => return Some(Err(e)),
                Ok(transaction) => match transaction {
                    None => return self.next(),
                    Some(transaction) => return Some(Ok(transaction)),
                },
            },
        }
    }
}

impl Reader {
    pub fn new(location: &str) -> Self {
        Self {
            sources: vec![],
            current_source: Some(Source::new(location)),
            state: ReaderState::None,
            current_transaction: None,
            current_posting: None,
        }
    }

    fn process_line(
        &mut self,
        line: LineType,
    ) -> Result<Option<Rc<RefCell<Transaction>>>, ReaderError> {
        let mut completed_transaction: Option<Rc<RefCell<Transaction>>> = None;

        match line {
            LineType::Empty => {
                self.add_posting()?;
                // We can take here as we want the transaction to be empty afterwards
                if let Some(t) = &mut self.current_transaction.take() {
                    self.close_transaction(t)?;
                    completed_transaction = Some(t.clone());
                }
                self.state = ReaderState::None;
                return Ok(completed_transaction);
            }
            LineType::IncludeDirective(include) => {
                println!(">> include: {}", include);
                // let path = self .location .parent() .unwrap() .join(include) .to_str() .unwrap();
                // self.included_file = Some(Box::new(Reader::new(path)));
                return Ok(completed_transaction);
            }
            LineType::TransactionHeader(transaction_header) => {
                // We might have just read a transaction, so add that to the previous transaction
                self.add_posting()?;

                // If had a previous transaction, we need to close it now we're starting a new one
                if let Some(t) = &self.current_transaction {
                    self.close_transaction(t)?;
                    completed_transaction = Some(t.clone());
                }

                self.state = ReaderState::InTransaction;
                self.current_transaction = Some(Rc::new(RefCell::new(Transaction::from_header(
                    transaction_header,
                ))));

                return Ok(completed_transaction);
            }
            LineType::Posting(posting) => {
                // If we're already in a posting, we need to add it to the current transaction
                // We haven't done this already because we might need to add following comments first
                self.add_posting()?;

                self.state = ReaderState::InPosting;
                self.current_posting = Some(posting);

                return Ok(completed_transaction);
            }
            LineType::PostingComment(comment) => match &mut self.current_posting {
                Some(p) => {
                    p.add_comment(comment);
                    return Ok(completed_transaction);
                }
                None => {
                    return Err(ReaderError::MissingPosting(
                        self.sources.last().unwrap().line_number,
                    ))
                }
            },
            LineType::TransactionComment(comment) => match self.current_transaction {
                Some(ref mut transaction) => {
                    transaction.borrow_mut().comments.push(comment);
                    return Ok(completed_transaction);
                }
                None => {
                    return Err(ReaderError::MissingTransaction(
                        self.sources.last().unwrap().line_number,
                    ))
                }
            },
            _ => Ok(None),
        }
    }

    fn add_posting(&mut self) -> Result<(), ReaderError> {
        match self.current_posting.take() {
            None => return Ok(()),
            Some(mut posting) => match self.current_transaction {
                None => {
                    return Err(ReaderError::MissingTransaction(
                        self.sources.last().unwrap().line_number,
                    ))
                }
                Some(ref transaction) => {
                    if posting.amount.is_none() {
                        if transaction.borrow().elided_amount_posting_index.is_some() {
                            return Err(ReaderError::TwoPostingsWithElidedAmounts(
                                self.sources.last().unwrap().line_number,
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

    pub fn close_transaction(
        &self,
        transaction: &Rc<RefCell<Transaction>>,
    ) -> Result<(), ReaderError> {
        let mut sum = 0_i64;
        for p in transaction.borrow_mut().postings.iter_mut() {
            match &p.amount {
                Some(a) => sum += a.quantity,
                None => (),
            }
        }

        if sum == 0 {
            return Ok(());
        }

        // If there is no posting with an elided amount, we can't balance the transaction
        if transaction.borrow().elided_amount_posting_index.is_none() {
            // we step up a line here because by this point we've moved past the transaction in
            // question
            return Err(ReaderError::TransactionDoesNotBalance(
                self.sources.last().unwrap().line_number - 1,
            ));
        }

        let index = transaction.borrow().elided_amount_posting_index.unwrap();

        match Rc::get_mut(&mut transaction.borrow_mut().postings[index]) {
            None => return Ok(()), // TODO we should probably handle this case
            Some(posting) => {
                posting.amount = Some(Amount::new(-sum, ""));
                Ok(())
            }
        }
    }
}
