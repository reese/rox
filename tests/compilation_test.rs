extern crate rox;

use std::env::temp_dir;
use std::path::PathBuf;

fn run_functions_compilation() -> std::io::Result<()> {
    let mut temp_file = temp_dir();
    temp_file.push("test.o");
    rox::build_file(
        PathBuf::from("tests/fixtures/functions.rox"),
        temp_file,
        true,
    )
}

#[test]
fn it_compiles_functions() {
    let result = run_functions_compilation();
    assert!(result.is_ok());
}
