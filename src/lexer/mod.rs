extern crate nom;
extern crate time;

mod account;
mod amount;
mod comment;
mod dates;
mod include;
mod payee;
mod posting;
mod status;
mod transaction;
mod transaction_header;
mod whitespace;

use std::io::BufRead;

use comment::*;
use include::include;
use posting::{posting, Posting};
use transaction::Transaction;
use transaction_header::transaction_header;

#[derive(Default)]
pub struct Lexer {
    state: LexerState,
    line_number: u64,
    current_transaction: Option<Transaction>,
    current_posting: Option<Posting>,
}

impl Lexer {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn lex(&mut self, path: &str) -> bool {
        let file = std::fs::File::open(path).expect(&format!("file not found: {}", path));
        let reader = std::io::BufReader::new(file);

        println!("Reading {}", path);

        for line in reader.lines() {
            self.line_number += 1;

            match line {
                Ok(l) => {
                    if !self.lex_line(l) {
                        return false;
                    }
                }
                Err(e) => {
                    println!("{}", e);
                    return false;
                }
            }
        }

        println!("There were {} lines", self.line_number);

        return true;
    }

    fn lex_line(&mut self, line: String) -> bool {
        if line.len() == 0 {
            self.add_posting();
            self.close_transaction();
            self.current_transaction = None;
            self.state = LexerState::None;
            return true;
        }

        if let Ok((_, include)) = include(&line) {
            println!(">> include: {}", include);
            return true;
        }

        if let Ok((_, t)) = transaction_header(&line) {
            if self.state != LexerState::None {
                println!("unexpected transaction header");
                return false;
            }

            self.add_posting();
            self.close_transaction();

            self.state = LexerState::InTransaction;
            self.current_transaction = Some(Transaction::from_header(t));

            return true;
        }

        // Currently, this must come before postings because that lexer will match comments greedily
        if let Ok((_, c)) = comment_min(2, &line) {
            match &mut self.state {
                LexerState::InPosting => {
                    if let Some(p) = &mut self.current_posting {
                        p.add_comment(c.to_owned());
                    }
                }
                LexerState::InTransaction => {
                    if let Some(transaction) = &mut self.current_transaction {
                        transaction.add_comment(c.to_owned())
                    } else {
                        println!("couldn't add transaction comment");
                    }
                }
                _ => {
                    println!("unexpected comment");
                    return false;
                }
            }

            return true;
        }

        if let Ok((_, posting)) = posting(&line) {
            if self.state == LexerState::None {
                println!("unexpected posting");
                return false;
            }

            // If we're already in a posting, we need to add it to the current transaction
            self.add_posting();

            self.state = LexerState::InPosting;
            self.current_posting = Some(posting);
        }

        true
    }

    fn add_posting(&mut self) -> bool {
        if let Some(current_posting) = self.current_posting.take() {
            if let Some(t) = &mut self.current_transaction {
                t.add_posting(current_posting);
                return true;
            } else {
                println!("no transaction to add posting to");
                return false;
            }
        }

        return true;
    }

    // TODO move inside Transaction
    fn close_transaction(&mut self) {
        match &mut self.current_transaction {
            Some(t) => {
                let mut sum = 0_i64;
                for p in t.postings.iter_mut() {
                    match &p.amount {
                        Some(a) => sum += a.quantity,
                        None => (),
                    }
                }

                if sum != 0 {
                    // TODO remove this once fn is inside Transaction impl
                    if !t.has_posting_with_elided_amount() {
                        panic!("transaction does not balance ({})\n{}", sum, t)
                    }

                    t.balance_elided_posting(-sum);
                }

                println!("{}", self.current_transaction.as_ref().unwrap());
                self.current_transaction = None;
            }
            None => return,
        }
    }
}

#[derive(Clone, Debug)]
struct Error {}

#[derive(Debug, PartialEq, Eq)]
enum LexerState {
    None,
    InTransaction,
    InPosting,
}

impl Default for LexerState {
    fn default() -> Self {
        Self::None
    }
}
