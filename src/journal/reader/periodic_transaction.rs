use super::dates::date;
use super::ReaderError;
use crate::journal::periodic_transaction::Period;
use crate::journal::periodic_transaction::PeriodInterval;
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

use std::convert::TryFrom;

#[derive(Debug, Clone, PartialEq, Eq)]
enum Direction {
    From,
    To,
    In,
}

type PHeader = (
    Option<PeriodInterval>,
    Option<Direction>,
    Option<time::Date>,
    Option<Direction>,
    Option<time::Date>,
);

impl TryFrom<PHeader> for Period {
    type Error = ReaderError;

    fn try_from(header: PHeader) -> Result<Self, ReaderError> {
        let mut period = Period::default();
        period.interval = header.0;

        match (header.2, header.4) {
            // No dates
            (None, None) => {}

            // Two dates
            (Some(_), Some(_)) => {
                period.start_date = header.2;
                period.end_date = header.4;
            }

            // First date
            (Some(date), None) => match (header.1, header.3) {
                // No directions
                (None, None) => {
                    panic!("should interpret one date + no direction as 'in'");
                }

                // Two directions
                (Some(_), Some(_)) => {
                    panic!("can't handle two directions with only one date")
                }

                // Only in
                (Some(Direction::In), None) => {
                    period.start_date = Some(date);
                    // period.end_date = date.with_time
                }

                // Only from
                (Some(Direction::From), None) => {
                    period.start_date = Some(date);
                }

                // Only to
                (Some(Direction::To), None) => {
                    period.end_date = Some(date);
                }

                _ => todo!(),
            },

            // Second date
            (None, Some(date)) => match (header.1, header.3) {
                // No directions
                (None, None) => {
                    panic!("should interpret one date + no direction as 'in'");
                }

                // Two directions
                (Some(_), Some(_)) => {
                    panic!("can't handle two directions with only one date")
                }

                // Only in
                (Some(Direction::In), None) | (None, Some(Direction::In)) => {
                    period.start_date = Some(date);
                }

                // Only from
                (Some(Direction::From), None) => {
                    period.start_date = Some(date);
                }

                // Only to
                (Some(Direction::To), None) | (None, Some(Direction::To)) => {
                    period.end_date = Some(date);
                }

                _ => todo!(),
            },
        }

        return Ok(period);
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
        |header| Period::try_from(header),
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
