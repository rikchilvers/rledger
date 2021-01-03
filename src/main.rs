use std::io::BufRead;

fn main() {
    let path = "tests/test.journal";
    let mut lexer = Lexer::new();
    lexer.lex(path);
}

#[derive(Default)]
struct Lexer {
    line: String,
    line_number: u64,
    column: u64,
}

impl Lexer {
    fn new() -> Self {
        Default::default()
    }

    fn lex(&mut self, path: &str) -> bool {
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

    fn lex_line(&self, line: String) -> bool {
        for c in line.chars() {
            print!("{}", c);
        }
        println!();
        true
    }

    fn next(&self) -> Option<char> {
        Some('c')
    }
}
