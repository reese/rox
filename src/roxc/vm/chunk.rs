use super::opcode::OpCode;
use super::value::Value;

#[derive(Debug, Clone, PartialOrd, PartialEq)]
pub(crate) struct Chunk {
    pub(crate) opcodes: Vec<OpCode>,
    pub(crate) constants: Vec<Value>,
}

impl Chunk {
    pub fn new() -> Self {
        Chunk {
            opcodes: Vec::new(),
            constants: Vec::new(),
        }
    }

    pub fn write(&mut self, op: OpCode) {
        self.opcodes.push(op);
    }

    pub fn add_constant(&mut self, value: Value) {
        self.constants.push(value);
        self.opcodes
            .push(OpCode::Constant(self.constants.len() - 1));
    }
}
