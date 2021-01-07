use nom::{
    bytes::complete::take_while,
    combinator::{map, verify},
    IResult,
};

/// Takes at least two spaces or one tab
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

pub fn is_space(chr: char) -> bool {
    nom::character::is_space(chr as u8)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_handles_tags() {
        assert_eq!(whitespace2("\tsomething else"), Ok(("something else", 2)));
        assert_eq!(whitespace2("\t\tsomething else"), Ok(("something else", 4)));
    }

    #[test]
    fn it_handles_spaces() {
        // 2 spaces
        assert_eq!(whitespace2("  something else"), Ok(("something else", 2)));
        // 3 spaces
        assert_eq!(whitespace2("   something else"), Ok(("something else", 3)));
        // 4 spaces
        assert_eq!(whitespace2("    something else"), Ok(("something else", 4)));
    }

    #[test]
    fn it_handles_tags_and_spaces() {
        assert_eq!(whitespace2(" \tsomething else"), Ok(("something else", 3)));
        assert_eq!(whitespace2(" \t something else"), Ok(("something else", 4)));
        assert_eq!(whitespace2("\t something else"), Ok(("something else", 3)));
        assert_eq!(whitespace2("\t  something else"), Ok(("something else", 4)));
    }

    #[test]
    fn it_handles_not_enough_whitespace() {
        assert_eq!(
            whitespace2(" something else"),
            Err(nom::Err::Error(nom::error::Error::new(
                " something else",
                nom::error::ErrorKind::Verify
            )))
        );
    }
}
