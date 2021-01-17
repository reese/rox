use crate::roxc::vm::function::Function;
use crate::roxc::vm::Chunk;

#[derive(Debug)]
pub(crate) struct CallFrame {
    pub(crate) function: Function,
    pub(crate) instruction_pointer: usize,
    pub(crate) slots_start_offset: usize,
}

impl CallFrame {
    pub(crate) fn get_chunk(&self) -> &Chunk {
        self.function.get_chunk()
    }
}
