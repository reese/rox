mod syntax;

use crate::roxc::RoxError;
use crate::Result;
use std::fs::read_to_string;
use std::path::PathBuf;
pub use syntax::*;

lalrpop_mod!(#[allow(clippy::all)] pub rox_parser);

pub fn parse_string(
    source: &str,
    path: &PathBuf,
) -> Result<Vec<Box<Statement>>> {
    let mut errors = Vec::new();
    let declarations = rox_parser::ProgramParser::new()
        .parse(&mut errors, source)
        .map_err(|e| RoxError::from_parse_error(&e, path.clone()))?;
    match errors {
        empty_vec if empty_vec.is_empty() => Ok(declarations),
        error_vec => Err(RoxError::from_error_recoveries(error_vec, path)),
    }
}

pub(crate) fn parse_file(
    file: impl Into<PathBuf> + Clone,
) -> Result<Vec<Box<Statement>>> {
    let path_buf: &PathBuf = &file.into();
    let source = read_to_string(path_buf).unwrap();
    parse_string(&source, path_buf)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_parses_functions_with_no_arguments() {
        let source = r#"
        fn some_test() -> String do
            return "Hello, world!";
        end

        fn main() do
            puts(some_test());
        end
        "#;

        assert!(parse_string(source, &PathBuf::new()).is_ok());
    }

    #[test]
    fn it_parses_functions_with_arguments() {
        let source = r#"
        fn some_test(some_string: String) -> String do
            return some_string;
        end

        fn main() do
            puts(some_test("Golly this is an amazing test case!"));
        end
        "#;

        assert!(parse_string(source, &PathBuf::new()).is_ok());
    }

    #[test]
    fn it_parses_type_declaration_with_fields() {
        let source = r#"
        struct TestStruct {
            first_field: String,
            is_real?: Bool,
        }

        struct AnotherOne<T> { generic_field: T, not_generic: String, }

        fn main() do
            let foo = TestStruct { first_field: "", is_real?: true };
        end
        "#;

        assert!(parse_string(source, &PathBuf::new()).is_ok());
    }
}
