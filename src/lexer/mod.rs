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
use posting::posting;
use transaction::Transaction;
use transaction_header::transaction_header;

#[derive(Default)]
pub struct Lexer {
    state: LexerState,
    line_number: u64,
    current_transaction: Option<Transaction>,
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
            self.close_transaction();
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

            self.close_transaction();
            self.state = LexerState::InTransaction;

            self.current_transaction = Some(Transaction::from_header(t));
            println!(
                "Transaction: {}",
                self.current_transaction.as_ref().unwrap().date
            );

            return true;
        }

        // Currently, this must come before postings because that lexer will match comments greedily
        if let Ok((_, c)) = comment_min(2, &line) {
            match self.state {
                LexerState::InPosting => {
                    println!("\tposting comment: {}", c);
                }
                LexerState::InTransaction => {
                    println!("\ttransaction comment: {}", c);
                }
                _ => {
                    println!("unexpected comment");
                    return false;
                }
            }

            return true;
        }

        if let Ok((_, posting)) = posting(&line) {
            if !(self.state == LexerState::InTransaction || self.state == LexerState::InPosting) {
                println!("unexpected posting");
                return false;
            }

            self.state = LexerState::InPosting;

            if let Some(t) = &mut self.current_transaction {
                t.add_posting(posting);
                return true;
            } else {
                println!("no transaction to add posting to");
                return false;
            }
        }

        true
    }

    fn close_transaction(&mut self) {
        if self.current_transaction.is_none() {
            return;
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
