use super::amount::Amount;

#[derive(Debug, Default, Eq, PartialEq)]
pub struct Posting {
    pub path: String,
    pub amount: Option<Amount>,
    pub comments: Vec<String>,
}

impl Posting {
    pub fn add_comment(&mut self, comment: String) {
        self.comments.push(comment)
    }
}
