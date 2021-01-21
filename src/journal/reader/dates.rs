use nom::{
    character::complete::{digit1, one_of},
    combinator::{map_res, opt, recognize},
    sequence::{preceded, tuple},
    IResult,
};
use std::convert::TryFrom;

pub fn date(i: &str) -> IResult<&str, time::Date> {
    map_res(
        tuple((
            digit_many,
            opt(preceded(one_of("/-."), digit_many)),
            opt(preceded(one_of("/-."), digit_many)),
        )),
        |parsed: (i32, Option<i32>, Option<i32>)| {
            let component_two = parsed.1.and_then(|c| u8::try_from(c).ok());
            let component_three = parsed.2.and_then(|c| u8::try_from(c).ok());
            match (component_two, component_three) {
                // Only one date component provided
                (None, None) => time::Date::try_from_ymd(parsed.0, 1, 1),

                // Only two date components provided
                (Some(a), None) => time::Date::try_from_ymd(parsed.0, a, 1),

                // All three date components provided
                (Some(a), Some(b)) => time::Date::try_from_ymd(parsed.0, a, b),

                (None, Some(_)) => panic!("how did this happen?"),
            }
        },
    )(i)
}

fn digit_many(i: &str) -> IResult<&str, i32> {
    map_res(recognize(digit1), str::parse)(i)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_finds_the_date() {
        assert_eq!(date("2021"), Ok(("", time::date!(2021 - 01 - 01))));
        assert_eq!(date("2021/"), Ok(("/", time::date!(2021 - 01 - 01))));

        assert_eq!(date("2021-11"), Ok(("", time::date!(2021 - 11 - 01))));
        assert_eq!(date("2021-11/"), Ok(("/", time::date!(2021 - 11 - 01))));

        assert_eq!(date("2021-01/21"), Ok(("", time::date!(2021 - 01 - 21))));
        assert_eq!(date("2021.01.21/"), Ok(("/", time::date!(2021 - 01 - 21))));
    }
}
