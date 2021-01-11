use super::{amount::Amount, posting::Posting, transaction_status::TransactionStatus};

#[derive(Debug, Eq, PartialEq)]
pub struct Transaction {
    pub date: time::Date,
    pub payee: String,
    pub status: TransactionStatus,
    pub postings: Vec<Posting>,
    pub comments: Vec<String>,
    pub elided_amount_posting_index: Option<usize>,
}

impl Transaction {
    pub fn add_comment(&mut self, comment: String) {
        self.comments.push(comment);
    }

    pub fn close(&mut self) {
        let mut sum = 0_i64;
        for p in self.postings.iter_mut() {
            match &p.amount {
                Some(a) => sum += a.quantity,
                None => (),
            }
        }

        if sum != 0 {
            if self.elided_amount_posting_index.is_none() {
                panic!("transaction does not balance ({})\n{}", sum, self)
            }

            match self.elided_amount_posting_index {
                None => return,
                Some(index) => {
                    self.postings[index].amount = Some(Amount::new(-sum, ""));
                }
            }
        }
    }

    pub fn add_posting(&mut self, posting: Posting) {
        match posting.amount {
            Some(_) => {}
            None => {
                if self.elided_amount_posting_index.is_some() {
                    panic!("cannot add two postings with elided amounts")
                }
                self.elided_amount_posting_index = Some(self.postings.len());
            }
        }
        self.postings.push(posting);
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
