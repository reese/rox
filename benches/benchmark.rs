#[macro_use]
extern crate rox;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rox::Value;

fn test_func(_limit: i64) -> Value {
    let result = rox! {
        let x = 0;
        while x < 100 {
            if x % 2 == 0 {
                print x;
            }
            x = x + 1;
        }
    };
    result.unwrap()
}

fn benchmark(c: &mut Criterion) {
    c.bench_function("prints even numbers", |b| {
        b.iter(|| test_func(black_box(100)))
    });
}

criterion_group!(benches, benchmark);
criterion_main!(benches);
