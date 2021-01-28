use nom::{
    branch::alt,
    bytes::complete::tag,
    combinator::{opt, recognize, value},
    IResult,
};

use journal::transaction::Status;

pub fn transaction_status(i: &str) -> IResult<&str, Status> {
    nom::combinator::map(
        opt(alt((
            value(Status::Cleared, recognize(tag("*"))),
            value(Status::Uncleared, recognize(tag("!"))),
        ))),
        |value| match value {
            Some(v) => return v,
            None => return Status::NoStatus,
        },
    )(i)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_handles_cleared() {
        assert_eq!(transaction_status("*"), Ok(("", Status::Cleared)));
    }

    #[test]
    fn it_handles_uncleared() {
        assert_eq!(transaction_status("!"), Ok(("", Status::Uncleared)));
    }

    #[test]
    fn it_handles_none() {
        assert_eq!(
            transaction_status("something else"),
            Ok(("something else", Status::NoStatus))
        );
    }
}
