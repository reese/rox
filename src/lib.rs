#![deny(missing_docs)]
#![doc(
    html_logo_url = "https://github.com/reese/rox/raw/master/assets/rox-logo-transparent-bg.png"
)]
#![doc(
    html_favicon_url = "https://github.com/reese/rox/raw/master/assets/favicon.png"
)]
//! # Rox
//! `Rox` is high-level, statically-typed, compiled language that mixes
//! the familiarity of C-style syntax with the rigorous and expressive features
//! of functional languages.

#[macro_use]
extern crate lalrpop_util;

mod roxc;

use crate::roxc::Compiler;
use roxc::{RoxError, RoxErrorType, RoxResult};
use std::env::temp_dir;
use std::fs;
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
pub(crate) fn build_file(
    path: PathBuf,
    output: PathBuf,
    no_link: bool,
) -> std::io::Result<()> {
    let source = fs::read_to_string(path)?;
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

/// This function generates the native object file
/// and optionally (see the `no_link` argument) links
/// it using the machine's default C compiler.
pub(crate) fn compile_and_maybe_link(
    input: &str,
    output: PathBuf,
    no_link: bool,
) -> RoxResult<()> {
    let mut compiler = Compiler::new();
    compiler.compile(input).unwrap();
    if !no_link {
        let temp_output_file = create_temp_object_file();
        compiler.finish(&temp_output_file).unwrap();
        Command::new("cc")
            .arg(temp_output_file.into_os_string())
            .arg("-o")
            .arg(output)
            .output()
            .expect("Failed to link output file.");
        Ok(())
    } else {
        compiler.finish(&output).unwrap();
        Ok(())
    }
}

fn create_temp_object_file() -> PathBuf {
    let mut dir = temp_dir();
    dir.push("__rox_temp_build_file.o");
    dir
}
