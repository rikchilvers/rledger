extern crate nom;

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
    posting_depth: u8,
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
            println!("Transaction header");

            return true;
        }

        // Currently, this must come before postings because that lexer will match comments greedily
        if let Ok((_, tc)) = transaction_comment(2, self.posting_depth, &line) {
            if self.state != LexerState::InTransaction {
                println!("unexpected transaction comment");
                return false;
            }

            println!("\ttransaction comment: {}", tc);

            return true;
        }

        // Currently, this must come before postings because that lexer will match comments greedily
        if let Ok((_, pc)) = posting_comment(self.posting_depth, &line) {
            if self.state != LexerState::InTransaction {
                println!("unexpected posting comment");
                return false;
            }

            println!("\tposting comment: {}", pc);

            return true;
        }

        if let Ok((_, (depth, posting))) = posting(&line) {
            if self.state != LexerState::InTransaction {
                println!("unexpected posting");
                return false;
            }

            self.posting_depth = depth;
            println!("\tposting ({})", depth);

            return true;
        }

        true
    }

    fn close_transaction(&mut self) {
        self.posting_depth = 0;
        if self.current_transaction.is_none() {
            return;
        }
        println!("would close transaction")
    }
}

#[derive(Clone, Debug)]
struct Error {}

#[derive(Debug, PartialEq, Eq)]
enum LexerState {
    None,
    InTransaction,
    InPosting,
    InPeriodicTransaction,
}

impl Default for LexerState {
    fn default() -> Self {
        Self::None
    }
}
