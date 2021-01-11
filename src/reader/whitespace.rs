use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::char,
    combinator::value,
    multi::{fold_many0, many0, many1},
    sequence::terminated,
    IResult,
};

/// Counts at least 2 spaces or a tab. If successful, always returns 2.
pub fn whitespace2(i: &str) -> IResult<&str, u8> {
    value(
        2,
        terminated(
            many1(alt((tag("  "), tag("\t"), tag(" \t")))),
            many0(tag(" ")),
        ),
    )(i)
}

pub fn manyspace0(i: &str) -> IResult<&str, &str> {
    fold_many0(alt((char(' '), char('\t'))), "", |a, _| a)(i)
    // many0(alt(tag(" "), tag("\t")))(i)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_handles_tags() {
        assert_eq!(whitespace2("\tsomething else"), Ok(("something else", 2)));
        assert_eq!(whitespace2("\t\tsomething else"), Ok(("something else", 2)));
    }

    #[test]
    fn it_handles_spaces() {
        // 2 spaces
        assert_eq!(whitespace2("  something else"), Ok(("something else", 2)));
        // 3 spaces
        assert_eq!(whitespace2("   something else"), Ok(("something else", 2)));
        // 4 spaces
        assert_eq!(whitespace2("    something else"), Ok(("something else", 2)));
    }

    #[test]
    fn it_handles_tags_and_spaces() {
        assert_eq!(whitespace2(" \tsomething else"), Ok(("something else", 2)));
        assert_eq!(whitespace2(" \t something else"), Ok(("something else", 2)));
        assert_eq!(whitespace2("\t something else"), Ok(("something else", 2)));
        assert_eq!(whitespace2("\t  something else"), Ok(("something else", 2)));
    }

    #[test]
    fn it_handles_not_enough_whitespace() {
        assert_eq!(
            whitespace2(" something else"),
            Err(nom::Err::Error(nom::error::Error::new(
                " something else",
                nom::error::ErrorKind::Tag
            )))
        );
    }
}
