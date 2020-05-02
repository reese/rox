#![deny(missing_docs)]
#![doc(
    html_logo_url = "https://github.com/reese/rox/raw/master/assets/geodude.png"
)]
#![doc(
    html_favicon_url = "https://github.com/reese/rox/raw/master/assets/favicon.png"
)]
//! # Rox
//! `Rox` is based on the `Lox` language implemented in Bob Nystrom's
//! series, "Crafting Interpreters."

#[macro_use]
extern crate lalrpop_util;

mod interpreter;

pub use interpreter::Value;
use interpreter::{InterpretError, InterpretErrorType, RoxResult, VM};
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
/// use rox::Value;
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
/// and runs them through the interpreter.
///
/// # Errors
/// Currently, the Rox interpreter exits with a
/// `1` error code if it fails during compilation
/// and a `2` if it fails at runtime. In the future,
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
        Err(InterpretError { error_type, .. })
            if error_type == InterpretErrorType::InterpretRuntimeError =>
        {
            exit(2)
        }
        Err(_) => exit(1),
        Ok(val) => {
            println!("{:?}", val);
            exit(0)
        }
    };
}

/// The `interpret` function runs the given source code (as a `&str`)
/// through the interpreter and returns the resulting value.
pub fn interpret(input: &str) -> RoxResult<Value> {
    VM::new().interpret(input)
}
