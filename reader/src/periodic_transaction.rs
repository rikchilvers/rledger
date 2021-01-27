use super::dates::{date, DateSource};
use journal::Period;
use journal::PeriodInterval;
use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::{multispace0, multispace1};
use nom::combinator::map_res;
use nom::combinator::opt;
use nom::combinator::value;
use nom::sequence::delimited;
use nom::sequence::preceded;
use nom::sequence::tuple;
use nom::IResult;

#[derive(Debug, Clone, PartialEq, Eq)]
enum Direction {
    From,
    To,
    In,
}

type PeriodicTransactionHeader = (
    Option<PeriodInterval>,
    Option<Direction>,
    Option<(time::Date, DateSource)>,
    Option<Direction>,
    Option<(time::Date, DateSource)>,
);

fn period_from_header(header: PeriodicTransactionHeader) -> Result<Period, nom::error::Error<()>> {
    let mut period = Period::default();
    period.interval = header.0;

    match (header.2, header.4) {
        // No dates
        (None, None) => {}

        // Two dates
        (Some((start_date, _)), Some((end_date, _))) => {
            period.start_date = Some(start_date);
            period.end_date = Some(end_date);
        }

        // One date
        (Some((date, source)), None) | (None, Some((date, source))) => match (header.1, header.3) {
            // No directions
            (None, None) => {
                period.start_date = Some(date);
                period.end_date = Some(final_date(date, source));
            }

            (None, Some(_)) => return Err(nom::error::Error::new((), nom::error::ErrorKind::ParseTo)),

            // Two directions
            (Some(_), Some(_)) => return Err(nom::error::Error::new((), nom::error::ErrorKind::ParseTo)),

            // Only in
            (Some(Direction::In), None) => {
                period.start_date = Some(date);
                period.end_date = Some(final_date(date, source));
            }

            // Only from
            (Some(Direction::From), None) => {
                period.start_date = Some(date);
            }

            // Only to
            // TODO: should this be inclusive or exclusive?
            (Some(Direction::To), None) => {
                period.end_date = Some(date);
            }
        },
    }

    return Ok(period);
}

fn final_date(date: time::Date, source: DateSource) -> time::Date {
    match source {
        DateSource::Year => {
            return time::Date::try_from_ymd(date.year() + 1, 1, 1).unwrap() - time::Duration::day();
        }
        DateSource::YearMonth => {
            return time::Date::try_from_ymd(date.year(), date.month() + 1, 1).unwrap() - time::Duration::day();
        }
        DateSource::YearMonthDay => date,
    }
}

pub fn periodic_transaction_header(i: &str) -> IResult<&str, Period> {
    preceded(preceded(tag("~"), multispace1), period_expression)(i)
}

fn period_expression(i: &str) -> IResult<&str, Period> {
    map_res(
        tuple((
            opt(period),
            opt(direction),
            opt(ws(date)),
            opt(direction),
            opt(ws(date)),
        )),
        |header| period_from_header(header),
    )(i)
}

fn direction(i: &str) -> IResult<&str, Direction> {
    alt((
        value(Direction::In, ws(tag("in"))),
        value(Direction::From, ws(tag("from"))),
        value(Direction::To, ws(tag("to"))),
    ))(i)
}

fn period(i: &str) -> IResult<&str, PeriodInterval> {
    alt((
        value(PeriodInterval::Daily, ws(tag("daily"))),
        value(PeriodInterval::Weekly, ws(tag("weekly"))),
        value(PeriodInterval::Monthly, ws(tag("monthly"))),
        value(PeriodInterval::Quarterly, ws(tag("quarterly"))),
        value(PeriodInterval::Yearly, ws(tag("yearly"))),
    ))(i)
}

/*
fn this_period(i: &str) -> IResult<&str, PeriodInterval> {
    preceded(opt(ws(tag("this"))), period)(i)
}
*/

