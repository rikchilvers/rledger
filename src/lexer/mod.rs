extern crate nom;

mod dates;
mod status;
mod whitespace;

use self::whitespace::*;
use std::io::BufRead;

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

        true
    }
}
