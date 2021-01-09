use nom::{
    branch::alt,
    bytes::complete::{tag, take_while},
    combinator::{map, value, verify},
    multi::{many0, many1},
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

/// Takes at least two spaces or one tab
pub fn whitespace2_count(i: &str) -> IResult<&str, u8> {
    verify(
        map(take_while(is_space), |s: &str| {
            let mut count = 0;
            for c in s.chars() {
                match c {
                    ' ' => count += 1,
                    '\t' => count += 2,
                    _ => (),
                }
            }
            count
        }),
        |c| *c >= 2,
    )(i)
}

pub fn is_space(chr: char) -> bool {
    nom::character::is_space(chr as u8)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_handles_tags() {
        assert_eq!(
            whitespace2_count("\tsomething else"),
            Ok(("something else", 2))
        );
        assert_eq!(
            whitespace2_count("\t\tsomething else"),
            Ok(("something else", 4))
        );
    }

    #[test]
    fn it_handles_spaces() {
        // 2 spaces
        assert_eq!(
            whitespace2_count("  something else"),
            Ok(("something else", 2))
        );
        // 3 spaces
        assert_eq!(
            whitespace2_count("   something else"),
            Ok(("something else", 3))
        );
        // 4 spaces
        assert_eq!(
            whitespace2_count("    something else"),
            Ok(("something else", 4))
        );
    }

    #[test]
    fn it_handles_tags_and_spaces() {
        assert_eq!(
            whitespace2_count(" \tsomething else"),
            Ok(("something else", 3))
        );
        assert_eq!(
            whitespace2_count(" \t something else"),
            Ok(("something else", 4))
        );
        assert_eq!(
            whitespace2_count("\t something else"),
            Ok(("something else", 3))
        );
        assert_eq!(
            whitespace2_count("\t  something else"),
            Ok(("something else", 4))
        );
    }

    #[test]
    fn it_handles_not_enough_whitespace() {
        assert_eq!(
            whitespace2_count(" something else"),
            Err(nom::Err::Error(nom::error::Error::new(
                " something else",
                nom::error::ErrorKind::Verify
            )))
        );
    }
}
