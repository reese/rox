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
use crate::roxc::{get_builtin_types, parse_file, Compiler, Stack};
use inkwell::context::Context;
use inkwell::passes::PassManager;
use std::collections::HashMap;
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
    let context = Context::create();
    compile_file(source, output, &context);
}

fn compile_file<T>(input_file: T, object_file_output: T, context: &Context)
where
    T: Into<PathBuf> + Sized + Clone,
{
    let module = context.create_module("rox");
    let declarations = match parse_file(input_file) {
        Ok(decl) => decl,
        Err(error) => {
            error.emit_error().unwrap();
            return;
        }
    };

    // TODO: Clean this shit up
    let mut environment_stack = Stack::new();
    environment_stack.push(HashMap::new());
    let (_, _, mut function_stack) = get_builtin_types();
    let function_pass_manager = PassManager::create(&module);

    let mut compiler = Compiler::new(
        &context,
        &module,
        &function_pass_manager,
        &mut environment_stack,
        &mut function_stack,
    );
    compiler.compile(declarations).unwrap();
    compiler.finish(object_file_output);
}
