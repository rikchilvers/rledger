use super::{comment::comment, dates::date, payee::payee, transaction_status::transaction_status};

use journal::Transaction;
use journal::TransactionStatus;

use nom::{
    character::complete::{multispace0, multispace1, one_of},
    combinator::{map, opt},
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
