use super::{comment::comment, dates::date, payee::payee, transaction_status::transaction_status};

use journal::Transaction;
use journal::TransactionStatus;

use nom::{
    bytes::complete::tag,
    character::complete::{multispace0, multispace1, one_of},
    combinator::{cut, map, opt, peek},
    sequence::{preceded, tuple},
    IResult,
};

#[derive(Debug, Eq, PartialEq)]
pub struct TransactionHeader {
    pub date: time::Date,
    pub status: TransactionStatus,
    // TODO this should be optional
    pub payee: String,
    pub comment: Option<String>,
}

pub fn transaction_from_header(header: TransactionHeader) -> Transaction {
    Transaction {
        date: header.date,
        status: header.status,
        payee: header.payee,
        postings: vec![],
        comments: vec![],
        elided_amount_posting_index: None,
    }
}

pub fn error_transformer<I: Clone, O1, O2, E: nom::error::ParseError<I>, F, G>(
    marker: F,
    parser: G,
) -> impl FnMut(I) -> IResult<I, O2, E>
where
    F: nom::Parser<I, O1, E>,
    G: nom::Parser<I, O2, E>,
{
    preceded(peek(marker), cut(parser))
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum VerificationResult<I, O> {
    Ok((I, O)), // the parser worked
    Error,      // the parser failed
    NoMatch,    // the parser did not match the input
}

pub fn verify_fn<'a, O>(
    input: &'a str,
    mut parser: impl FnMut(&'a str) -> IResult<&'a str, O>,
) -> VerificationResult<&'a str, O> {
    match parser(input) {
        Ok((remaining, output)) => return VerificationResult::Ok((remaining, output)),
        Err(e) => match e {
            nom::Err::Error(e) => {
                if e.input.len() != input.len() {
                    return VerificationResult::Error;
                }
                return VerificationResult::NoMatch;
            }
            _ => unimplemented!(),
        },
    }
}
pub fn verify<'a, O>(input: &str, parse_result: IResult<&'a str, O>) -> VerificationResult<&'a str, O> {
    match parse_result {
        Ok((remaining, output)) => return VerificationResult::Ok((remaining, output)),
        Err(e) => match e {
            nom::Err::Error(e) => {
                if e.input.len() != input.len() {
                    return VerificationResult::Error;
                }
                return VerificationResult::NoMatch;
            }
            _ => unimplemented!(),
        },
    }
}

fn t_h(i: &str) -> IResult<&str, &str> {
    preceded(one_of("0123456789"), tag("abc"))(i)
}

pub fn transaction_header(i: &str) -> IResult<&str, TransactionHeader> {
    map(
        tuple((
            date,
            opt(preceded(multispace1, transaction_status)),
            opt(preceded(multispace1, payee)),
            opt(preceded(multispace0, comment)),
        )),
        |parsed: ((time::Date, _), Option<TransactionStatus>, Option<&str>, Option<&str>)| {
            return TransactionHeader {
                date: parsed.0 .0,
                status: parsed.1.unwrap_or_default(),
                payee: parsed.2.unwrap_or("").trim_end().to_owned(),
                comment: parsed.3.map(|s| s.to_owned()),
            };
        },
    )(i)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_th() {
        let input = "0abc";
        assert_eq!(verify_fn("0abc", t_h), VerificationResult::Ok(("", "abc")));
        assert_eq!(verify_fn("0abc0", t_h), VerificationResult::Ok(("0", "abc")));
        assert_eq!(verify_fn("0abd", t_h), VerificationResult::Error);
        assert_eq!(verify_fn("abc", t_h), VerificationResult::NoMatch);
    }

    #[test]
    fn it_lexes_just_date() {
        let date = time::Date::try_from_ymd(2020, 01, 01).unwrap();
        let comment = "a comment".to_owned();
        let payee = "".to_owned();

        let input = "2020-01-01 ; a comment";
        let th = TransactionHeader {
            date,
            status: TransactionStatus::NoStatus,
            payee,
            comment: Some(comment),
        };

        assert_eq!(transaction_header(input), Ok(("", th)));
    }

    #[test]
    fn it_lexes_to_status() {
        let date = time::Date::try_from_ymd(2020, 01, 01).unwrap();
        let comment = "a comment".to_owned();
        let payee = "".to_owned();

        let input = "2020-01-01 ! ; a comment";
        let th = TransactionHeader {
            date,
            status: TransactionStatus::Uncleared,
            payee,
            comment: Some(comment),
        };

        assert_eq!(transaction_header(input), Ok(("", th)));
    }

    #[test]
    fn it_lexes_to_payee() {
        let date = time::Date::try_from_ymd(2020, 01, 01).unwrap();
        let comment = "a comment".to_owned();
        let payee = "a payee".to_owned();

        let input = "2020-01-01 * a payee ; a comment";
        let th = TransactionHeader {
            date,
            status: TransactionStatus::Cleared,
            payee,
            comment: Some(comment),
        };

        assert_eq!(transaction_header(input), Ok(("", th)));
    }
}
