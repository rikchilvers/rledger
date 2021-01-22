use super::{comment::comment, dates::date, payee::payee, transaction_status::transaction_status};
use nom::combinator::opt;
use nom::IResult;

use crate::journal::transaction_header::TransactionHeader;
use crate::journal::transaction_status::TransactionStatus;

pub fn transaction_header(i: &str) -> IResult<&str, TransactionHeader> {
    let (i, date) = date(i)?;

    let (i, maybe_comment) = opt(comment)(i)?;
    if let Some(comment) = maybe_comment {
        let th = TransactionHeader {
            date: date.0,
            status: TransactionStatus::NoStatus,
            payee: "".to_owned(),
            comment: Some(comment.to_owned()),
        };
        return Ok((i, th));
    }

    let (i, maybe_status) = transaction_status(i)?;
    let status = maybe_status.unwrap_or(TransactionStatus::NoStatus);

    let (i, maybe_comment) = opt(comment)(i)?;
    if let Some(comment) = maybe_comment {
        let th = TransactionHeader {
            date: date.0,
            status,
            payee: "".to_owned(),
            comment: Some(comment.to_owned()),
        };
        return Ok((i, th));
    }

    let (i, payee) = payee(i)?;
    let trimmed_payee = payee.trim_end().to_owned();

    let (i, maybe_comment) = opt(comment)(i)?;
    if let Some(comment) = maybe_comment {
        let th = TransactionHeader {
            date: date.0,
            status,
            payee: trimmed_payee,
            comment: Some(comment.to_owned()),
        };
        return Ok((i, th));
    }

    let th = TransactionHeader {
        date: date.0,
        status,
        payee: trimmed_payee,
        comment: None,
    };
    Ok(("", th))
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
