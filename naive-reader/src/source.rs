use std::iter::Peekable;
use std::path::Path;
use std::path::PathBuf;
use std::str::Chars;
use std::sync::mpsc::Sender;
use std::sync::Arc;
use std::thread;

use journal::transaction::Status;
use journal::Amount;
use journal::Posting;
use journal::Transaction;

use super::bufreader::BufReader;
use super::error::Error;
use super::error::LineType;
use super::transaction_header::TransactionHeader;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum State {
    None,
    InTransaction,
    InPeriodicTransaction,
    InPosting,
}

pub enum ParseResult {
    SourceComplete,
    Transaction(Arc<Transaction>),
    IncludeDirective(Arc<PathBuf>),
}

pub struct Source {
    location: Arc<PathBuf>,
    contents: BufReader,
    state: State,
    transaction: Option<Arc<Transaction>>,
    posting: Option<Posting>,
    line_number: u64,
}

impl Source {
    pub fn new<P: Into<PathBuf> + AsRef<Path> + Copy>(path: P) -> Self {
        Self {
            location: Arc::new(path.into()),
            contents: BufReader::open(path).unwrap(),
            state: State::None,
            transaction: None,
            posting: None,
            line_number: 0,
        }
    }

    pub fn parse(&mut self, sender: Sender<Result<ParseResult, Error>>) {
        let result = self.parse_line();
        let mut should_continue = true;

        match &result {
            Err(e) => {
                println!("parser error: {}", e);
                return;
            }
            Ok(result) => match result {
                ParseResult::SourceComplete => {
                    should_continue = false;
                }
                ParseResult::IncludeDirective(include) => {
                    let send = sender.clone();
                    let include = include.clone();
                    thread::spawn(move || {
                        let mut source = Source::new(include.as_path());
                        source.parse(send);
                    });
                }
                ParseResult::Transaction(_) => {}
            },
        }

        match sender.send(result) {
            Ok(_) => {
                if should_continue {
                    self.parse(sender);
                }
            }
            Err(_) => {
                println!("sender: breaking");
            }
        }
    }

    fn parse_line(&mut self) -> Result<ParseResult, Error> {
        match self.contents.next() {
            None => {
                // If the source is complete, we need to finish the last transactions
                if let Some(transaction) = &mut self.transaction.take() {
                    // We might have just read a posting, so add that to the previous transaction
                    if let Some(posting) = self.posting.take() {
                        add_posting_to_transaction(transaction, posting, self.line_number - 1)?;
                    }

                    close_transaction(transaction, self.line_number - 1)?;

                    return Ok(ParseResult::Transaction(Arc::clone(&transaction)));
                }

                return Ok(ParseResult::SourceComplete);
            }
            Some(line) => match line {
                // TODO: handle this error
                Err(e) => panic!("{}", e),
                Ok(line) => {
                    self.line_number += 1;

                    let mut iter = line.chars().peekable();

                    match iter.peek() {
                        // Empty line
                        None => {
                            // If the line is empty, we need to finish the previous transaction
                            if let Some(transaction) = &mut self.transaction.take() {
                                // We might have just read a posting, so add that to the previous transaction
                                if let Some(posting) = self.posting.take() {
                                    add_posting_to_transaction(transaction, posting, self.line_number - 1)?;
                                }

                                close_transaction(transaction, self.line_number - 1)?;

                                return Ok(ParseResult::Transaction(Arc::clone(&transaction)));
                            }

                            self.state = State::None;
                            self.parse_line()
                        }

                        // Transaction header
                        Some(c) if c.is_numeric() => {
                            if self.state != State::None && self.state != State::InPosting {
                                return Err(Error::UnexpectedItem(LineType::TransactionHeader, self.line_number));
                            }
                            self.state = State::InTransaction;

                            let transaction = self.lex_transaction_header(&mut iter)?;
                            self.finish_transaction(Some(transaction))
                        }

                        // Posting or comment
                        Some(c) if c.is_whitespace() => {
                            if consume_space(&mut iter) < 2 {
                                panic!("not enough spaces beginning line {}", self.line_number)
                            }

                            // TODO: add new error type for this
                            let next = iter
                                .peek()
                                .ok_or(Error::UnexpectedItem(LineType::Comment, self.line_number))?;

                            // Handle comment
                            if is_comment_indicator(next) {
                                // Advance past the comment indicator
                                iter.next();

                                match self.lex_comment(&mut iter) {
                                    None => return self.parse_line(),
                                    Some(comment) => match self.state {
                                        State::InPosting => match &mut self.posting {
                                            None => return Err(Error::MissingPosting(self.line_number)),
                                            Some(posting) => posting.add_comment(comment),
                                        },

                                        State::InTransaction => match &mut self.transaction {
                                            None => return Err(Error::MissingTransaction(self.line_number)),
                                            Some(transaction) => {
                                                if let Some(transaction) = Arc::get_mut(transaction) {
                                                    transaction.comments.push(comment)
                                                }
                                            }
                                        },

                                        // TODO handle comments in periodic transactions
                                        _ => return Err(Error::UnexpectedItem(LineType::Comment, self.line_number)),
                                    },
                                }

                                return self.parse_line();
                            }

                            // Handle posting
                            if self.state == State::None {
                                return Err(Error::UnexpectedItem(LineType::Posting, self.line_number));
                            }
                            self.state = State::InPosting;

                            let posting = self.lex_posting(&mut iter)?;

                            // If we're already in a posting, we need to add it to the current transaction
                            // We haven't done this already because we might need to add following comments first
                            if let Some((transaction, posting)) = self.transaction.as_mut().zip(self.posting.take()) {
                                add_posting_to_transaction(transaction, posting, self.line_number - 1)?;
                            }

                            // posting.transaction = Some(Arc::downgrade(self.transaction));
                            self.posting = Some(posting);

                            self.parse_line()
                        }

                        // Include directive
                        Some(c) if c == &'i' => {
                            let include = self.lex_include_directive(&mut iter)?;
                            match self.location.clone().parent() {
                                None => panic!("no parent"),
                                Some(parent) => {
                                    return Ok(ParseResult::IncludeDirective(Arc::new(parent.join(include))));
                                }
                            }
                        }

                        // File comment
                        Some(c) if c == &';' => self.parse_line(),

                        // Periodic transaction
                        Some(c) if c == &'~' => {
                            unimplemented!();
                        }

                        // Unmatched line type
                        _ => self.parse_line(),
                    }
                }
            },
        }
    }

