use nom::{character::complete::*, combinator::map, multi::*, IResult};

fn digit4(i: &str) -> IResult<&str, Vec<&str>> {
    many_m_n(4, 4, digit1)(i)
}

fn digit_many(i: &str) -> IResult<&str, Result<u64, nom::error::Error<&str>>> {
    map(
        nom::bytes::complete::take_while(is_digit_char),
        |s: &str| {
            s.parse::<u64>()
                .map_err(|_| nom::error::Error::new(i, nom::error::ErrorKind::ParseTo))
        },
    )(i)
}

fn is_digit_char(c: char) -> bool {
    c.is_ascii() && nom::character::is_digit(c as u8)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_finds_four_digits() {
        assert_eq!(digit_many("2021"), Ok(("", Ok(2021))));
        assert_eq!(digit_many("2021/"), Ok(("/", Ok(2021))));
        assert_eq!(
            digit_many("/2021/"),
            Err(nom::error::Error::new(
                "/2021/",
                nom::error::ErrorKind::ParseTo
            ))
        );
    }
}
