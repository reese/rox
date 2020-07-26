mod syntax;
use crate::roxc::RoxError;
use crate::Result;
use std::fs::read_to_string;
use std::path::PathBuf;
pub use syntax::*;

lalrpop_mod!(#[allow(clippy::all)] pub rox_parser);

pub fn parse_string(source: &str) -> Result<Vec<Box<Statement>>> {
    let mut errors = Vec::new();
    let declarations = rox_parser::ProgramParser::new()
        .parse(&mut errors, source)
        .map_err(|e| {
            RoxError::from_parse_error(&e, PathBuf::from("./scratch/test.rox"))
        })?;
    match errors {
        empty_vec if empty_vec.is_empty() => Ok(declarations),
        error_vec => Err(RoxError::from_error_recoveries(
            error_vec,
            PathBuf::from("./scratch/test.rox"),
        )
        .unwrap()),
    }
}

pub(crate) fn parse_file(
    file: impl Into<PathBuf> + std::clone::Clone,
) -> Result<Vec<Box<Statement>>> {
    let source = read_to_string(file.into()).unwrap();
    parse_string(&source)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_parses_functions_with_no_arguments() {
        let source = r#"
        fn some_test() -> String {
            return "Hello, world!";
        }

        fn main() {
            puts(some_test());
        }
        "#;

        assert!(parse_string(source).is_ok());
    }

    #[test]
    fn it_parses_functions_with_arguments() {
        let source = r#"
        fn some_test(some_string: String) -> String {
            return some_string;
        }

        fn main() {
            puts(some_test("Golly this is an amazing test case!"));
        }
        "#;

        assert!(parse_string(source).is_ok());
    }

    #[test]
    fn it_parses_type_declaration_with_fields() {
        let source = r#"
        struct TestStruct = {
            first_field: String,
            is_real?: Bool,
            nested_struct: Option<TestStruct>
        }

        struct AnotherOne<T> = { generic_field: T, not_generic: String, }

        fn main() {
            let foo = TestStruct { first_field: "", is_real?: true, nested_struct: Option::None };
        }
        "#;

        assert!(parse_string(source).is_ok());
    }
}
