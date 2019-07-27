use super::common::OpCode;
use super::value::{Value, ValueArray};
use super::traits::*;

#[derive(Debug)]
pub struct Chunk {
  pub codes: Vec<OpCode>,
  pub constants: ValueArray,
  pub lines: Vec<i32>,
}

impl Chunk {
  pub fn new() -> Chunk {
    return Chunk {
      codes: vec![],
      constants: ValueArray::new(),
      lines: vec![]
    }
  }
}

impl Clone for Chunk {
  fn clone(&self) -> Chunk {
    return Chunk {
      codes: self.codes.clone(),
      constants: self.constants.clone(),
      lines: self.lines.clone(),
    }
  }
}

impl PushLine<OpCode> for Chunk {
  fn push_line(&mut self, byte: OpCode, line: i32) -> usize {
    self.codes.push(byte);
    self.lines.push(line);
    return self.codes.len() - 1
  }
}

impl PushLine<Value> for Chunk {
  fn push_line(&mut self, byte: Value, line: i32) -> usize {
    self.constants.push(byte);
    self.lines.push(line);
    return self.constants.values.len() - 1
  }
}
