use super::{posting::*, status::Status, transaction_header::*};

#[derive(Debug, Eq, PartialEq)]
pub struct Transaction {
    pub date: time::Date,
    pub payee: String,
    pub status: Status,
    pub postings: Vec<Posting>,
    pub comments: Vec<String>,
}

impl Transaction {
    pub fn from_header(header: TransactionHeader) -> Self {
        Transaction {
            date: header.date,
            status: header.status,
            payee: header.payee,
            postings: vec![],
            comments: vec![],
        }
    }

    pub fn add_comment(&mut self, comment: String) {
        self.comments.push(comment);
    }

    pub fn add_posting(&mut self, posting: Posting) {
        self.postings.push(posting);
    }
}

impl std::fmt::Display for Transaction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut comments: Option<String> = None;
        for comment in self.comments.iter() {
            match comments {
                Some(c) => comments = Some(format!("{}\n\t; {}", c, comment)),
                None => comments = Some(format!("\t; {}", comment)),
            }
        }

        let mut postings = String::new();
        for p in self.postings.iter() {
            postings.push_str(&p.to_string())
        }

        if let Some(comments) = comments {
            write!(
                f,
                "{} {} {}\n{}\n{}",
                self.date, self.status, self.payee, comments, postings
            )
        } else {
            write!(
                f,
                "{} {} {}\n{}",
                self.date, self.status, self.payee, postings
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::amount::Amount;

    #[test]
    fn it_works() {
        let th = TransactionHeader {
            date: time::Date::try_from_ymd(2020, 01, 01).unwrap(),
            payee: "A Shop".to_owned(),
            status: Status::Cleared,
            comment: None,
        };
        let ps = vec![Posting {
            path: "Assets:Current".to_owned(),
            amount: Some(Amount {
                commodity: "£".to_owned(),
                quantity: 1500,
            }),
            comments: vec![],
        }];
        // let expected = Transaction::from_header_and_postings((th, ps));
        // let input = "2020-01-01 * A Shop\n\tAssets:Current  £15\n";
        // let t = transaction(input);
        // assert_eq!(transaction(input), Ok(((""), expected)));
    }
}
