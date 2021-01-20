use super::dates::date;
use crate::journal::periodic_transaction::Period;
use crate::journal::periodic_transaction::PeriodInterval;
use nom::branch::{alt, permutation};
use nom::bytes::complete::tag;
use nom::character::complete::{multispace0, multispace1};
use nom::combinator::opt;
use nom::combinator::value;
use nom::sequence::delimited;
use nom::sequence::preceded;
use nom::IResult;

pub fn periodic_transaction_header(i: &str) -> IResult<&str, time::Date> {
    preceded(preceded(tag("~"), multispace1), date)(i)
}

fn period_expression(
    i: &str,
) -> IResult<
    &str,
    (
        Option<PeriodInterval>, // daily, weekly etc
        Option<time::Date>,     // start
        Option<time::Date>,     // end
    ),
> {
    permutation((
        preceded(opt(direction), opt(period)),
        preceded(opt(direction), opt(ws(date))),
        preceded(opt(direction), opt(ws(date))),
    ))(i)
}

fn direction(i: &str) -> IResult<&str, &str> {
    alt((ws(tag("in")), ws(tag("from")), ws(tag("to"))))(i)
}

fn period(i: &str) -> IResult<&str, PeriodInterval> {
    alt((
        value(PeriodInterval::Daily, ws(tag("daily"))),
        value(PeriodInterval::Weekly, ws(tag("weekly"))),
        value(PeriodInterval::Weekly, ws(tag("monthly"))),
        value(PeriodInterval::Weekly, ws(tag("quarterly"))),
        value(PeriodInterval::Weekly, ws(tag("yearly"))),
    ))(i)
}

fn this_period(i: &str) -> IResult<&str, PeriodInterval> {
    preceded(opt(ws(tag("this"))), period)(i)
}

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
    fn it_matches_weekly_periods() {
        let input = "weekly from 2009/1/1 to 2009/4/1";
        let start = time::Date::try_from_ymd(2009, 1, 1).unwrap();
        let end = time::Date::try_from_ymd(2009, 4, 1).unwrap();
        let expected = Ok(("", (Some(PeriodInterval::Weekly), Some(start), Some(end))));
        let output = period_expression(input);

        assert_eq!(output, expected);
    }
}
