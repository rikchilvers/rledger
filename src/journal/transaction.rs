use super::{posting::Posting, transaction_status::TransactionStatus};
use std::rc::Rc;

#[derive(Debug)]
pub struct Transaction {
    pub date: time::Date,
    pub payee: String,
    pub status: TransactionStatus,
    pub postings: Vec<Rc<Posting>>,
    pub comments: Vec<String>,
    pub elided_amount_posting_index: Option<usize>,
}

impl Transaction {
    pub fn new() -> Self {
        Self {
            date: time::Date::try_from_ymd(0000, 01, 01).expect("oops"),
            payee: String::from(""),
            status: TransactionStatus::NoStatus,
            postings: vec![],
            comments: vec![],
            elided_amount_posting_index: None,
        }
    }
}

impl std::fmt::Display for Transaction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut comments: Option<String> = None;
        for comment in self.comments.iter() {
            match comments {
                Some(c) => comments = Some(format!("{}\n  ; {}", c, comment)),
                None => comments = Some(format!("  ; {}", comment)),
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
    use crate::journal::amount::Amount;

    #[test]
    fn it_works() {
        /*
        let th = TransactionHeader {
            date: time::Date::try_from_ymd(2020, 01, 01).unwrap(),
            payee: "A Shop".to_owned(),
            status: TransactionStatus::Cleared,
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
        */
        // let expected = Transaction::from_header_and_postings((th, ps));
        // let input = "2020-01-01 * A Shop\n\tAssets:Current  £15\n";
        // let t = transaction(input);
        // assert_eq!(transaction(input), Ok(((""), expected)));
    }
}
