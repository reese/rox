extern crate rox;

#[test]
fn it_compiles_functions() {
    let result = rox::execute_source_string(
        r#"
    extern fn puts(String);

    fn test(x: Number) -> Number do
        if x == 9 do
            return x + 1;
        end
        return 4 + 5;
    end

    fn main() do
        let ten = test(9);
        let nine = test(1234);
        let three = ten - 7;
        let x = "Hello, world!";
        puts(x);
    end
    "#,
    );
    dbg!(result.clone());
    assert!(result.is_ok());
}

#[test]
fn it_compiles_arrays() {
    let result = rox::execute_source_string(
        r#"
    fn test(x: Number) -> Array<Number> do
        let arr = [x, x * 2];
        return arr;
    end

    fn main() do
        let ten = test(9);
        let nine = test(1234);
        let x = "Hello, world!";
    end

    "#,
    );
    assert!(result.is_ok());
}
