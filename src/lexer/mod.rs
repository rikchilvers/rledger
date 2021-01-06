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

use posting::posting;
use transaction::transaction;

#[derive(Default)]
pub struct Lexer {
    line_number: u64,
}

impl Lexer {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn lex(&mut self, path: &str) -> bool {
        let file = std::fs::File::open(path).expect(&format!("file not found: {}", path));
        let reader = std::io::BufReader::new(file);

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
            return true;
        }

        if let Ok(t) = transaction(&line) {
            println!("{:?}", t)
        }

        if let Ok(t) = posting(&line) {
            println!("{:?}", t)
        }

        // let mut state = LexerState::None;
        // let mut last = "";
        // for g in line.graphemes(true) {
        //     print!("{}", g);
        //     last = g;
        // }

        true
    }
}

enum LexerState {
    None,
    InDate,
}
