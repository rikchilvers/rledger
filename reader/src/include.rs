use nom::{bytes::complete::tag, character::complete::space1, combinator::rest, sequence::preceded, IResult};

pub fn include(i: &str) -> IResult<&str, &str> {
    preceded(preceded(tag("include"), space1), rest)(i)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_finds_include_statements() {
        let input = "include some_other_path.journal";
        assert_eq!(include(input), Ok(("", "some_other_path.journal")));
    }

    #[test]
    fn it_errors_on_no_space_include_statements() {
        let input = "includesome_other_path.journal";
        assert_eq!(
            include(input),
            Err(nom::Err::Error(nom::error::Error::new(
                "some_other_path.journal",
                nom::error::ErrorKind::Space
            )))
        );
    }
}