    /// Handles closing the previous transaction (if there was one) and starting a new one
    fn finish_transaction(&mut self, new_transaction: Option<Transaction>) -> Result<ParseResult, Error> {
        // If we had a previous transaction, we need to close it now we're starting a new one
        if let Some(transaction) = &mut self.transaction {
            // We might have just read a posting, so add that to the previous transaction
            if let Some(posting) = self.posting.take() {
                add_posting_to_transaction(transaction, posting, self.line_number - 1)?;
            }

            close_transaction(transaction, self.line_number - 1)?;

            let completed_transaction = Arc::clone(transaction);

            match new_transaction {
                Some(new_transaction) => self.transaction = Some(Arc::new(new_transaction)),
                None => self.transaction = Some(Arc::new(Transaction::new())),
            }

            return Ok(ParseResult::Transaction(completed_transaction));
        } else {
            match new_transaction {
                Some(new_transaction) => self.transaction = Some(Arc::new(new_transaction)),
                None => self.transaction = Some(Arc::new(Transaction::new())),
            }

            return self.parse_line();
        }
    }

    fn lex_include_directive(&mut self, iter: &mut Peekable<Chars>) -> Result<String, Error> {
        let include = take_to_space(iter);
        if include != "include" {
            return Err(Error::Parse(LineType::IncludeDirective, self.line_number));
        }
        Ok(take_to_end(iter))
    }

    fn lex_transaction_header(&mut self, iter: &mut Peekable<Chars>) -> Result<Transaction, Error> {
        let date = self.parse_date(take_to_space(iter))?;

        let mut transaction = Transaction::new();
        transaction.date = date;

        consume_space(iter);

        match iter.peek() {
            Some(c) if is_status(c) => {
                transaction.status = match c {
                    &'*' => Status::Cleared,
                    &'!' => Status::Uncleared,
                    _ => Status::NoStatus,
                };
                iter.next();
                consume_space(iter);
            }
            _ => transaction.status = Status::NoStatus,
        }

        transaction.payee = take_to_comment_or_end(iter);
        transaction.header_comment = self.lex_comment(iter);

        return Ok(transaction);
    }

    fn parse_date(&self, s: String) -> Result<time::Date, Error> {
        let mut year: Option<i32> = None;
        let mut month: Option<u8> = None;
        let mut day: Option<u8> = None;

        for sep in &['.', '-', '/'] {
            if s.contains(*sep) {
                for (i, component) in s.split(*sep).enumerate() {
                    match i {
                        0 => year = component.parse().ok(),
                        1 => month = component.parse().ok(),
                        2 => day = component.parse().ok(),
                        _ => panic!("too many components"),
                    }
                }
                break;
            }
        }

        return match (year, month, day) {
            (Some(y), Some(m), Some(d)) => time::Date::try_from_ymd(y, m, d),
            (Some(y), Some(m), None) => time::Date::try_from_ymd(y, m, 1),
            (Some(y), None, None) => time::Date::try_from_ymd(y, 1, 1),
            _ => unimplemented!(),
        }
        .map_err(|_| Error::Parse(LineType::TransactionHeader, self.line_number));
    }

