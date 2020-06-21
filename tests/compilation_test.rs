extern crate rox;

fn run_functions_compilation() -> rox::Result<()> {
    rox::execute_source_string(
        r#"
    fn test (x: Number) -> Number {
        if x == 9 {
            return x + 1;
        }
        return 4 + 5;
    }

    fn main () {
        let ten = test(9);
        let nine = test(1234);
        let three = ten - 7;
        let x = "Hello, world!";
        puts(x);
    }"#,
    )
}

#[test]
fn it_compiles_functions() {
    let result = run_functions_compilation();
    assert!(result.is_ok());
}
