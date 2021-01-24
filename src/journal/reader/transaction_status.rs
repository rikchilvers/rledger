use nom::{
    branch::alt,
    bytes::complete::tag,
    combinator::{opt, recognize, value},
    IResult,
};

use crate::journal::transaction_status::TransactionStatus;

pub fn transaction_status(i: &str) -> IResult<&str, TransactionStatus> {
    nom::combinator::map(
        opt(alt((
            value(TransactionStatus::Cleared, recognize(tag("*"))),
            value(TransactionStatus::Uncleared, recognize(tag("!"))),
        ))),
        |value| match value {
            Some(v) => return v,
            None => return TransactionStatus::NoStatus,
        },
    )(i)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_handles_cleared() {
        assert_eq!(transaction_status("*"), Ok(("", TransactionStatus::Cleared)));
    }

    #[test]
    fn it_handles_uncleared() {
        assert_eq!(transaction_status("!"), Ok(("", TransactionStatus::Uncleared)));
    }

    #[test]
    fn it_handles_none() {
        assert_eq!(
            transaction_status("something else"),
            Ok(("something else", TransactionStatus::NoStatus))
        );
    }
}
