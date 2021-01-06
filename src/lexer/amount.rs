use nom::{
    branch::alt,
    branch::permutation,
    bytes::complete::{tag, take_while},
    character::complete::digit1,
    combinator::{map_res, opt, recognize, value},
    sequence::{preceded, tuple},
    IResult,
};

#[derive(Default)]
pub struct Amount {
    commodity: String,
    quantity: i64,
}

pub fn amount(i: &str) -> IResult<&str, (f64, Option<&str>)> {
    permutation((mapped_quantity, opt(commodity)))(i)
}

fn commodity(i: &str) -> IResult<&str, &str> {
    // TODO refactor to not use take_while
    take_while(is_not_number_or_comment)(i)
    // preceded(space0, take_while(is_not_number_or_comment))(i)
    // terminated(take_while(is_not_number_or_comment), space0)(i)
    // one_of("0123456789;#-")(i)
}

fn is_not_number_or_comment(chr: char) -> bool {
    !nom::character::is_digit(chr as u8) && chr != ';' && chr != '#' && chr != '+' && chr != '-'
}

fn mapped_quantity(i: &str) -> IResult<&str, f64> {
    map_res(recognize(quantity), |q: &str| q.parse::<f64>())(i)
}

fn quantity(i: &str) -> IResult<&str, (Option<i8>, &str, Option<&str>)> {
    tuple((opt(sign), digit1, opt(preceded(tag("."), digit1))))(i)
}

fn sign(i: &str) -> IResult<&str, i8> {
    alt((value(-1, tag("-")), value(1, tag("+"))))(i)
}

#[cfg(test)]
mod tests {
    use super::*;
    use nom::error::ErrorKind;

    #[test]
    fn it_matches_amounts_with_no_commodity() {
        assert_eq!(amount("42.81"), Ok(("", (42.81, Some("")))));
    }

    #[test]
    fn it_matches_commodities() {
        assert_eq!(commodity("£"), Ok(("", "£")));
        assert_eq!(commodity("£55"), Ok(("55", "£")));
        assert_eq!(commodity("£ 42"), Ok(("42", "£ ")));
        assert_eq!(commodity("£ ; a comment"), Ok(("; a comment", "£ ")));
    }

    #[test]
    fn it_matches_commodities_after_a_number() {
        assert_eq!(amount("42USD"), Ok(("", (42.0, Some("USD")))));
        assert_eq!(amount("81 USD"), Ok(("", (81.0, Some(" USD")))));
        assert_eq!(amount("55.34USD"), Ok(("", (55.34, Some("USD")))));
        assert_eq!(
            amount("84 USD ; a comment"),
            Ok(("; a comment", (84.0, Some(" USD "))))
        );
    }

    #[test]
    fn it_matches_commodities_before_a_number() {
        assert_eq!(amount("£42"), Ok(("", (42.0, Some("£")))));
        assert_eq!(amount("£42.01"), Ok(("", (42.01, Some("£")))));
        assert_eq!(
            amount("£ 42 ; a comment"),
            Ok((" ; a comment", (42.0, Some("£ "))))
        );
    }

    #[test]
    fn it_matches_quantities() {
        assert_eq!(mapped_quantity("42"), Ok(("", 42_f64)));
        assert_eq!(mapped_quantity("+42"), Ok(("", 42_f64)));
        assert_eq!(mapped_quantity("+42.0"), Ok(("", 42.0)));
        assert_eq!(mapped_quantity("-42.01"), Ok(("", -42.01)));
    }

    #[test]
    fn it_matches_signs() {
        assert_eq!(sign("+42"), Ok(("42", 1)));
        assert_eq!(sign("-42"), Ok(("42", -1)));
    }

    #[test]
    fn it_fails_with_no_sign() {
        assert_eq!(
            sign("42"),
            Err(nom::Err::Error(nom::error::Error::new(
                "42",
                ErrorKind::Tag
            )))
        );
    }
}
