use super::{
    comment::*, include::include, posting::posting, transaction_header::transaction_header,
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
        loop {
            match self.lines.next() {
                None => return None,
                Some(line) => match line {
                    Ok(line) => {
                        self.line_number += 1;
                        match self.read_line(&line) {
                            Ok(transaction) => match transaction {
                                None => continue,
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

    fn read_line(&mut self, line: &str) -> Result<Option<Rc<RefCell<Transaction>>>, &str> {
        let mut completed_transaction: Option<Rc<RefCell<Transaction>>> = None;

        if line.len() == 0 {
            self.add_posting();
            if let Some(t) = &mut self.current_transaction.take() {
                t.borrow_mut().close();
                completed_transaction = Some(t.clone());
            }
            self.state = ReaderState::None;
            return Ok(completed_transaction);
        }

        if let Ok((_, include)) = include(&line) {
            println!(">> include: {}", include);
            return Ok(completed_transaction);
        }

        if let Ok((_, t)) = transaction_header(&line) {
            if self.state != ReaderState::None {
                return Err("unexpected transaction header");
            }

            self.add_posting();
            if let Some(ref mut t) = &mut self.current_transaction {
                t.borrow_mut().close();
                completed_transaction = Some(t.clone());
            }
            self.current_transaction = None;

            self.state = ReaderState::InTransaction;
            self.current_transaction = Some(Rc::new(RefCell::new(Transaction::from_header(t))));

            return Ok(completed_transaction);
        }

        // Currently, this must come before postings because that lexer will match comments greedily
        if let Ok((_, c)) = comment_min(2, &line) {
            match &mut self.state {
                ReaderState::InPosting => match &mut self.current_posting {
                    Some(p) => p.add_comment(c.to_owned()),
                    None => return Err("no posting"),
                },
                ReaderState::InTransaction => match &mut self.current_transaction {
                    Some(transaction) => transaction.borrow_mut().add_comment(c.to_owned()),
                    None => return Err("couldn't add transaction comment"),
                },
                _ => return Err("unexpected comment"),
            }

            return Ok(completed_transaction);
        }

        if let Ok((_, posting)) = posting(&line) {
            if self.state == ReaderState::None {
                return Err("unexpected posting");
            }

            // If we're already in a posting, we need to add it to the current transaction
            self.add_posting();

            self.state = ReaderState::InPosting;
            self.current_posting = Some(posting);
        }

        Ok(completed_transaction)
    }

    fn add_posting(&mut self) -> bool {
        if let Some(mut current_posting) = self.current_posting.take() {
            if let Some(t) = &mut self.current_transaction {
                current_posting.transaction = Some(Rc::downgrade(t));
                t.borrow_mut().add_posting(current_posting);
                return true;
            } else {
                println!("no transaction to add posting to");
                return false;
            }
        }

        return true;
    }
}
