use nom::{
    combinator::{map, opt},
    sequence::{preceded, tuple},
    IResult,
};

use super::{
    account::account,
    amount::amount_mapped,
    error::Error,
    error::LineType,
    peek_and_parse::{parse_line, peek_and_parse},
    whitespace::{manyspace0, whitespace2},
};

use journal::{Amount, Posting};

pub fn parse_posting(i: &str, line_number: u64) -> Result<Option<Posting>, Error> {
    parse_line(i, LineType::Posting, line_number, peek_and_parse(whitespace2, posting))
}

/// Returns the indentation level and the posting
fn posting(i: &str) -> IResult<&str, Posting> {
    preceded(
        whitespace2,
        map(
            tuple((account, opt(preceded(manyspace0, amount_mapped)))),
            |t: (&str, Option<Amount>)| Posting {
                path: t.0.to_owned(),
                amount: t.1,
                comments: vec![],
                transaction: None,
            },
        ),
    )(i)
}
