#[macro_use]
extern crate text_io;
#[macro_use]
extern crate matches;

use lib::{Chunk, InterpretError, InterpretErrorType, RoxResult, Value, VM};
use std::env::args;
use std::fs;
use std::io::{self, Write};
use std::process::exit;

mod lib;

fn main() {
  let arguments: Vec<String> = args().collect();
  if arguments.len() == 1 {
    repl();
  } else if arguments.len() == 2 {
    run_file(&arguments[1]).unwrap();
  } else {
    panic!("Usage: rox [path_to_run]")
  }
  exit(0);
}

fn run_file(path: &String) -> std::io::Result<()> {
  let source = fs::read(path)?;
  let result = interpret(source);
  match result {
    Err(InterpretError { error_type, .. })
      if error_type == InterpretErrorType::InterpretCompileError =>
    {
      exit(65)
    }
    Err(InterpretError { error_type, .. })
      if error_type == InterpretErrorType::InterpretRuntimeError =>
    {
      exit(70)
    }
    Err(_) => exit(1),
    Ok(val) => {
      println!("{:?}", val);
      exit(0)
    }
  };
}

fn repl() {
  let _lines: Vec<String> = vec![];
  loop {
    print!("\nrox > ");
    io::stdout().flush().unwrap();

    let input_string: String = read!();
    println!("{:?}", input_string);
    if input_string == String::from("exit()") {
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
