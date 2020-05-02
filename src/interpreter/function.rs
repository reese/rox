use crate::interpreter::Chunk;

#[derive(Debug, PartialOrd, PartialEq)]
pub struct Function {
    arity: u8,
    chunk: Chunk,
}

impl Function {
    pub fn new(arity: u8) -> Self {
        Function {
            arity,
            chunk: Chunk::new(),
        }
    }

    pub fn get_chunk(&mut self) -> &mut Chunk {
        &mut self.chunk
    }
}
