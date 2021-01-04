mod lexer;

use crate::lexer::*;

fn main() {
    // let path = "tests/test.journal";
    let path = "/Users/rik/Desktop/Untitled.txt";
    let mut lexer = Lexer::new();
    lexer.lex(path);
}
