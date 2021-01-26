use criterion::{criterion_group, criterion_main, Criterion};
use reader::peek_and_parse::peek_and_parse;

use nom::{bytes::complete::tag, sequence::preceded, IResult};

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

pub fn verification_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("verification");

    // We want to match on the start of this and then fail
    let input = "abc a long string to make this slighly less trivial";

    fn abc_parser(i: &str) -> IResult<&str, &str> {
        tag("abc")(i)
    }

    fn xyz_parser(i: &str) -> IResult<&str, &str> {
        tag("xyz")(i)
    }

    fn both(i: &str) -> IResult<&str, &str> {
        preceded(abc_parser, xyz_parser)(i)
    }

    assert_eq!(
        peek_and_parse(abc_parser, xyz_parser)(input),
        Err(nom::Err::Failure(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Tag
        )))
    );

    group.bench_function("error transformer", |b| {
        b.iter(|| peek_and_parse(abc_parser, xyz_parser)(input))
    });

    assert_eq!(verify_fn(input, both), VerificationResult::Error);

    group.bench_function("verify", |b| b.iter(|| verify_fn(input, both)));
}

criterion_group!(benches, verification_benchmark);
criterion_main!(benches);
