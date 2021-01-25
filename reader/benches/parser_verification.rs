use criterion::{black_box, criterion_group, criterion_main, Criterion};
use journal::reader::transaction_header::*;

pub fn criterion_benchmark(c: &mut Criterion) {
    // c.bench_function("error transformer", |b| b.iter(|| error_transformer())));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
