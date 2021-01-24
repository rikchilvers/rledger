use nom::{
    branch::alt, bytes::complete::take_till, character::complete::multispace0, combinator::rest, sequence::preceded,
    IResult,
};

pub fn payee(i: &str) -> IResult<&str, &str> {
    preceded(multispace0, alt((take_till(is_comment), rest)))(i)
}

fn is_comment(chr: char) -> bool {
    chr == ';' || chr == '#'
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_finds_payees() {
        assert_eq!(payee(""), Ok(("", "")));
        assert_eq!(payee("a really long payee"), Ok(("", "a really long payee")));
    }

    #[test]
    fn it_finds_payees_before_comments() {
        assert_eq!(payee("the payee ; a comment"), Ok(("; a comment", "the payee ")));
        assert_eq!(payee("the payee; a comment"), Ok(("; a comment", "the payee")));
    }
}
