use nom::{
    character::complete::{one_of, space0},
    combinator::{rest, verify},
    sequence::preceded,
    IResult,
};

use super::whitespace::*;

pub fn comment(i: &str) -> IResult<&str, &str> {
    preceded(one_of(";#"), preceded(space0, rest))(i)
}

/// Ensures at least `min` spaces
pub fn comment_min(min: u8, i: &str) -> IResult<&str, &str> {
    preceded(verify(whitespace2, |count| *count >= min), comment)(i)
}

/// Ensures min < spaces < max (unless max is 0)
pub fn comment_min_max(min: u8, max: u8, i: &str) -> IResult<&str, &str> {
    preceded(
        verify(whitespace2, |count| {
            if max > 0 {
                return *count >= min && *count <= max;
            }
            return *count >= min;
        }),
        comment,
    )(i)
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
