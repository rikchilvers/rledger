use std::{iter::Peekable, path::PathBuf, str::Chars, sync::mpsc::Sender, thread};

use journal::{transaction::Status, Amount, Posting, Transaction};

use super::{
    bufreader::BufReader,
    error::LineType,
    error::{Error, ErrorKind},
};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum State {
    None,
    InTransaction,
    InPeriodicTransaction,
    InPosting,
}

pub struct ParsedItem {
    pub location: PathBuf,
    pub kind: ItemKind,
}

pub enum ItemKind {
    SourceComplete,
    Transaction(Transaction, Vec<Posting>),
    IncludeDirective(PathBuf),
}

pub struct Source {
    // TODO see if we can remove this Arc until it's being passed between threads
    location: PathBuf,
    contents: BufReader,
    state: State,
    line: u64,
    transaction: Option<Transaction>,
    postings: Vec<Posting>,
}

impl Source {
    pub fn new<P: Into<PathBuf>>(path: P) -> Self {
        let path = path.into();

        return Self {
            location: path.clone(),
            contents: BufReader::open(&path).unwrap(),
            state: State::None,
            line: 0,
            transaction: None,
            postings: Vec::with_capacity(10),
        };
    }

    /// Wraps a ParsedItem with the location of this source
    fn new_item(&self, kind: ItemKind) -> ParsedItem {
        ParsedItem {
            location: self.location.clone(),
            kind,
        }
    }

    /// Wraps an ErrorKind with the location of this source
    fn new_error(&self, kind: ErrorKind) -> Error {
        Error {
            location: self.location.clone(),
            line: self.line,
            kind,
        }
    }

    pub fn parse(&mut self, sender: Sender<Result<ParsedItem, Error>>) {
        let result = self.parse_line();
        let mut should_continue = true;

        match &result {
            Err(_) => should_continue = false,
            Ok(result) => match &result.kind {
                ItemKind::Transaction(_, _) => {}
                ItemKind::SourceComplete => should_continue = false,
                ItemKind::IncludeDirective(include) => {
                    let send = sender.clone();
                    let include = include.clone();
                    thread::spawn(move || {
                        let mut source = Source::new(include.as_path());
                        source.parse(send);
                    });
                }
            },
        }

        match sender.send(result) {
            Ok(_) => {
                if should_continue {
                    self.parse(sender);
                }
            }
            Err(_) => {
                // println!("Error while bloop parsing: {}", e);
            }
        }
    }

    /// Checks that the transaction balances (and handles an elided amount)
    fn close_transaction(&mut self) -> Option<Result<ParsedItem, ErrorKind>> {
        match self.transaction.take() {
            None => return None,
            Some(transaction) => {
                let mut elided_index = None;
                let mut sum = 0_i64;
                let mut postings = Vec::with_capacity(self.postings.len());

                for (i, posting) in self.postings.drain(0..).enumerate() {
                    match &posting.amount {
                        Some(amount) => sum += amount.quantity,
                        None => {
                            if elided_index.is_some() {
                                return Some(Err(ErrorKind::TwoPostingsWithElidedAmounts));
                            }
                            elided_index = Some(i);
                        }
                    }
                    postings.push(posting);
                }

                // The transaction is balanced so return it
                if sum == 0 {
                    return Some(Ok(self.new_item(ItemKind::Transaction(transaction, postings))));
                }

                match elided_index {
                    // If there is no posting with an elided amount, we can't balance the transaction
                    None => return Some(Err(ErrorKind::TransactionDoesNotBalance)),
                    // TODO assign a commodity to this amount
                    Some(i) => postings[i].amount = Some(Amount::new(-sum, "")),
                }

                return Some(Ok(self.new_item(ItemKind::Transaction(transaction, postings))));
            }
        }
    }

