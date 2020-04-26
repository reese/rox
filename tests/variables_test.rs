#![feature(test)]
extern crate rox;
extern crate test;

use std::io::Error;

fn run_variable_compilation() -> std::io::Result<()> {
    rox::run_file("tests/fixtures/variables.rox".as_ref())
}

#[test]
fn it_compiles_variables() {
    let result = run_variable_compilation();
    assert!(result.is_ok());
}

#[bench]
fn benchmark_variable_compilation(bencher: &mut test::Bencher) {
    let n = test::black_box(100);

    bencher.iter(|| {
        (0..n).for_each(|_| {
            run_variable_compilation().unwrap();
        })
    });
}
