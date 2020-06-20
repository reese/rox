use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark(c: &mut Criterion) {
    c.bench_function("", |_| {});
}

criterion_group!(benches, benchmark);
criterion_main!(benches);
