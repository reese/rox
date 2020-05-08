#![feature(test)]
extern crate rox;
extern crate test;

fn run_blocks_compilation() -> std::io::Result<()> {
    rox::run_file("tests/fixtures/blocks.rox".as_ref())
}

fn run_functions_compilation() -> std::io::Result<()> {
    rox::run_file("tests/fixtures/functions.rox".as_ref())
}

#[test]
fn it_compiles_blocks() {
    let result = run_blocks_compilation();
    assert!(result.is_ok());
}

#[test]
fn it_compiles_functions() {
    let result = run_functions_compilation();
    assert!(result.is_ok());
}
