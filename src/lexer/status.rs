use nom::{
    branch::alt,
    bytes::complete::{tag, take_till},
    combinator::{opt, recognize, value},
    sequence::preceded,
    IResult,
};

use super::payee::is_not_space;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Status {
    NoStatus,
    Cleared,
    Uncleared,
}

impl Default for Status {
    fn default() -> Self {
        Status::NoStatus
    }
}

impl std::fmt::Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Status::NoStatus => write!(f, " "),
            Status::Cleared => write!(f, "*"),
            Status::Uncleared => write!(f, "!"),
        }
    }
}

pub fn status(i: &str) -> IResult<&str, Option<Status>> {
    preceded(
        take_till(is_not_space),
        opt(alt((
            value(Status::Cleared, recognize(tag("*"))),
            value(Status::Uncleared, recognize(tag("!"))),
        ))),
    )(i)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_handles_cleared() {
        assert_eq!(status("*"), Ok(("", Some(Status::Cleared))));
    }

    #[test]
    fn it_handles_uncleared() {
        assert_eq!(status("!"), Ok(("", Some(Status::Uncleared))));
    }

    #[test]
    fn it_handles_none() {
        assert_eq!(status("something else"), Ok(("something else", None)));
    }
}
