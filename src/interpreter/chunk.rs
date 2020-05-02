use super::op_code::OpCode;
use super::traits::Push;
use super::value::{Value, ValueArray};

#[derive(Copy, Clone, Debug, PartialOrd, PartialEq)]
pub enum Byte {
    Op(OpCode),
    Constant(usize),
}

#[derive(Debug, Default, PartialOrd, PartialEq)]
pub struct Chunk {
    pub codes: Vec<Byte>,
    pub constants: ValueArray,
}

impl Chunk {
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
    fn push(&mut self, byte: Byte) -> usize {
        self.codes.push(byte);
        self.codes.len() - 1
    }
}

impl Push<Value> for Chunk {
    fn push(&mut self, byte: Value) -> usize {
        self.constants.push(byte);
        self.constants.values.len() - 1
    }
}
