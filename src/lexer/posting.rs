use super::{
    account::account,
    amount::{amount_mapped, Amount},
    whitespace::whitespace,
};

use nom::{
    combinator::{map_res, opt},
    sequence::{preceded, tuple},
    IResult,
};

#[derive(Debug, Default, Eq, PartialEq)]
pub struct Posting {
    path: String,
    amount: Option<Amount>,
}

impl Posting {
    pub fn new() -> Self {
        Default::default()
    }
}

impl From<(&str, Option<Amount>)> for Posting {
    fn from(lexed: (&str, Option<Amount>)) -> Self {
        Posting {
            path: lexed.0.to_owned(),
            amount: lexed.1,
        }
    }
}

pub fn posting(i: &str) -> IResult<&str, Posting> {
    preceded(
        whitespace,
        map_res::<_, _, _, _, nom::error::Error<&str>, _, _>(
            tuple((account, opt(amount_mapped))),
            |t: (&str, Option<Amount>)| Ok(Posting::from(t)),
        ),
    )(i)
}
