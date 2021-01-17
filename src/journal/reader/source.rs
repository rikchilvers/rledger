use super::{
    comment::*, error::ReaderError, include::include, posting::posting, reader::ReaderState,
    transaction_header::transaction_header,
};
use crate::journal::{posting::Posting, transaction_header::TransactionHeader};
use std::io::BufRead;
use std::io::Lines;

pub enum LineType {
    None,
    Empty,
    PostingComment(String),
    TransactionComment(String),
    TransactionHeader(TransactionHeader),
    Posting(Posting),
    IncludeDirective(String),
}

pub struct Source {
    lines: Lines<std::io::BufReader<std::fs::File>>,
    pub line_number: u64,
}

impl Source {
    pub fn new(path: &str) -> Self {
        let file = std::fs::File::open(path).expect(&format!("File not found: {}", path));

        Self {
            lines: std::io::BufReader::new(file).lines(),
            line_number: 0,
        }
    }

    // TODO: Make this an iterator?
    pub fn parse_line(&mut self, state: &ReaderState) -> Option<Result<LineType, ReaderError>> {
        match self.lines.next() {
            None => return None,
            Some(line) => match line {
                Err(e) => {
                    println!("{}", e);
                    return None;
                }
                Ok(line) => {
                    self.line_number += 1;

                    if line.len() == 0 {
                        return Some(Ok(LineType::Empty));
                    }

                    // Check for include directive
                    if let Ok((_, include)) = include(&line) {
                        if state != &ReaderState::None {
                            return Some(Err(ReaderError::UnexpectedItem(
                                "include directive".to_owned(),
                                self.line_number,
                            )));
                        }
                        return Some(Ok(LineType::IncludeDirective(include.to_owned())));
                    }

                    // Check for transaction header
                    if let Ok((_, transaction_header)) = transaction_header(&line) {
                        if state != &ReaderState::None {
                            return Some(Err(ReaderError::UnexpectedItem(
                                "transaction header".to_owned(),
                                self.line_number,
                            )));
                        }

                        return Some(Ok(LineType::TransactionHeader(transaction_header)));
                    }

                    // Check for comments
                    // This must come before postings because that lexer will match comments greedily
                    if let Ok((_, comment)) = comment_min(2, &line) {
                        match state {
                            ReaderState::InPosting => {
                                return Some(Ok(LineType::PostingComment(comment.to_owned())))
                            }
                            ReaderState::InTransaction => {
                                return Some(Ok(LineType::TransactionComment(comment.to_owned())))
                            }
                            _ => {
                                return Some(Err(ReaderError::UnexpectedItem(
                                    "comment".to_owned(),
                                    self.line_number,
                                )))
                            }
                        }
                    }

                    // Check for postings
                    if let Ok((_, posting)) = posting(&line) {
                        if state == &ReaderState::None {
                            return Some(Err(ReaderError::UnexpectedItem(
                                "posting".to_owned(),
                                self.line_number,
                            )));
                        }

                        return Some(Ok(LineType::Posting(posting)));
                    }

                    Some(Ok(LineType::None))
                }
            },
        }
    }
}
