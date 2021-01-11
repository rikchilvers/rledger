mod journal;
mod reader;

use crate::reader::Reader;

fn main() {
    // let path = "tests/test.journal";
    let path = "/Users/rik/Documents/Personal/Finance/current.journal";
    let mut reader = Reader::new();
    reader.lex(path);
}
