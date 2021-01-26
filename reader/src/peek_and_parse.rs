use nom::{
    combinator::{cut, peek},
    sequence::preceded,
    IResult,
};

/// Peeks at the `marker` and if it succeeds, tries the `parser` (which it will hard fail if
/// unsuccessful)
pub fn peek_and_parse<I: Clone, O1, O2, E: nom::error::ParseError<I>, F, G>(
    marker: F,
    parser: G,
) -> impl FnMut(I) -> IResult<I, O2, E>
where
    F: nom::Parser<I, O1, E>,
    G: nom::Parser<I, O2, E>,
{
    preceded(peek(marker), cut(parser))
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
