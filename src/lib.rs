#![deny(missing_docs)]
#![allow(clippy::needless_doctest_main)]
#![allow(clippy::vec_box)]
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/reese/rox/master/assets/rox-logo-transparent-bg.png"
)]
#![doc(
    html_favicon_url = "https://raw.githubusercontent.com/reese/rox/master/assets/favicon.ico"
)]
//! # Rox

#[macro_use]
extern crate lalrpop_util;

mod roxc;

pub use crate::roxc::Result;
use crate::roxc::{parse_file, Compiler};
use std::path::PathBuf;

/// `run_file` reads the contents of the given path
/// and runs them through the roxc.
///
/// # Errors
/// Currently, the Rox roxc exits with a
/// `1` error code if it fails to compile. In the future,
/// it would likely be a good idea to refactor these
/// to have POSIX compliant error codes, or at least
/// some consistent error code system.
pub fn build_file(path: PathBuf, output: PathBuf) {
    build_source_string(output, path);
}

/// Builds the given source string and links to the output file
pub fn build_source_string(output: PathBuf, source: PathBuf) {
    compile_file(source, output);
}

fn compile_file<T>(input_file: T, object_file_output: T)
where
    T: Into<PathBuf> + Sized + Clone,
{
    let declarations_result = parse_file(input_file);
    match declarations_result {
        Err(e) => e.emit_error().unwrap(),
        Ok(declarations) => {
            // TODO: Clean this shit up
            // let (_, _, function_stack) = get_builtin_types();

            let mut compiler = Compiler::new();
            compiler.compile(declarations).unwrap();
            compiler.finish(object_file_output);
        }
    }
}
