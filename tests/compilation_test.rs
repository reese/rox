extern crate rox;

use color_eyre::Result;
use std::path::PathBuf;

fn run_functions_compilation() -> Result<()> {
    rox::run_file(PathBuf::from("tests/fixtures/functions.rox"))
}

#[test]
fn it_compiles_functions() {
    let result = run_functions_compilation();
    assert!(result.is_ok());
}
