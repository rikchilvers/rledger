use super::Status;
use std::cmp::Ordering;

use super::Posting;

#[derive(Debug, PartialEq, Eq)]
pub struct Transaction {
    pub date: time::Date,
    // TODO: This should be optional
    pub payee: String,
    pub status: Status,
    pub header_comment: Option<String>,
    /// Indexes of the postings vec
    pub postings: Vec<usize>,
    pub comments: Vec<String>,
    pub elided_amount_posting_index: Option<usize>,
}

impl Transaction {
    pub fn new() -> Self {
        Self {
            date: time::Date::try_from_ymd(2020, 01, 01).unwrap(),
            status: Status::NoStatus,
            payee: String::from(""),
            header_comment: None,
            postings: vec![],
            comments: vec![],
            elided_amount_posting_index: None,
        }
    }

    pub fn display(&self, postings: &[Posting]) {
        let mut comments: Option<String> = None;
        for comment in self.comments.iter() {
            match comments {
                Some(c) => comments = Some(format!("{}\n  ; {}", c, comment)),
                None => comments = Some(format!("  ; {}", comment)),
            }
        }

        let mut posting_output = String::new();
        for p_idx in self.postings.iter() {
            posting_output.push_str(&postings[*p_idx].to_string())
        }

        match comments {
            Some(c) => {
                println!(
                    "{} {} {}\n{}\n{}",
                    self.date, self.status, self.payee, c, posting_output,
                )
            }
            None => {
                println!("{} {} {}\n{}", self.date, self.status, self.payee, posting_output,)
            }
        }
    }
}

impl Ord for Transaction {
    fn cmp(&self, other: &Self) -> Ordering {
        self.date.cmp(&other.date)
    }
}

impl PartialOrd for Transaction {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::amount::Amount;

    #[test]
    fn it_works() {
        /*
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
        */
        // let expected = Transaction::from_header_and_postings((th, ps));
        // let input = "2020-01-01 * A Shop\n\tAssets:Current  £15\n";
        // let t = transaction(input);
        // assert_eq!(transaction(input), Ok(((""), expected)));
    }
}
