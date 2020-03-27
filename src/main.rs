#[macro_use]
extern crate text_io;
#[macro_use]
extern crate matches;

use std::env::args;
use std::fs;
use std::io::{self, Write};
use std::process::exit;

mod lib;
use lib::*;

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
  println!("{:?}", source);
  let result: InterpretResult = interpret(source);
  match result {
    InterpretResult::InterpretCompileError => exit(65),
    InterpretResult::InterpretRuntimeError => exit(70),
    _ => exit(0),
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

    interpret(input_string.as_bytes().to_vec());
  }
}

fn interpret(input: Vec<u8>) -> InterpretResult {
  let chunk = &mut Chunk::new();
  VM::new(chunk).interpret(input)
}
