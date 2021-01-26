use criterion::{black_box, criterion_group, criterion_main, Criterion};
use reader::verify::{error_transformer, verify_fn, VerificationResult};

use nom::{bytes::complete::tag, sequence::preceded, IResult};

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
        error_transformer(abc_parser, xyz_parser)(input),
        Err(nom::Err::Failure(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Tag
        )))
    );

    group.bench_function("error transformer", |b| {
        b.iter(|| error_transformer(abc_parser, xyz_parser)(input))
    });

    assert_eq!(verify_fn(input, both), VerificationResult::Error);
    println!("hello");

    group.bench_function("verify", |b| b.iter(|| verify_fn(input, both)));
}

criterion_group!(benches, verification_benchmark);
criterion_main!(benches);
