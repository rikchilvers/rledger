use super::{
    account::account,
    amount::amount_mapped,
    whitespace::{manyspace0, whitespace2},
};
use crate::journal::{Amount, Posting};

use nom::{
    combinator::{map_res, opt},
    sequence::{preceded, tuple},
    IResult,
};

impl From<(&str, Option<Amount>)> for Posting {
    fn from(lexed: (&str, Option<Amount>)) -> Self {
        Posting {
            path: lexed.0.to_owned(),
            amount: lexed.1,
            comments: vec![],
            transaction: None,
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
