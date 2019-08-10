#[macro_use]
extern crate text_io;

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
      run_file(&arguments[1]);
    } else {
      panic!("Usage: rox [path_to_run]")
    }
    exit(0);
    // let mut chunk = Chunk::new();
    // chunk.push_line(OpCode::OpConstant, 123);
    // chunk.push_line(Value { f: 1.2 }, 123);
    // chunk.push_line(OpCode::OpConstant, 7);
    // chunk.push_line(Value { f: 78 as f64 }, 7);
    // chunk.push_line(OpCode::OpNegate, 123);
    // chunk.push_line(OpCode::OpConstant, 7);
    // chunk.push_line(Value { f: 4 as f64 }, 7);
    // chunk.push_line(OpCode::OpConstant, 7);
    // chunk.push_line(Value { f: 5 as f64 }, 7);
    // chunk.push_line(OpCode::OpSubtract, 7);
    // chunk.push_line(OpCode::OpReturn, 7);
    // let mut vm = VM::new(chunk);
    // vm.interpret();
    // println!("{:?}", vm)
}

fn run_file(path: &String) -> std::io::Result<()> {
  let source = fs::read(path)?;
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
    if input_string == String::from("exit()") {
      println!("\nexit");
      break;
    }

    interpret(input_string.as_bytes().to_vec());
  }
}

fn interpret(input: Vec<u8>) -> InterpretResult {
  let chunk = &mut Chunk::new();
  let mut compiler = Compiler::new(&input, chunk);
  compiler.compile();
  return InterpretResult::InterpretOk
}
