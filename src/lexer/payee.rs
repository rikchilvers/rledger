use nom::{
    branch::alt,
    bytes::complete::take_till,
    combinator::{map_res, rest},
    sequence::preceded,
    IResult,
};

pub fn payee(i: &str) -> IResult<&str, &str> {
    preceded(take_till(is_not_space), alt((take_till(is_comment), rest)))(i)
}

pub fn is_not_space(chr: char) -> bool {
    !nom::character::is_space(chr as u8)
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
        assert_eq!(
            payee("a really long payee"),
            Ok(("", "a really long payee"))
        );
    }

    #[test]
    fn it_finds_payees_before_comments() {
        assert_eq!(
            payee("the payee ; a comment"),
            Ok(("; a comment", "the payee "))
        );
        assert_eq!(
            payee("the payee; a comment"),
            Ok(("; a comment", "the payee"))
        );
    }
}
