use nom::{
    character::complete::{digit1, one_of},
    combinator::{map_res, opt, recognize},
    sequence::preceded,
    IResult,
};

pub fn date(i: &str) -> IResult<&str, String> {
    let (i, date_first) = digit_many(i)?;

    let (i, maybe_date_second) = opt(preceded(one_of("/-."), digit_many))(i)?;

    if let Some(date_second) = maybe_date_second {
        let (i, maybe_date_third) = opt(preceded(one_of("/-."), digit_many))(i)?;

        if let Some(date_third) = maybe_date_third {
            return Ok((
                i,
                format!(
                    "{0}-{1:>0width$}-{2:>0width$}",
                    date_first,
                    date_second,
                    date_third,
                    width = 2
                ),
            ));
        }

        return Ok((
            i,
            format!("{0}-{1:>0width$}", date_first, date_second, width = 2),
        ));
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

        assert_eq!(date("2021-01/21"), Ok(("", String::from("2021-01-21"))));
        assert_eq!(date("2021.01.21/"), Ok(("/", String::from("2021-01-21"))));
    }
}
