use std::cell::RefCell;
use std::rc::Weak;

use super::amount::Amount;
use super::transaction::Transaction;

#[derive(Debug, Default)]
pub struct Posting {
    pub path: String,
    pub amount: Option<Amount>,
    pub comments: Vec<String>,
    pub transaction: Option<Weak<RefCell<Transaction>>>,
}

impl Posting {
    pub fn add_comment(&mut self, comment: String) {
        self.comments.push(comment)
    }
}
