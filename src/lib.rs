#![deny(missing_docs)]
#![allow(clippy::needless_doctest_main)]
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/reese/rox/master/assets/rox-logo-transparent-bg.png"
)]
#![doc(
    html_favicon_url = "https://raw.githubusercontent.com/reese/rox/master/assets/favicon.ico"
)]
//! # Rox
//! `Rox` is high-level, statically-typed, compiled language that mixes
//! the familiarity of C-style syntax with the rigorous and expressive features
//! of functional languages.

#[macro_use]
extern crate lalrpop_util;

mod roxc;

pub use crate::roxc::Result;
use crate::roxc::{
    init_object_module, init_simplejit_module, parse_file, parse_string,
    Compiler, Declaration, RoxError,
};
use codespan_reporting::files::SimpleFile;
use core::mem;
use cranelift_module::FuncOrDataId;
use std::env::temp_dir;
use std::ffi::OsString;
use std::fs::File;
use std::io::Write;
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
pub fn build_file(
    path: PathBuf,
    output: PathBuf,
    no_link: bool,
) -> Result<isize> {
    build_source_string(output, no_link, path)
}

/// Builds the given source string and links to the output file
pub fn build_source_string(
    output: PathBuf,
    no_link: bool,
    source: PathBuf,
) -> Result<isize> {
    if no_link {
        compile_file(source, output)?;
        Ok(0)
    } else {
        compile_and_link(source, output)?;
        Ok(0)
    }
}

/// Run the given file with the SimpleJITBackend
pub fn run_file(path: PathBuf) -> Result<isize> {
    let declarations = parse_file(path)?;
    execute_declarations(declarations)
}

/// Executes the raw source string with the JIT compiler.
/// ```
/// use rox::execute_source_string;
/// let source = r#"
/// fn main() {
///     puts("Hello, world!");
/// }
/// "#;
/// execute_source_string(source);
/// ```
pub fn execute_source_string(source: &str) -> Result<isize> {
    let declarations = parse_string(source)?;
    execute_declarations(declarations)
}

fn execute_declarations(declarations: Vec<Declaration>) -> Result<isize> {
    let mut compiler = Compiler::new(init_simplejit_module());
    let compile_result = compiler.compile(declarations);
    if compile_result.is_err() {
        return Err(compile_result.err().unwrap());
    }
    let Compiler { mut module, .. } = compiler;
    let func_id = module.get_name("main").unwrap();
    // TODO: Handle `argc` and `argv` in `main`
    // TODO: Could this unsafe block be pushed into `SimpleJITBackend`?
    match func_id {
        FuncOrDataId::Func(func) => Ok(unsafe {
            mem::transmute::<_, fn() -> isize>(
                module.get_finalized_function(func),
            )()
        }),
        _ => unreachable!(),
    }
}

/// This function generates the native object file
/// and links it using the machine's default C compiler.
fn compile_and_link(input_file: PathBuf, output: PathBuf) -> Result<Output> {
    let temp = create_temp_object_file();
    match compile_file(input_file, temp.clone()) {
        Err(err) => Err(err),
        Ok(_) => link_file(temp, output),
    }
}

fn compile_file<T>(input_file: T, object_file_output: T) -> Result<()>
where
    T: Into<PathBuf> + Sized + Clone,
{
    let declarations = parse_file(input_file)?;
    let mut compiler = Compiler::new(init_object_module());
    match compiler.compile(declarations) {
        Err(err) => Err(err),
        Ok(_) => {
            let product = compiler.finish();
            let bytes = product.emit().unwrap();
            File::create(object_file_output.into())
                .unwrap()
                .write_all(&bytes)
                .unwrap();
            Ok(())
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