    fn parse_line(&mut self) -> Result<ParsedItem, Error> {
        match self.contents.next() {
            None => {
                // If the source is complete, we need to finish the last transactions
                if let Some(result) = self.close_transaction() {
                    return result.map_err(|kind| self.new_error(kind));
                }

                return Ok(self.new_item(ItemKind::SourceComplete));
            }
            Some(line) => match line {
                Err(e) => return Err(self.new_error(ErrorKind::IO(e))),
                Ok(line) => {
                    self.line += 1;

                    let mut iter = line.chars().peekable();

                    match iter.peek() {
                        // Empty line
                        None => {
                            // If the line is empty, we need to finish the previous transaction
                            if let Some(result) = self.close_transaction() {
                                return result.map_err(|kind| self.new_error(kind));
                            }

                            self.state = State::None;
                            self.parse_line()
                        }

                        // Transaction header
                        Some(c) if c.is_numeric() => {
                            if self.state != State::None && self.state != State::InPosting {
                                return Err(self.new_error(ErrorKind::UnexpectedItem(LineType::TransactionHeader)));
                            }
                            self.state = State::InTransaction;

                            let new_transaction = self.parse_transaction_header(&mut iter)?;

                            if let Some(result) = self.close_transaction() {
                                self.transaction = Some(new_transaction);
                                return result.map_err(|kind| self.new_error(kind));
                            } else {
                                self.transaction = Some(new_transaction);
                                self.parse_line()
                            }
                        }

                        // Posting or comment
                        Some(c) if c.is_whitespace() => {
                            if consume_space(&mut iter) < 2 {
                                return Err(self.new_error(ErrorKind::IncorrectFormatting(
                                    "not enough spaces beginning line".to_string(),
                                )));
                            }

                            let next = iter.peek().ok_or(self.new_error(ErrorKind::IncorrectFormatting(
                                "not enough remaining characters".to_string(),
                            )))?;

                            // Handle comment
                            if is_comment_indicator(next) {
                                // Advance past the comment indicator
                                iter.next();

                                match self.parse_comment(&mut iter) {
                                    None => return self.parse_line(),
                                    Some(comment) => match self.state {
                                        State::InPosting => match &mut self.postings.last_mut() {
                                            None => return Err(self.new_error(ErrorKind::MissingPosting)),
                                            Some(posting) => posting.add_comment(comment),
                                        },

                                        State::InTransaction => match &mut self.transaction {
                                            None => return Err(self.new_error(ErrorKind::MissingTransaction)),
                                            Some(transaction) => transaction.comments.push(comment),
                                        },

                                        // TODO handle comments in periodic transactions
                                        _ => return Err(self.new_error(ErrorKind::UnexpectedItem(LineType::Comment))),
                                    },
                                }

                                return self.parse_line();
                            }

                            // Handle posting
                            if self.state == State::None {
                                return Err(self.new_error(ErrorKind::UnexpectedItem(LineType::Posting)));
                            }
                            self.state = State::InPosting;

                            let posting = self.parse_posting(&mut iter)?;
                            self.postings.push(posting);

                            self.parse_line()
                        }

                        // Include directive
                        Some(c) if c == &'i' => {
                            let include = self.parse_include_directive(&mut iter)?;
                            match self.location.clone().parent() {
                                None => panic!("no parent"),
                                Some(parent) => {
                                    return Ok(self.new_item(ItemKind::IncludeDirective(parent.join(include))));
                                }
                            }
                        }

                        // File comment
                        Some(c) if c == &';' => self.parse_line(),

                        // Periodic transaction
                        // TODO handle periodic transactions
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

    fn parse_include_directive(&mut self, iter: &mut Peekable<Chars>) -> Result<String, Error> {
        let include = take_to_space(iter);
        if include != "include" {
            return Err(self.new_error(ErrorKind::Parse(LineType::IncludeDirective)));
        }
        Ok(take_to_end(iter))
    }

    fn parse_transaction_header(&mut self, iter: &mut Peekable<Chars>) -> Result<Transaction, Error> {
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

        transaction.payee = take_to_comment_or_end(iter).trim_end().to_owned();
        transaction.header_comment = self.parse_comment(iter);

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
        .map_err(|_| self.new_error(ErrorKind::Parse(LineType::TransactionHeader)));
    }

    fn parse_posting(&mut self, iter: &mut Peekable<Chars>) -> Result<Posting, Error> {
        // FIXME there should be a way of doing this that doesn't introduce a new allocation
        let account = take_to_multispace(iter).trim_end().to_owned();

        consume_space(iter);

        let commodity = take_to_number(iter);

        let mut quantity: Option<i64> = None;
        let parsed = take_to_end(iter);
        if parsed.len() > 0 {
            let q: f64 = parsed
                .trim_end()
                .parse()
                .map_err(|_| self.new_error(ErrorKind::Parse(LineType::Posting)))?;
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
    fn parse_comment(&mut self, iter: &mut Peekable<Chars>) -> Option<String> {
        consume_space(iter);
        let comment = take_to_end(iter);
        match comment.len() {
            0 => None,
            _ => Some(comment),
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
        } else {
            *state = "";
        }

        Some(c)
    })
    .collect()
}

fn take_to_space(iter: &mut Peekable<Chars>) -> String {
    iter.take_while(|c| whitespace_size(c) == 0).collect()
}

fn take_to_comment_or_end(iter: &mut Peekable<Chars>) -> String {
    // TODO this implementation means that we have to trim space from the end of payees (+1
    // allocation) - is there a better way?
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
