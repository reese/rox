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
use crate::roxc::{
    get_builtin_types, parse_file, parse_string, Compiler, RoxError, Stack,
    Statement,
};
use codespan_reporting::files::SimpleFile;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::passes::PassManager;
use std::collections::HashMap;
use std::env::temp_dir;
use std::ffi::OsString;
use std::path::PathBuf;
use std::process::{Command, Output};

/// `run_file` reads the contents of the given path
/// and runs them through the roxc.
///
/// # Errors
/// Currently, the Rox roxc exits with a
/// `1` error code if it fails to compile. In the future,
/// it would likely be a good idea to refactor these
/// to have POSIX compliant error codes, or at least
/// some consistent error code system.
pub fn build_file(path: PathBuf, output: PathBuf) -> Result<isize> {
    build_source_string(output, path)
}

/// Builds the given source string and links to the output file
pub fn build_source_string(output: PathBuf, source: PathBuf) -> Result<isize> {
    let context = Context::create();
    let module = context.create_module("rox");
    compile_file(source, output, &context, &module)?;
    Ok(0)
}

/// Executes the raw source string with the JIT compiler.
/// ```
/// use rox::execute_source_string;
/// let source = r#"
/// extern fn puts(String);
/// fn main() do
///     puts("Hello, world!");
/// end
/// "#;
/// execute_source_string(source).unwrap();
/// ```
pub fn execute_source_string(source: &str) -> Result<()> {
    let declarations = parse_string(source)?;
    let context = Context::create();
    let module = context.create_module("rox");
    execute_declarations(declarations, &context, &module)?;
    Ok(())
}

fn execute_declarations<'c>(
    declarations: Vec<Box<Statement>>,
    context: &'c Context,
    module: &'c Module<'c>,
) -> Result<()> {
    let mut environment_stack = Stack::new();
    environment_stack.push(HashMap::new());
    let (_, _, mut function_stack) = get_builtin_types();
    let function_pass_manager = PassManager::create(module);

    let mut compiler = Compiler::new(
        context,
        module,
        &function_pass_manager,
        &mut environment_stack,
        &mut function_stack,
    );
    compiler.compile(declarations)
}

fn compile_file<'c, T>(
    input_file: T,
    object_file_output: T,
    context: &'c Context,
    module: &'c Module<'c>,
) -> Result<()>
where
    T: Into<PathBuf> + Sized + Clone,
{
    let declarations = parse_file(input_file)?;

    // TODO: Clean this shit up
    let mut environment_stack = Stack::new();
    environment_stack.push(HashMap::new());
    let (_, _, mut function_stack) = get_builtin_types();
    let function_pass_manager = PassManager::create(module);

    let mut compiler = Compiler::new(
        &context,
        &module,
        &function_pass_manager,
        &mut environment_stack,
        &mut function_stack,
    );
    match compiler.compile(declarations) {
        Err(err) => Err(err),
        Ok(_) => {
            let product = compiler.finish(object_file_output);
            match product {
                false => Err(RoxError::with_file_placeholder(
                    "Something bad happened",
                )),
                true => Ok(()),
            }
        }
    }
}

fn link_file<T: Into<OsString> + Clone>(
    file_to_link: T,
    output_file: T,
) -> Result<Output> {
    Command::new("cc")
        .arg(file_to_link.clone().into())
        .arg("-o")
        .arg(output_file.into())
        .output()
        .map_err(|err| RoxError {
            file: SimpleFile::new(
                file_to_link.clone().into().into_string().unwrap(),
                file_to_link.into().into_string().unwrap(),
            ),
            message: Some(String::from(
                "Failed to link file due to unexpected error.",
            )),
            labels: vec![],
            notes: vec![err.to_string()],
        })
}

fn create_temp_object_file() -> PathBuf {
    let mut dir = temp_dir();
    dir.push("__rox_temp_build_file.o");
    dir
}
