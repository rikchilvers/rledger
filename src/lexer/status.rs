use nom::{
    branch::alt,
    bytes::complete::tag,
    combinator::{opt, recognize, value},
    IResult,
};

fn status(i: &str) -> IResult<&str, Option<Status>> {
    opt(alt((
        value(Status::Cleared, recognize(tag("*"))),
        value(Status::Uncleared, recognize(tag("!"))),
    )))(i)
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Status {
    None,
    Cleared,
    Uncleared,
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
