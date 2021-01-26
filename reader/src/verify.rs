use nom::{
    combinator::{cut, peek},
    sequence::preceded,
    IResult,
};

/// Peeks at the `marker` and if it succeeds, tries the `parser` (which it will hard fail if
/// unsuccessful)
pub fn error_transformer<I: Clone, O1, O2, E: nom::error::ParseError<I>, F, G>(
    marker: F,
    parser: G,
) -> impl FnMut(I) -> IResult<I, O2, E>
where
    F: nom::Parser<I, O1, E>,
    G: nom::Parser<I, O2, E>,
{
    preceded(peek(marker), cut(parser))
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum VerificationResult<I, O> {
    Ok((I, O)), // the parser worked
    Error,      // the parser failed
    NoMatch,    // the parser did not match the input
}

/// Attemps the `parser` and checks its length against the input's length to see if it hard failed
pub fn verify_fn<'a, O>(
    input: &'a str,
    mut parser: impl FnMut(&'a str) -> IResult<&'a str, O>,
) -> VerificationResult<&'a str, O> {
    match parser(input) {
        Ok((remaining, output)) => return VerificationResult::Ok((remaining, output)),
        Err(e) => match e {
            nom::Err::Error(e) => {
                if e.input.len() != input.len() {
                    return VerificationResult::Error;
                }
                return VerificationResult::NoMatch;
            }
            _ => unimplemented!(),
        },
    }
}

pub fn verify<'a, O>(input: &str, parse_result: IResult<&'a str, O>) -> VerificationResult<&'a str, O> {
    match parse_result {
        Ok((remaining, output)) => return VerificationResult::Ok((remaining, output)),
        Err(e) => match e {
            nom::Err::Error(e) => {
                if e.input.len() != input.len() {
                    return VerificationResult::Error;
                }
                return VerificationResult::NoMatch;
            }
            _ => unimplemented!(),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_th() {
        // let input = "0abc";
        // assert_eq!(verify_fn("0abc", t_h), VerificationResult::Ok(("", "abc")));
        // assert_eq!(verify_fn("0abc0", t_h), VerificationResult::Ok(("0", "abc")));
        // assert_eq!(verify_fn("0abd", t_h), VerificationResult::Error);
        // assert_eq!(verify_fn("abc", t_h), VerificationResult::NoMatch);
    }
}
