use std::sync::Weak;

use super::amount::Amount;
use super::Transaction;

#[derive(Debug, Default)]
pub struct Posting {
    pub path: String,
    pub amount: Option<Amount>,
    pub comments: Vec<String>,
    // Index of the transaction
    pub transaction: Option<usize>,
}

impl Posting {
    pub fn add_comment(&mut self, comment: String) {
        self.comments.push(comment)
    }
}

impl PartialEq for Posting {
    fn eq(&self, other: &Self) -> bool {
        let mut equal = false;
        equal = equal && self.path == other.path;
        equal = equal && self.amount == other.amount;
        equal = equal && self.comments == other.comments;

        // TODO: compare transactions

        equal
    }
}

impl Eq for Posting {}

impl std::fmt::Display for Posting {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut comments: Option<String> = None;
        for comment in self.comments.iter() {
            match comments {
                Some(c) => comments = Some(format!("{}\n    ; {}", c, comment)),
                None => comments = Some(format!("    ; {}", comment)),
            }
        }

        match &self.amount {
            Some(a) => match comments {
                Some(c) => return write!(f, "  {}\t{}\n{}\n", self.path, a, c),
                None => return write!(f, "  {}\t{}\n", self.path, a),
            },
            None => match comments {
                Some(c) => return write!(f, "  {}\n{}\n", self.path, c),
                None => return write!(f, "  {}\n", self.path),
            },
        }
    }
}
