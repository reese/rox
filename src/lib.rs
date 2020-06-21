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

pub use crate::roxc::Result;
use crate::roxc::{init_object_module, init_simplejit_module, Compiler};
use core::mem;
use cranelift_module::FuncOrDataId;
use std::env::temp_dir;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;

/// `run_file` reads the contents of the given path
/// and runs them through the roxc.
///
/// # Errors
/// Currently, the Rox roxc exits with a
/// `1` error code if it fails to compile. In the future,
/// it would likely be a good idea to refactor these
/// to have POSIX compliant error codes, or at least
/// some consistent error code system.
pub fn build_file(path: PathBuf, output: PathBuf, no_link: bool) -> Result<()> {
    build_source_string(output, no_link, path)
}

/// Builds the given source string and links to the output file
pub fn build_source_string(
    output: PathBuf,
    no_link: bool,
    source: PathBuf,
) -> Result<()> {
    compile_and_maybe_link(source, output, no_link)
}

/// Run the given file with the SimpleJITBackend
pub fn run_file(path: PathBuf) -> Result<()> {
    let mut compiler = Compiler::new(init_simplejit_module());
    let compile_result = compiler.compile(&path);
    if compile_result.is_err() {
        return Err(compile_result.err().unwrap());
    }
    let Compiler { mut module, .. } = compiler;
    let func_id = module.get_name("main").unwrap();
    // TODO: Handle `argc` and `argv` in `main`
    // TODO: Actually return the result of this.
    // TODO: Could this unsafe block be pushed into `SimpleJITBackend`?
    match func_id {
        FuncOrDataId::Func(func) => {
            unsafe {
                mem::transmute::<_, fn() -> isize>(
                    module.get_finalized_function(func),
                )()
            };
        }
        _ => unreachable!(),
    }
    Ok(())
}

/// This function generates the native object file
/// and optionally (see the `no_link` argument) links
/// it using the machine's default C compiler.
fn compile_and_maybe_link(
    input_file: PathBuf,
    output: PathBuf,
    no_link: bool,
) -> Result<()> {
    let mut compiler = Compiler::new(init_object_module());
    match compiler.compile(input_file) {
        Err(err) => Err(err),
        Ok(_) => {
            let product = compiler.finish();
            let bytes = product.emit().unwrap();
            if !no_link {
                let temp_output_file = create_temp_object_file();
                let file_result = File::create(temp_output_file.clone())
                    .unwrap()
                    .write_all(&bytes);
                Command::new("cc")
                    .arg(temp_output_file.into_os_string())
                    .arg("-o")
                    .arg(output)
                    .output()
                    .expect("Failed to link output file.");
                file_result.unwrap();
                Ok(())
            } else {
                File::create(output).unwrap().write_all(&bytes).unwrap();
                Ok(())
            }
        }
    }
}

fn create_temp_object_file() -> PathBuf {
    let mut dir = temp_dir();
    dir.push("__rox_temp_build_file.o");
    dir
}
