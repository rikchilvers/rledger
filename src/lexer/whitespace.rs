use nom::{
    branch::alt,
    bytes::complete::tag,
    bytes::complete::take_while,
    combinator::{map, value, verify},
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

pub fn whitespace4(i: &str) -> IResult<&str, ()> {
    value(
        (),
        verify(take_while(is_space), |s: &str| {
            let mut count = 0;
            for c in s.chars() {
                match c {
                    ' ' => count += 1,
                    '\t' => count += 2,
                    _ => (),
                }
                if count >= 4 {
                    return true;
                }
            }
            count >= 4
        }),
    )(i)
}

/// Takes at least two spaces
pub fn whitespace2(i: &str) -> IResult<&str, u8> {
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
pub fn whitespacex(n: i8, i: &str) -> impl Fn(&str) -> IResult<&str, i8> {
    move |i: &str| {
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
        })(i)
    }
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
