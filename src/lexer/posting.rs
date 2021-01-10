use super::{
    account::account,
    amount::{amount_mapped, Amount},
    whitespace::{manyspace0, whitespace2},
};

use nom::{
    combinator::{map_res, opt},
    sequence::{preceded, tuple},
    IResult,
};

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

impl From<(&str, Option<Amount>)> for Posting {
    fn from(lexed: (&str, Option<Amount>)) -> Self {
        Posting {
            path: lexed.0.to_owned(),
            amount: lexed.1,
            comments: vec![],
        }
    }
}

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

/// Returns the indentation level and the posting
pub fn posting(i: &str) -> IResult<&str, Posting> {
    preceded(
        whitespace2,
        map_res::<_, _, _, _, nom::error::Error<&str>, _, _>(
            tuple((account, preceded(manyspace0, opt(amount_mapped)))),
            |t: (&str, Option<Amount>)| Ok(Posting::from(t)),
        ),
    )(i)
}
