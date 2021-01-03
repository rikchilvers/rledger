use std::io::BufRead;

fn main() {
    let path = "tests/test.journal";
    lex(path);
}

fn lex(path: &str) -> bool {
    let file = std::fs::File::open(path).expect(&format!("file not found: {}", path));
    let reader = std::io::BufReader::new(file);

    let mut line_number = 0;
    for line in reader.lines() {
        line_number += 1;
        match line {
            Ok(l) => {
                if !lex_line(l) {
                    return false;
                }
            }
            Err(e) => {
                println!("{}", e);
                return false;
            }
        }
    }

    println!("There were {} lines", line_number);

    return true;
}

fn lex_line(line: String) -> bool {
    println!("{}", line);
    true
}
