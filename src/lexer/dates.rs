use nom::{
    bytes::complete::is_a,
    character::complete::digit1,
    combinator::{map_res, opt, recognize},
    sequence::preceded,
    IResult,
};

fn date(i: &str) -> IResult<&str, String> {
    let (i, date_first) = digit_many(i)?;

    let (i, maybe_date_second) = opt(preceded(is_a("/-."), digit_many))(i)?;

    if let Some(date_second) = maybe_date_second {
        let (i, maybe_date_third) = opt(preceded(is_a("/-."), digit_many))(i)?;

        if let Some(date_third) = maybe_date_third {
            return Ok((i, format!("{}-{}-{}", date_first, date_second, date_third)));
        }

        return Ok((i, format!("{}-{}", date_first, date_second)));
    }

    Ok((i, format!("{}", date_first)))
}

fn digit_many(i: &str) -> IResult<&str, u64> {
    map_res(recognize(digit1), str::parse)(i)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_finds_the_date() {
        assert_eq!(date("20"), Ok(("", String::from("20"))));

        assert_eq!(date("2021"), Ok(("", String::from("2021"))));
        assert_eq!(date("2021/"), Ok(("/", String::from("2021"))));

        assert_eq!(date("2021-11"), Ok(("", String::from("2021-11"))));
        assert_eq!(date("2021-11/"), Ok(("/", String::from("2021-11"))));

        assert_eq!(date("2021-01/21"), Ok(("", String::from("2021-1-21"))));
        assert_eq!(date("2021.01.21/"), Ok(("/", String::from("2021-1-21"))));
    }
}
