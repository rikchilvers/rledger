use std::iter::Peekable;
use std::path::Path;
use std::path::PathBuf;
use std::str::Chars;
use std::sync::Arc;

use super::bufreader::BufReader;

pub struct Source {
    location: Arc<PathBuf>,
    contents: BufReader,
}

impl Source {
    pub fn new<P: Into<PathBuf> + AsRef<Path> + Copy>(path: P) -> Self {
        Self {
            location: Arc::new(path.into()),
            contents: BufReader::open(path).unwrap(),
        }
    }

    pub fn lex(&mut self) {
        match self.contents.next() {
            None => return,
            Some(line) => match line {
                Err(e) => panic!("NR source error: {}", e),
                Ok(line) => {
                    self.lex_line(&line);
                    self.lex();
                }
            },
        }
    }

    fn lex_line(&mut self, line: &str) {
        let mut iter = line.chars().peekable();

        match iter.peek() {
            None => return,
            Some(c) if c.is_numeric() => {
                self.lex_transaction_header(&mut iter);
            }
            Some(c) if c.is_whitespace() => {
                self.lex_posting(&mut iter);
            }
            Some(c) if c == &'i' => {
                let include = self.lex_include_directive(&mut iter);
                println!("include {}", include);
            }
            Some(c) if c == &';' => {
                // println!("\ncomment")
            }
            Some(c) if c == &'~' => {
                println!("periodic transaction header")
            }
            _ => println!("unhandled line type"),
        }
    }

    fn lex_include_directive(&mut self, iter: &mut Peekable<Chars>) -> String {
        take_to_space(iter);
        take_to_end(iter)
    }

    fn lex_transaction_header(&mut self, iter: &mut Peekable<Chars>) {
        let date = take_to_space(iter);

        consume_space(iter);

        let mut status: Option<u8> = None;
        match iter.peek() {
            None => panic!(""),
            Some(c) if is_status(c) => {
                status = Some(1);
                iter.next();
                consume_space(iter);
            }
            _ => {}
        }

        let payee = take_to_comment_or_end(iter);

        let comment = self.lex_comment(iter);

        println!("\n{} {:?} '{}' ; {}", date, status, payee.trim_end(), comment);
    }

    fn lex_posting(&mut self, iter: &mut Peekable<Chars>) {
        let spaces = consume_space(iter);
        if spaces < 2 {
            panic!("not enough spaces starting a posting")
        }

        if let Some(c) = iter.peek() {
            if is_comment_indicator(c) {
                self.lex_comment(iter);
                return;
            }
        }

        let account = take_to_multispace(iter);

        consume_space(iter);

        let commodity = take_to_number(iter);

        let quantity = take_to_end(iter);

        println!("  {}\t{} {}", account, commodity, quantity);
    }

    fn lex_comment(&mut self, iter: &mut Peekable<Chars>) -> String {
        consume_space(iter);
        take_to_end(iter)
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
    iter.take_while(|c| !c.is_numeric() || c != &'+' || c != &'-').collect()
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
