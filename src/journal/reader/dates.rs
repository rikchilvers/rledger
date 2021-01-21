use nom::{
    character::complete::{digit1, one_of},
    combinator::{map_res, opt, recognize},
    sequence::preceded,
    IResult,
};

pub fn date(i: &str) -> IResult<&str, time::Date> {
    let (i, year) = digit_many(i)?;

    let (i, maybe_month) = opt(preceded(one_of("/-."), digit_many))(i)?;
    if let Some(month) = maybe_month {
        let (i, maybe_day) = opt(preceded(one_of("/-."), digit_many))(i)?;
        if let Some(day) = maybe_day {
            // TODO remove this unwrap
            let date = time::Date::try_from_ymd(year, month as u8, day as u8).unwrap();
            return Ok((i, date));
        }

        // TODO remove this unwrap
        let date = time::Date::try_from_ymd(year, month as u8, 01).unwrap();
        return Ok((i, date));
    }

    // TODO remove this unwrap
    let date = time::Date::try_from_ymd(year, 01, 01).unwrap();
    Ok((i, date))
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
