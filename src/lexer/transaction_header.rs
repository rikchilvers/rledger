use super::comment::comment;
use super::dates::date;
use super::payee::payee;
use super::status::*;
use nom::combinator::opt;
use nom::IResult;

// TODO: first string here should be a date
type TransactionHeader = (String, Status, String, Option<String>);

fn transaction_header(i: &str) -> IResult<&str, TransactionHeader> {
    let (i, date) = date(i)?;

    let (i, maybe_comment) = opt(comment)(i)?;
    if let Some(comment) = maybe_comment {
        let th: TransactionHeader = (
            date,
            Status::NoStatus,
            "".to_owned(),
            Some(comment.to_owned()),
        );
        return Ok((i, th));
    }

    let (i, maybe_status) = status(i)?;
    let status = maybe_status.unwrap_or(Status::NoStatus);

    let (i, maybe_comment) = opt(comment)(i)?;
    if let Some(comment) = maybe_comment {
        let th: TransactionHeader = (date, status, "".to_owned(), Some(comment.to_owned()));
        return Ok((i, th));
    }

    let (i, payee) = payee(i)?;
    let trimmed_payee = payee.trim_end().to_owned();

    let (i, maybe_comment) = opt(comment)(i)?;
    if let Some(comment) = maybe_comment {
        let th: TransactionHeader = (date, status, trimmed_payee, Some(comment.to_owned()));
        return Ok((i, th));
    }

    let th: TransactionHeader = (date, status, trimmed_payee, None);
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
        let th: TransactionHeader = (date, Status::NoStatus, payee, Some(comment));

        assert_eq!(transaction_header(input), Ok(("", th)));
    }

    #[test]
    fn it_lexes_to_status() {
        let date = "2020-01-01".to_owned();
        let comment = "a comment".to_owned();
        let payee = "".to_owned();

        let input = "2020-01-01 ! ; a comment";
        let th: TransactionHeader = (date, Status::Uncleared, payee, Some(comment));

        assert_eq!(transaction_header(input), Ok(("", th)));
    }

    #[test]
    fn it_lexes_to_payee() {
        let date = "2020-01-01".to_owned();
        let comment = "a comment".to_owned();
        let payee = "a payee".to_owned();

        let input = "2020-01-01 * a payee ; a comment";
        let th: TransactionHeader = (date, Status::Cleared, payee, Some(comment));

        assert_eq!(transaction_header(input), Ok(("", th)));
    }
}
