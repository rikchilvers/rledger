use nom::{
    branch::alt,
    bytes::complete::tag,
    multi::{many0, many1},
    sequence::terminated,
    IResult,
};

/// Counts at least 2 spaces or a tab
pub fn whitespace(i: &str) -> IResult<&str, Vec<&str>> {
    terminated(
        many1(alt((tag("  "), tag("\t"), tag(" \t")))),
        many0(tag(" ")),
    )(i)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_handles_tags() {
        assert_eq!(
            whitespace("\tsomething else"),
            Ok(("something else", vec!["\t"]))
        );
        assert_eq!(
            whitespace("\t\tsomething else"),
            Ok(("something else", vec!["\t", "\t"]))
        );
    }

    #[test]
    fn it_handles_spaces() {
        assert_eq!(
            // 2 spaces
            whitespace("  something else"),
            Ok(("something else", vec!["  "]))
        );
        assert_eq!(
            // 3 spaces
            whitespace("   something else"),
            Ok(("something else", vec!["  "]))
        );
        assert_eq!(
            // 4 spaces
            whitespace("    something else"),
            Ok(("something else", vec!["  ", "  "]))
        );
    }

    #[test]
    fn it_handles_tags_and_spaces() {
        assert_eq!(
            whitespace(" \tsomething else"),
            Ok(("something else", vec![" \t"]))
        );

        assert_eq!(
            whitespace(" \t something else"),
            Ok(("something else", vec![" \t"]))
        );

        assert_eq!(
            whitespace("\t something else"),
            Ok(("something else", vec!["\t"]))
        );

        assert_eq!(
            whitespace("\t  something else"),
            Ok(("something else", vec!["\t", "  "]))
        );
    }

    #[test]
    fn it_handles_not_enough_whitespace() {
        assert_eq!(
            whitespace(" something else"),
            Err(nom::Err::Error(nom::error::Error::new(
                " something else",
                nom::error::ErrorKind::Tag
            )))
        );
    }
}
