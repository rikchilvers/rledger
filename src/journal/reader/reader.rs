use super::{
    comment::*, error::ReaderError, include::include, posting::posting,
    transaction_header::transaction_header,
};
use crate::journal::{
    amount::Amount, posting::Posting, transaction::Transaction,
    transaction_header::TransactionHeader,
};
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

enum LineType {
    None,
    Empty,
    PostingComment(String),
    TransactionComment(String),
    TransactionHeader(TransactionHeader),
    Posting(Posting),
    IncludeDirective(String),
}

pub struct Reader {
    state: ReaderState,
    line_number: u64,
    lines: Lines<std::io::BufReader<std::fs::File>>,

    current_transaction: Option<Rc<RefCell<Transaction>>>,
    // We keep track of the current posting so we can add commennts to it
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
                    match self.read_to_next(&line) {
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
    pub fn new(location: &str) -> Self {
        let file = std::fs::File::open(location).expect(&format!("file not found"));
        let reader = std::io::BufReader::new(file);

        Self {
            state: ReaderState::None,
            line_number: 0,
            lines: reader.lines(),
            current_transaction: None,
            current_posting: None,
        }
    }

    fn read_to_next(
        &mut self,
        line: &str,
    ) -> Result<Option<Rc<RefCell<Transaction>>>, ReaderError> {
        let mut completed_transaction: Option<Rc<RefCell<Transaction>>> = None;

        match self.parse_line(line) {
            Err(e) => return Err(e),
            Ok(line) => match line {
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
                    self.current_transaction = Some(Rc::new(RefCell::new(
                        Transaction::from_header(transaction_header),
                    )));

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
                    None => return Err(ReaderError::MissingPosting(self.line_number)),
                },
                LineType::TransactionComment(comment) => match self.current_transaction {
                    Some(ref mut transaction) => {
                        transaction.borrow_mut().comments.push(comment);
                        return Ok(completed_transaction);
                    }
                    None => return Err(ReaderError::MissingTransaction(self.line_number)),
                },
                _ => Ok(None),
            },
        }
    }

    fn parse_line(&mut self, line: &str) -> Result<LineType, ReaderError> {
        if line.len() == 0 {
            return Ok(LineType::Empty);
        }

        // Check for include directive
        if let Ok((_, include)) = include(&line) {
            if self.state != ReaderState::None {
                return Err(ReaderError::UnexpectedItem(
                    "include directive".to_owned(),
                    self.line_number,
                ));
            }
            return Ok(LineType::IncludeDirective(include.to_owned()));
        }

        // Check for transaction header
        if let Ok((_, transaction_header)) = transaction_header(&line) {
            if self.state != ReaderState::None {
                return Err(ReaderError::UnexpectedItem(
                    "transaction header".to_owned(),
                    self.line_number,
                ));
            }

            return Ok(LineType::TransactionHeader(transaction_header));
        }

        // Check for comments
        // This must come before postings because that lexer will match comments greedily
        if let Ok((_, comment)) = comment_min(2, &line) {
            match self.state {
                ReaderState::InPosting => return Ok(LineType::PostingComment(comment.to_owned())),
                ReaderState::InTransaction => {
                    return Ok(LineType::TransactionComment(comment.to_owned()))
                }
                _ => {
                    return Err(ReaderError::UnexpectedItem(
                        "comment".to_owned(),
                        self.line_number,
                    ))
                }
            }
        }

        // Check for postings
        if let Ok((_, posting)) = posting(&line) {
            if self.state == ReaderState::None {
                return Err(ReaderError::UnexpectedItem(
                    "posting".to_owned(),
                    self.line_number,
                ));
            }

            return Ok(LineType::Posting(posting));
        }

        Ok(LineType::None)
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
            return Err(ReaderError::TransactionDoesNotBalance(self.line_number - 1));
        }

        let index = transaction.borrow().elided_amount_posting_index.unwrap();

        match Rc::get_mut(&mut transaction.borrow_mut().postings[index]) {
            None => return Ok(()), // TODO we should probably handle this case
            Some(posting) => {
                posting.amount = Some(Amount::new(-sum, ""));
            }
        }

        Ok(())
    }
}
