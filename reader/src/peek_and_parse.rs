use nom::{
    combinator::{cut, peek},
    sequence::preceded,
    IResult,
};

use super::error::LineType;
use super::ReaderError;

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

fn parse_line<I: Clone, O, E: nom::error::ParseError<I>>(
    input: I,
    line_type: LineType,
    line_number: u64,
    // mut parser: impl FnMut(I) -> IResult<I, O, E>,
    mut parser: impl FnMut(I) -> IResult<I, O, E>,
) -> Result<Option<O>, ReaderError> {
    match parser(input) {
        Ok((_, output)) => return Ok(Some(output)),
        Err(e) => match e {
            nom::Err::Error(_) => Ok(None),
            nom::Err::Failure(_) => Err(ReaderError::Parse(line_type, line_number)),
            nom::Err::Incomplete(_) => unimplemented!(),
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
