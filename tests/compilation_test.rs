extern crate rox;

fn run_functions_compilation() -> std::io::Result<()> {
    rox::run_file("tests/fixtures/functions.rox".as_ref())
}

#[test]
fn it_compiles_functions() {
    let result = run_functions_compilation();
    assert!(result.is_ok());
}
