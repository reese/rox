#![deny(missing_docs)]
//! # Rox
//! `Rox` is an interpreted, dynamically typed language
//! based on the `Lox` language implemented in Bob Nystrom's
//! series, "Crafting Interpreters."

#[macro_use]
extern crate text_io;

mod interpreter;

use interpreter::{
    Chunk, InterpretError, InterpretErrorType, RoxResult, Value, VM,
};
use std::fs;
use std::io::{self, Write};
use std::path::Path;
use std::process::exit;

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
    let source = fs::read(path)?;
    let result = interpret(source);
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

/// `repl` starts and evaluates in a REPL
/// (Read-Evaluate-Print-Loop), taking in
/// some input string, running it through the interpreter,
/// and returning the value. Note that this REPL doesn't currently
/// maintain any sort of state (since we don't support variable) assignment
/// yet, so this is really just a glorified calculator at the moment.
pub fn repl() {
    loop {
        print!("\nrox > ");
        io::stdout().flush().unwrap();

        let input_string: String = read!();
        println!("{:?}", input_string);
        if input_string.eq("exit()") {
            println!("exit");
            break;
        }

        interpret(input_string.as_bytes().to_vec()).unwrap();
    }
}

fn interpret(input: Vec<u8>) -> RoxResult<Value> {
    let chunk = &mut Chunk::new();
    VM::new(chunk).interpret(input)
}
