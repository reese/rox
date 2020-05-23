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
use roxc::{InterpretError, InterpretErrorType, RoxResult};
use std::fs;
use std::path::Path;
use std::process::exit;

/// # `Rox` macro
/// The `rox!` macro allows you to embed `rox` directly
/// into your Rust applications.
///
/// ```
/// # #[macro_use]
/// # extern crate rox;
/// # fn main() {
/// let result = rox! {
///     let x = 4;
///     while x < 10 {
///         print x;
///         x = x + 1;
///     }
///     return x;
/// };
/// assert!(result.is_ok())
/// # }
/// ```
#[macro_export]
macro_rules! rox {
    ($ ($ t : tt) *) => {
        rox::interpret(stringify!($($t)*));
    };
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
pub fn run_file(path: &Path) -> std::io::Result<()> {
    let source = fs::read_to_string(path)?;
    let result = interpret(&source);
    match result {
        Err(InterpretError { error_type, .. })
            if error_type == InterpretErrorType::InterpretCompileError =>
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

/// The `interpret` function runs the given source code (as a `&str`)
/// through the roxc and returns the resulting value.
pub fn interpret(input: &str) -> RoxResult<()> {
    let mut compiler = Compiler::new("test.o");
    compiler.compile(input).unwrap();
    compiler.finish(Path::new("test.o")).unwrap();
    Ok(())
}
