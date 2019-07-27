use super::interpret_result::InterpretResult;
use super::chunk::Chunk;
use super::common::OpCode;

#[derive(Debug)]
pub struct VM {
  pub chunk: Chunk,
  ips: Vec<OpCode>,
}

impl VM {
  pub fn new(chunk: Chunk) -> VM {
    return VM { 
      chunk: chunk,
      ips: vec![]
    }
  }

  pub fn interpret(&mut self) -> InterpretResult {
    self.ips = self.chunk.codes.clone();
    return self.run()
  }

  fn run(&self) -> InterpretResult {
    let mut code_index = 0;
    let mut constant_index = 0;
    while code_index < self.ips.len() {
      match self.ips[code_index] {
        OpCode::OpReturn => return InterpretResult::InterpretOk,
        OpCode::OpConstant => {
          let constant = &self.chunk.constants.values[constant_index];
          constant_index += 1;
          println!("{:?}", constant);
        },
      }
      code_index += 1;
    }
    return InterpretResult::InterpretCompileError
  }
}

