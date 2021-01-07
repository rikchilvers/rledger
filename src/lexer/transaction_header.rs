use super::comment::journal_comment;
use super::dates::date;
use super::payee::payee;
use super::status::*;
use nom::combinator::opt;
use nom::IResult;

// TODO: first string here should be a date
// Date, Status, Payee, Comment
// pub type TransactionHeader = (String, Status, String, Option<String>);
#[derive(Default, Debug, Eq, PartialEq)]
pub struct TransactionHeader {
    pub date: String,
    pub status: Status,
    pub payee: String,
    pub comment: Option<String>,
}

impl TransactionHeader {
    pub fn new() -> Self {
        Default::default()
    }
}

pub fn transaction_header(i: &str) -> IResult<&str, TransactionHeader> {
    let (i, date) = date(i)?;

    let (i, maybe_comment) = opt(journal_comment)(i)?;
    if let Some(comment) = maybe_comment {
        let th = TransactionHeader {
            date,
            status: Status::NoStatus,
            payee: "".to_owned(),
            comment: Some(comment.to_owned()),
        };
        return Ok((i, th));
    }

    let (i, maybe_status) = status(i)?;
    let status = maybe_status.unwrap_or(Status::NoStatus);

    let (i, maybe_comment) = opt(journal_comment)(i)?;
    if let Some(comment) = maybe_comment {
        let th = TransactionHeader {
            date,
            status,
            payee: "".to_owned(),
            comment: Some(comment.to_owned()),
        };
        return Ok((i, th));
    }

    let (i, payee) = payee(i)?;
    let trimmed_payee = payee.trim_end().to_owned();

    let (i, maybe_comment) = opt(journal_comment)(i)?;
    if let Some(comment) = maybe_comment {
        let th = TransactionHeader {
            date,
            status,
            payee: trimmed_payee,
            comment: Some(comment.to_owned()),
        };
        return Ok((i, th));
    }

    let th = TransactionHeader {
        date,
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
        let date = "2020-01-01".to_owned();
        let comment = "a comment".to_owned();
        let payee = "".to_owned();

        let input = "2020-01-01 ; a comment";
        let th = TransactionHeader {
            date,
            status: Status::NoStatus,
            payee,
            comment: Some(comment),
        };

        assert_eq!(transaction_header(input), Ok(("", th)));
    }

    #[test]
    fn it_lexes_to_status() {
        let date = "2020-01-01".to_owned();
        let comment = "a comment".to_owned();
        let payee = "".to_owned();

        let input = "2020-01-01 ! ; a comment";
        let th = TransactionHeader {
            date,
            status: Status::Uncleared,
            payee,
            comment: Some(comment),
        };

        assert_eq!(transaction_header(input), Ok(("", th)));
    }

    #[test]
    fn it_lexes_to_payee() {
        let date = "2020-01-01".to_owned();
        let comment = "a comment".to_owned();
        let payee = "a payee".to_owned();

        let input = "2020-01-01 * a payee ; a comment";
        let th = TransactionHeader {
            date,
            status: Status::Cleared,
            payee,
            comment: Some(comment),
        };

        assert_eq!(transaction_header(input), Ok(("", th)));
    }
}
