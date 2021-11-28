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
use std::{collections::HashMap, process::Command};
use std::{path::PathBuf, process::ExitStatus};
use tempfile::NamedTempFile;

/// `build_file` reads the contents at `path` and outputs
/// an executable at the given `output` path
pub fn build_file(path: PathBuf, output: PathBuf) -> ExitStatus {
    let context = Context::create();

    // N.B. NamedTempFile instances won't be cleaned up if the destructor isn't run,
    // but we need a file path, so `tempfile()` won't work here
    let temp_bitcode_file = NamedTempFile::new().unwrap();
    let temp_object_file = NamedTempFile::new().unwrap();
    let bitcode_file_path = temp_bitcode_file.into_temp_path();
    let object_file_path = temp_object_file.into_temp_path();

    compile_file(path, bitcode_file_path.as_os_str().into(), &context);

    Command::new("llc")
        .args(&[
            bitcode_file_path.as_os_str().to_str().unwrap(),
            "-filetype=obj",
            "-o",
            object_file_path.as_os_str().to_str().unwrap(),
        ])
        .output()
        .expect("Failed to execute `llc`");
    Command::new("cc")
        .args(&[
            object_file_path.to_str().unwrap(),
            "-o",
            output.to_str().unwrap(),
        ])
        .output()
        .expect("Failed to link with `cc`")
        .status
}

/// `run_file` reads the contents of the given path
/// and runs them through the roxc.
///
/// # Errors
/// Currently, the Rox roxc exits with a
/// `1` error code if it fails to compile. In the future,
/// it would likely be a good idea to refactor these
/// to have POSIX compliant error codes, or at least
/// some consistent error code system.
pub fn run_file(path: PathBuf) -> ExitStatus {
    let context = Context::create();
    let temp_bitcode_file = NamedTempFile::new().unwrap();
    let file_path = temp_bitcode_file.into_temp_path();
    compile_file(path, file_path.as_os_str().into(), &context);
    Command::new("lli")
        .args(&[file_path.as_os_str().to_str().unwrap()])
        .output()
        .expect("Failed to execute `lli`")
        .status
}

fn compile_file<T>(input_file: T, bitcode_file_output: T, context: &Context)
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
        context,
        &module,
        &function_pass_manager,
        &mut environment_stack,
        &mut function_stack,
    );
    let is_successful = compiler.compile(declarations).is_ok();
    compiler.finish(bitcode_file_output, is_successful);
}
