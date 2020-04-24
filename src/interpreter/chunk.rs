use super::op_code::OpCode;
use super::traits::Push;
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
}

impl Chunk {
    pub fn new() -> Chunk {
        Chunk {
            codes: vec![],
            constants: ValueArray::new(),
        }
    }

    pub fn constant_at(&self, index: usize) -> &Value {
        &self.constants.values[index]
    }
}

impl Clone for Chunk {
    fn clone(&self) -> Chunk {
        Chunk {
            codes: self.codes.clone(),
            constants: self.constants.clone(),
        }
    }
}

impl Push<Byte> for Chunk {
    fn push(&mut self, byte: Byte) -> u8 {
        self.codes.push(byte);
        (self.codes.len() - 1) as u8
    }
}

impl Push<Value> for Chunk {
    fn push(&mut self, byte: Value) -> u8 {
        self.constants.push(byte);
        (self.constants.values.len() - 1) as u8
    }
}
