use nom::{
    character::complete::{one_of, space0},
    combinator::rest,
    sequence::{preceded, tuple},
    IResult,
};

use super::error::Error;
use super::error::LineType;
use super::peek_and_parse::*;

use super::whitespace::*;

pub fn parse_comment(i: &str, line_number: u64) -> Result<Option<&str>, Error> {
    parse_line(
        i,
        LineType::Comment,
        line_number,
        peek_and_parse(tuple((whitespace2, one_of(";#"))), preceded(whitespace2, comment)),
    )
}

pub fn comment(i: &str) -> IResult<&str, &str> {
    preceded(one_of(";#"), preceded(space0, rest))(i)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_finds_colon_comments() {
        assert_eq!(comment("; comment"), Ok(("", "comment")));
        assert_eq!(comment(";comment"), Ok(("", "comment")));
    }

    #[test]
    fn it_finds_hash_comments() {
        assert_eq!(comment("# comment"), Ok(("", "comment")));
        assert_eq!(comment("#comment"), Ok(("", "comment")));
    }
}
