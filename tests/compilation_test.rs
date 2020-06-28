extern crate rox;

#[test]
fn it_compiles_functions() {
    let result = rox::execute_source_string(
        r#"
    fn test(x: Number) -> Number {
        if x == 9 {
            return x + 1;
        }
        return 4 + 5;
    }

    fn main() {
        let ten = test(9);
        let nine = test(1234);
        let three = ten - 7;
        let x = "Hello, world!";
        puts(x);
    }"#,
    );
    assert!(result.is_ok());
}

#[test]
fn it_compiles_arrays() {
    let result = rox::execute_source_string(
        r#"
    fn test(x: Number) -> Array<Number> {
        let arr = array<Number>[2];
        arr[0] = x;
        arr[1] = x * 2;
        return arr;
    }

    fn main() {
        let ten = test(9);
        let nine = test(1234);
        let x = "Hello, world!";
        puts(4);
        puts(x);
    }
    "#,
    );
    assert!(result.is_ok());
}