// Cribbed from nom's recipes module
/// A combinator that takes a parser `inner` and produces a parser that also consumes both leading and
/// trailing whitespace, returning the output of `inner`.
fn ws<'a, F: 'a, O, E: nom::error::ParseError<&'a str>>(inner: F) -> impl FnMut(&'a str) -> IResult<&'a str, O, E>
where
    F: Fn(&'a str) -> IResult<&'a str, O, E>,
{
    delimited(multispace0, inner, multispace0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_matches_inferred_in_periods() {
        // Y
        let start_date = Some(time::Date::try_from_ymd(2009, 1, 1).unwrap());
        let end_date = Some(time::Date::try_from_ymd(2009, 12, 31).unwrap());
        let period = Period {
            start_date,
            end_date,
            interval: None,
            frequency: 0,
        };

        let expected = Ok(("", period));

        let input = "2009";
        let output = period_expression(input);
        assert_eq!(output, expected);

        // YM
        let start_date = Some(time::Date::try_from_ymd(2009, 1, 1).unwrap());
        let end_date = Some(time::Date::try_from_ymd(2009, 1, 31).unwrap());
        let period = Period {
            start_date,
            end_date,
            interval: None,
            frequency: 0,
        };

        let expected = Ok(("", period));

        let input = "2009-01";
        let output = period_expression(input);
        assert_eq!(output, expected);

        // YMD
        let start_date = Some(time::Date::try_from_ymd(2009, 1, 1).unwrap());
        let end_date = Some(time::Date::try_from_ymd(2009, 1, 1).unwrap());
        let period = Period {
            start_date,
            end_date,
            interval: None,
            frequency: 0,
        };

        let expected = Ok(("", period));

        let input = "2009-01-01";
        let output = period_expression(input);
        assert_eq!(output, expected);
    }

    #[test]
    fn it_matches_in_periods() {
        let start_date = Some(time::Date::try_from_ymd(2009, 1, 1).unwrap());
        let end_date = Some(time::Date::try_from_ymd(2009, 12, 31).unwrap());
        let period = Period {
            start_date,
            end_date,
            interval: None,
            frequency: 0,
        };

        let expected = Ok(("", period));

        let input = "in 2009";
        let output = period_expression(input);
        assert_eq!(output, expected);
    }

    #[test]
    fn it_matches_from_periods() {
        let start_date = Some(time::Date::try_from_ymd(2009, 1, 1).unwrap());
        let period = Period {
            start_date,
            end_date: None,
            interval: None,
            frequency: 0,
        };

        let expected = Ok(("", period));

        let input = "from 2009/1/1";
        let output = period_expression(input);
        assert_eq!(output, expected);

        let input = "from 2009/1";
        let output = period_expression(input);
        assert_eq!(output, expected);

        let input = "from 2009";
        let output = period_expression(input);
        assert_eq!(output, expected);
    }

    #[test]
    fn it_matches_to_periods() {
        let end_date = Some(time::Date::try_from_ymd(2009, 1, 1).unwrap());
        let period = Period {
            start_date: None,
            end_date,
            interval: None,
            frequency: 0,
        };

        let expected = Ok(("", period));

        let input = "to 2009";
        let output = period_expression(input);
        assert_eq!(output, expected);

        let input = "to 2009/1";
        let output = period_expression(input);
        assert_eq!(output, expected);

        let input = "to 2009";
        let output = period_expression(input);
        assert_eq!(output, expected);
    }

    #[test]
    fn it_matches_interval_from_to_periods() {
        let input = "weekly from 2009/1/1 to 2009/4/1";
        let start_date = Some(time::Date::try_from_ymd(2009, 1, 1).unwrap());
        let end_date = Some(time::Date::try_from_ymd(2009, 4, 1).unwrap());
        let period = Period {
            start_date,
            end_date,
            interval: Some(PeriodInterval::Weekly),
            frequency: 0,
        };
        let expected = Ok(("", period));
        let output = period_expression(input);

        assert_eq!(output, expected);
    }

    #[test]
    fn it_matches_interval_from_periods() {
        let input = "yearly from 2009/1/1";
        let start_date = Some(time::Date::try_from_ymd(2009, 1, 1).unwrap());
        let period = Period {
            start_date,
            end_date: None,
            interval: Some(PeriodInterval::Yearly),
            frequency: 0,
        };
        let expected = Ok(("", period));
        let output = period_expression(input);

        assert_eq!(output, expected);
    }

    #[test]
    fn it_matches_interval_to_periods() {
        let input = "monthly to 2009";
        let end_date = Some(time::Date::try_from_ymd(2009, 1, 1).unwrap());
        let period = Period {
            start_date: None,
            end_date,
            interval: Some(PeriodInterval::Monthly),
            frequency: 0,
        };
        let expected = Ok(("", period));
        let output = period_expression(input);

        assert_eq!(output, expected);
    }
}
