use super::interpret_result::InterpretResult;
use super::chunk::Chunk;
use super::common::OpCode;
use super::value::Value;

#[derive(Debug)]
pub struct VM {
  pub chunk: Chunk,
  ips: Vec<OpCode>,
  stack: Vec<Value>,
}

impl VM {
  pub fn new(chunk: Chunk) -> VM {
    return VM { 
      chunk: chunk,
      ips: vec![],
      stack: vec![]
    }
  }

  pub fn interpret(&mut self) -> InterpretResult {
    self.ips = self.chunk.codes.clone();
    return self.run()
  }

  fn run(&mut self) -> InterpretResult {
    let mut code_index = 0;
    let mut constant_index = 0;
    while code_index < self.ips.len() {
      match self.ips[code_index] {
        OpCode::OpReturn => {
          println!("{:?}", self.stack.pop());
          return InterpretResult::InterpretOk
        },
        OpCode::OpNegate => {
          let next_constant = self.get_next_constant();
          self.stack.push(-next_constant)
        },
        OpCode::OpAdd => self.binary_operation("+"),
        OpCode::OpSubtract => self.binary_operation("-"),
        OpCode::OpMultiply => self.binary_operation("*"),
        OpCode::OpDivide => self.binary_operation("/"),
        OpCode::OpConstant => {
          let constant = &self.chunk.constants.values[constant_index];
          constant_index += 1;
          &self.stack.push(constant.clone());
        },
      }
      code_index += 1;
    }
    return InterpretResult::InterpretCompileError
  }

  fn binary_operation(&mut self, operation: &str) {
    let first = self.get_next_constant();
    let second = self.get_next_constant();
    let result = match operation {
      "+" => first + second,
      "-" => first - second,
      "*" => first * second,
      "/" => first / second,
      _ => panic!("Unknown binary operation attempted.")
    };
    self.stack.push(result)
  }

  fn get_next_constant(&mut self) -> Value {
    match self.stack.pop() {
      Some(x) => return x,
      None => panic!("Nothing on the constants stack to pop"),
    }
  }
}

