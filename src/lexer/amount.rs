use nom::{
    branch::alt,
    branch::permutation,
    bytes::complete::{tag, take_while},
    character::complete::digit1,
    combinator::{map_res, opt, recognize, value},
    sequence::{preceded, tuple},
    IResult,
};

#[derive(Debug, Default, Eq, PartialEq)]
pub struct Amount {
    pub commodity: String,
    pub quantity: i64,
}

impl Amount {
    pub fn new() -> Self {
        Default::default()
    }
}

impl From<(f64, Option<&str>)> for Amount {
    fn from(lexed: (f64, Option<&str>)) -> Self {
        Amount {
            commodity: lexed.1.unwrap_or("").to_owned(),
            quantity: lexed.0 as i64 * 100 + (lexed.0.fract() * 100.) as i64,
        }
    }
}

pub fn amount_mapped(i: &str) -> IResult<&str, Amount> {
    map_res::<_, _, _, _, nom::error::Error<&str>, _, _>(amount, |a: (f64, Option<&str>)| {
        Ok(Amount::from(a))
    })(i)
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
    fn it_maps_amounts() {
        let expected = Amount {
            commodity: " USD".to_owned(),
            quantity: 4200,
        };

        assert_eq!(amount_mapped("42 USD"), Ok(("", expected)));

        let expected = Amount {
            commodity: "£".to_owned(),
            quantity: 4560,
        };

        assert_eq!(amount_mapped("£45.60"), Ok(("", expected)));

        let expected = Amount {
            commodity: "€ ".to_owned(),
            quantity: 3400,
        };

        assert_eq!(
            amount_mapped("34€ ; a comment"),
            Ok(("; a comment", expected))
        );
    }

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