    fn lex_posting(&mut self, iter: &mut Peekable<Chars>) -> Result<Posting, Error> {
        let account = take_to_multispace(iter);

        consume_space(iter);

        let commodity = take_to_number(iter);

        let mut quantity: Option<i64> = None;
        let parsed = take_to_end(iter);
        if parsed.len() > 0 {
            let q: f64 = parsed
                .trim_end()
                .parse()
                .map_err(|_| Error::Parse(LineType::Posting, self.line_number))?;
            quantity = Some((q * 100.) as i64);
        }

        let mut posting = Posting {
            path: account,
            amount: None,
            comments: vec![],
            transaction: None,
        };

        match quantity {
            None => {}
            Some(q) => posting.amount = Some(Amount::new(q, &commodity)),
        }

        return Ok(posting);
    }

    /// If the comment's length is 0, this will return None
    fn lex_comment(&mut self, iter: &mut Peekable<Chars>) -> Option<String> {
        consume_space(iter);
        let comment = take_to_end(iter);
        match comment.len() {
            0 => None,
            _ => Some(comment),
        }
    }
}

fn add_posting_to_transaction(transaction: &mut Arc<Transaction>, posting: Posting, line: u64) -> Result<(), Error> {
    match Arc::get_mut(transaction) {
        None => unimplemented!(),
        Some(transaction) => {
            if posting.amount.is_none() {
                if transaction.elided_amount_posting_index.is_some() {
                    return Err(Error::TwoPostingsWithElidedAmounts(line));
                }
                let index = transaction.postings.len();
                transaction.elided_amount_posting_index = Some(index);
            }

            transaction.postings.push(Arc::new(posting));

            return Ok(());
        }
    }
}

/// Returns true if the transaction was closed
fn close_transaction(transaction: &mut Arc<Transaction>, line: u64) -> Result<(), Error> {
    match Arc::get_mut(transaction) {
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

            match Arc::get_mut(&mut transaction.postings[index]) {
                None => return Err(Error::TransactionDoesNotBalance(line)),
                Some(posting) => posting.amount = Some(Amount::new(-sum, "")),
            }

            Ok(())
        }
    }
}

fn take_to_multispace(iter: &mut Peekable<Chars>) -> String {
    iter.scan("", |state, c| {
        if c == '\t' {
            return None;
        }

        if c == ' ' {
            if state == &" " {
                return None;
            }
            *state = " ";
        }

        Some(c)
    })
    .collect()
}

fn take_to_space(iter: &mut Peekable<Chars>) -> String {
    iter.take_while(|c| whitespace_size(c) == 0).collect()
}

fn take_to_comment_or_end(iter: &mut Peekable<Chars>) -> String {
    iter.take_while(|c| !is_comment_indicator(c)).collect()
}

fn take_to_end(iter: &mut Peekable<Chars>) -> String {
    iter.collect()
}

fn take_to_number(iter: &mut Peekable<Chars>) -> String {
    take_until(iter, |c| c.is_numeric() || c == &'+' || c == &'-', "".to_string())
}

fn take_until(iter: &mut Peekable<Chars>, predicate: fn(&char) -> bool, mut buffer: String) -> String {
    match iter.peek() {
        None => buffer,
        Some(c) => {
            if predicate(c) {
                buffer
            } else {
                buffer.push(*c);
                iter.next();
                take_until(iter, predicate, buffer)
            }
        }
    }
}

/// Consumes spaces in the iterator and returns how many it did
fn consume_space(iter: &mut Peekable<Chars>) -> u8 {
    match iter.peek() {
        None => return 0,
        Some(c) => {
            let space_size = whitespace_size(c);
            if space_size == 0 {
                return 0;
            } else {
                iter.next();
                return space_size + consume_space(iter);
            }
        }
    }
}

fn whitespace_size(c: &char) -> u8 {
    match c {
        &' ' => 1,
        &'\t' => 2,
        _ => 0,
    }
}

fn is_comment_indicator(c: &char) -> bool {
    c == &';' || c == &'#'
}

fn is_status(c: &char) -> bool {
    c == &'!' || c == &'*'
}
