extern crate nom;
use nom::{
    bytes::complete::is_a,
    // character::complete::space0,
    // character::is_space,
    combinator::map,
    // error::{context, ErrorKind, VerboseError},
    // multi::{count, many0},
    IResult,
};

pub fn count_whitespace_multi(i: &str) -> IResult<&str, u64> {
    nom::multi::fold_many0(count_whitespace, 0, |mut acc: u64, item| {
        acc += item;
        acc
    })(i)
}

fn count_whitespace(i: &str) -> IResult<&str, u64> {
    nom::branch::alt((count_spaces, count_tabs))(i)
}

fn count_tabs(i: &str) -> IResult<&str, u64> {
    map(is_a("\t"), |s: &str| (s.len() * 2) as u64)(i)
}

fn count_spaces(i: &str) -> IResult<&str, u64> {
    map(is_a(" "), |s: &str| (s.len() as u64))(i)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_counts_spaces() {
        assert_eq!(count_spaces("  something else"), Ok(("something else", 2)));
        assert_eq!(count_spaces(" something else"), Ok(("something else", 1)));
        assert_eq!(
            count_spaces("something else"),
            Err(nom::Err::Error(nom::error::Error::new(
                "something else",
                nom::error::ErrorKind::IsA
            )))
        );
        assert_eq!(
            count_spaces("\t something else"),
            Err(nom::Err::Error(nom::error::Error::new(
                "\t something else",
                nom::error::ErrorKind::IsA
            )))
        );
        assert_eq!(
            count_spaces(""),
            Err(nom::Err::Error(nom::error::Error::new(
                "",
                nom::error::ErrorKind::IsA
            )))
        );
    }

    #[test]
    fn it_counts_tabs() {
        assert_eq!(count_tabs("\t something else"), Ok((" something else", 2)));
        assert_eq!(count_tabs("\tsomething else"), Ok(("something else", 2)));
        // assert_eq!(count_tabs("something else"), Ok(("something else", 0)));
        // assert_eq!(count_tabs(""), Ok(("", 0)));
    }
}
