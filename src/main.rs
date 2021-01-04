extern crate nom;

use nom::{
    bytes::complete::is_a,
    // character::complete::space0,
    // character::is_space,
    // combinator::map,
    // error::{context, ErrorKind, VerboseError},
    // multi::{count, many0},
    IResult,
};
use std::io::BufRead;

fn main() {
    // let path = "tests/test.journal";
    let path = "/Users/rik/Desktop/Untitled.txt";
    let mut lexer = Lexer::new();
    lexer.lex(path);
}

#[derive(Default)]
struct Lexer {
    state: LexerState,
    field: Vec<char>,
    line_number: u64,
    column: u64,
}

enum LexerState {
    ConsumingSpace,
}

impl Default for LexerState {
    fn default() -> Self {
        LexerState::ConsumingSpace
    }
}

impl Lexer {
    fn new() -> Self {
        Lexer {
            column: 1,
            ..Default::default()
        }
    }

    fn lex(&mut self, path: &str) -> bool {
        let file = std::fs::File::open(path).expect(&format!("file not found: {}", path));
        let reader = std::io::BufReader::new(file);

        for line in reader.lines() {
            self.line_number += 1;
            self.field.clear();

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

        // let spaces = count_whitespace(&line);
        let spaces = count_whitespace_multi(&line);

        if let Ok(s) = spaces {
            println!("{} spaces followed by {}", s.1, s.0);
        }

        true
    }
}

fn count_whitespace_multi(i: &str) -> IResult<&str, u64> {
    nom::multi::fold_many0(count_whitespace, 0, |mut acc: u64, item| {
        acc += item;
        acc
    })(i)
}

fn count_whitespace(i: &str) -> IResult<&str, u64> {
    nom::branch::alt((count_spaces, count_tabs))(i)
}

fn count_spaces(i: &str) -> IResult<&str, u64> {
    nom::combinator::map(is_a("\t"), |s: &str| (s.len() * 2) as u64)(i)
}

fn count_tabs(i: &str) -> IResult<&str, u64> {
    nom::combinator::map(is_a(" "), |s: &str| (s.len() as u64))(i)
}
