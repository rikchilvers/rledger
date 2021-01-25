use super::{
    account::account,
    amount::amount_mapped,
    whitespace::{manyspace0, whitespace2},
};
use journal::{Amount, Posting};

use nom::{
    combinator::{map, opt},
    sequence::{preceded, tuple},
    IResult,
};

fn posting_from_lexed(lexed: (&str, Option<Amount>)) -> Posting {
    Posting {
        path: lexed.0.to_owned(),
        amount: lexed.1,
        comments: vec![],
        transaction: None,
    }
}

/// Returns the indentation level and the posting
pub fn posting(i: &str) -> IResult<&str, Posting> {
    preceded(
        whitespace2,
        map(
            tuple((account, opt(preceded(manyspace0, amount_mapped)))),
            |t: (&str, Option<Amount>)| posting_from_lexed(t),
        ),
    )(i)
}
