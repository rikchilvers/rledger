use super::Amount;
use super::Posting;

use std::cmp::Ordering;
use std::sync::Arc;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Status {
    // TODO: change to None
    NoStatus,
    Cleared,
    Uncleared,
}

impl Default for Status {
    fn default() -> Self {
        Status::NoStatus
    }
}

impl std::fmt::Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Status::NoStatus => write!(f, " "),
            Status::Cleared => write!(f, "*"),
            Status::Uncleared => write!(f, "!"),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Transaction {
    pub date: time::Date,
    // TODO: This should be optional
    pub payee: String,
    pub status: Status,
    pub header_comment: Option<String>,
    pub postings: Vec<Arc<Posting>>,
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

    /// Returns false if the posting already had an elided amount
    pub fn add_posting(&mut self, posting: Posting) -> bool {
        if posting.amount.is_none() {
            if self.elided_amount_posting_index.is_some() {
                return false;
            }
            let index = self.postings.len();
            self.elided_amount_posting_index = Some(index);
        }

        self.postings.push(Arc::new(posting));

        true
    }

    /// Returns true if the transaction was closed or false if the transaction did not balance
    pub fn close(&mut self) -> bool {
        let mut sum = 0_i64;
        for p in self.postings.iter_mut() {
            if let Some(a) = &p.amount {
                sum += a.quantity;
            }
        }

        if sum == 0 {
            return true;
        }

        // If there is no posting with an elided amount, we can't balance the transaction
        if self.elided_amount_posting_index.is_none() {
            // we step up a line here because by this point we've moved past the transaction in question
            return false;
        }

        let index = self.elided_amount_posting_index.unwrap();

        match Arc::get_mut(&mut self.postings[index]) {
            None => return false,
            Some(posting) => posting.amount = Some(Amount::new(-sum, "")),
        }

        true
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
            write!(f, "{} {} {}\n{}", self.date, self.status, self.payee, postings)
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
