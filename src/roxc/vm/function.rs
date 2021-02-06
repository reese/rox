use crate::roxc::vm::Chunk;

#[derive(Clone, Debug, PartialOrd, PartialEq)]
pub(crate) struct Function {
    arity: u8,
    pub(crate) chunk: Chunk,
    pub(crate) name: String,
}

impl Function {
    pub(crate) fn new_main() -> Self {
        Function {
            arity: 0,
            chunk: Chunk::new(),
            name: String::from(""),
        }
    }

    pub(crate) fn new(arity: u8, chunk: Chunk, name: String) -> Self {
        Function { arity, chunk, name }
    }

    pub(crate) fn get_chunk(&self) -> &Chunk {
        &self.chunk
    }

    pub(crate) fn get_mut_chunk(&mut self) -> &mut Chunk {
        &mut self.chunk
    }
}
