use nom::{
    branch::alt,
    bytes::complete::{tag, take_till},
    combinator::{opt, recognize, value},
    sequence::preceded,
    IResult,
};

use super::payee::is_not_space;
use crate::journal::transaction_status::TransactionStatus;

pub fn transaction_status(i: &str) -> IResult<&str, Option<TransactionStatus>> {
    preceded(
        take_till(is_not_space),
        opt(alt((
            value(TransactionStatus::Cleared, recognize(tag("*"))),
            value(TransactionStatus::Uncleared, recognize(tag("!"))),
        ))),
    )(i)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_handles_cleared() {
        assert_eq!(
            transaction_status("*"),
            Ok(("", Some(TransactionStatus::Cleared)))
        );
    }

    #[test]
    fn it_handles_uncleared() {
        assert_eq!(
            transaction_status("!"),
            Ok(("", Some(TransactionStatus::Uncleared)))
        );
    }

    #[test]
    fn it_handles_none() {
        assert_eq!(
            transaction_status("something else"),
            Ok(("something else", None))
        );
    }
}
