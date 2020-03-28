use super::common::OpCode;
use super::traits::*;
use super::value::{Value, ValueArray};

#[derive(Clone, Debug)]
pub enum Byte {
    Op(OpCode),
    Constant(u8),
}

#[derive(Debug)]
pub struct Chunk {
    pub codes: Vec<Byte>,
    pub constants: ValueArray,
    pub lines: Vec<i32>,
}

impl Chunk {
    pub fn new() -> Chunk {
        Chunk {
            codes: vec![],
            constants: ValueArray::new(),
            lines: vec![],
        }
    }
}

impl Clone for Chunk {
    fn clone(&self) -> Chunk {
        Chunk {
            codes: self.codes.clone(),
            constants: self.constants.clone(),
            lines: self.lines.clone(),
        }
    }
}

impl PushLine<Byte> for Chunk {
    fn push_line(&mut self, byte: Byte, line: i32) -> u8 {
        self.codes.push(byte);
        self.lines.push(line);
        (self.codes.len() - 1) as u8
    }
}

impl PushLine<Value> for Chunk {
    fn push_line(&mut self, byte: Value, line: i32) -> u8 {
        self.constants.push(byte);
        self.lines.push(line);
        (self.constants.values.len() - 1) as u8
    }
}
