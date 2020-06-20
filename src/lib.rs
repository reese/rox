#![deny(missing_docs)]
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

use crate::roxc::{init_object_module, init_simplejit_module, Compiler};
use core::mem;
use cranelift_module::FuncOrDataId;
use roxc::{RoxError, RoxErrorType, RoxResult};
use std::env::temp_dir;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::process::{exit, Command};

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
) -> std::io::Result<()> {
    let source = fs::read_to_string(path)?;
    build_source_string(output, no_link, &source);
    Ok(())
}

/// Builds the given source string and links to the output file
pub fn build_source_string(output: PathBuf, no_link: bool, source: &str) {
    let result = compile_and_maybe_link(&source, output, no_link);
    match result {
        Err(RoxError { error_type, .. })
            if error_type == RoxErrorType::CompileError =>
        {
            exit(1)
        }
        Err(_) => exit(1),
        Ok(val) => {
            println!("{:?}", val);
            exit(0)
        }
    };
}

/// Run the given file with the SimpleJITBackend
pub fn run_file(path: PathBuf) -> std::io::Result<()> {
    let source = fs::read_to_string(path)?;
    let mut compiler = Compiler::new(init_simplejit_module());
    compiler.compile(source.as_ref()).unwrap();
    let Compiler { mut module, .. } = compiler;
    let func_id = module.get_name("main").unwrap();
    // TODO: Handle `argc` and `argv` in `main`
    // TODO: Actually return the result of this.
    // TODO: Could this unsafe block be pushed into `SimpleJITBackend`?
    match func_id {
        FuncOrDataId::Func(func) => {
            let main_func = module.get_finalized_function(func);
            let main = unsafe { mem::transmute::<_, fn() -> isize>(main_func) };
            main();
        }
        _ => panic!("Pointer returned for `main` was not a function."),
    }
    Ok(())
}

/// This function generates the native object file
/// and optionally (see the `no_link` argument) links
/// it using the machine's default C compiler.
fn compile_and_maybe_link(
    input: &str,
    output: PathBuf,
    no_link: bool,
) -> RoxResult<()> {
    let mut compiler = Compiler::new(init_object_module());
    compiler.compile(input).unwrap();
    let product = compiler.finish();
    let bytes = product.emit().unwrap();
    if !no_link {
        let temp_output_file = create_temp_object_file();
        File::create(temp_output_file.clone())
            .unwrap()
            .write_all(&bytes)
            .unwrap();
        Command::new("cc")
            .arg(temp_output_file.into_os_string())
            .arg("-o")
            .arg(output)
            .output()
            .expect("Failed to link output file.");
        Ok(())
    } else {
        File::create(output).unwrap().write_all(&bytes).unwrap();
        Ok(())
    }
}

fn create_temp_object_file() -> PathBuf {
    let mut dir = temp_dir();
    dir.push("__rox_temp_build_file.o");
    dir
}
