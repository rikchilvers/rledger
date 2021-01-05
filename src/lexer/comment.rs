use nom::{
    character::complete::{one_of, space0},
    combinator::rest,
    sequence::preceded,
    IResult,
};

fn comment(i: &str) -> IResult<&str, &str> {
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
