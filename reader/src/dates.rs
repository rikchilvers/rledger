use nom::{
    character::complete::{digit1, one_of},
    combinator::{consumed, map_res, opt, recognize},
    sequence::{preceded, tuple},
    IResult,
};
use std::convert::TryFrom;

#[derive(Debug, PartialEq, Eq)]
pub enum DateSource {
    Year,
    YearMonth,
    YearMonthDay,
}

pub fn date(i: &str) -> IResult<&str, (time::Date, DateSource)> {
    map_res(
        tuple((
            digit_many,
            opt(preceded(one_of("/-."), digit_many)),
            opt(preceded(one_of("/-."), digit_many)),
        )),
        |parsed: ((&str, i32), Option<(&str, i32)>, Option<(&str, i32)>)| {
            let part_one = parsed.0 .1;
            let part_two = parsed.1.and_then(|c| u8::try_from(c.1).ok());
            let part_three = parsed.2.and_then(|c| u8::try_from(c.1).ok());
            match (part_two, part_three) {
                // Only one date component provided
                (None, None) => match time::Date::try_from_ymd(part_one, 1, 1) {
                    Err(e) => Err(e),
                    Ok(date) => Ok((date, DateSource::Year)),
                },

                // Only two date components provided
                (Some(part_two), None) => match time::Date::try_from_ymd(part_one, part_two, 1) {
                    Err(e) => Err(e),
                    Ok(date) => Ok((date, DateSource::YearMonth)),
                },

                // All three date components provided
                (Some(part_two), Some(part_three)) => match time::Date::try_from_ymd(part_one, part_two, part_three) {
                    Err(e) => Err(e),
                    Ok(date) => Ok((date, DateSource::YearMonthDay)),
                },

                (None, Some(_)) => panic!("how did this happen?"),
            }
        },
    )(i)
}

fn digit_many(i: &str) -> IResult<&str, (&str, i32)> {
    consumed(map_res(recognize(digit1), str::parse))(i)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_finds_y() {
        assert_eq!(date("2021"), Ok(("", (time::date!(2021 - 01 - 01), DateSource::Year))));
        assert_eq!(
            date("2021/"),
            Ok(("/", (time::date!(2021 - 01 - 01), DateSource::Year)))
        );
    }

    #[test]
    fn it_finds_ym() {
        assert_eq!(
            date("2021-11"),
            Ok(("", (time::date!(2021 - 11 - 01), DateSource::YearMonth)))
        );
        assert_eq!(
            date("2021-11/"),
            Ok(("/", (time::date!(2021 - 11 - 01), DateSource::YearMonth)))
        );
    }

    #[test]
    fn it_finds_ymd() {
        assert_eq!(
            date("2021-01/21"),
            Ok(("", (time::date!(2021 - 01 - 21), DateSource::YearMonthDay)))
        );
        assert_eq!(
            date("2021.01.21/"),
            Ok(("/", (time::date!(2021 - 01 - 21), DateSource::YearMonthDay)))
        );
    }
}
