use crate::roxc::stack::Stack;
use crate::roxc::vm::function::Function;
use crate::roxc::vm::{Chunk, Value};
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug)]
pub(crate) struct CallFrame {
    pub(crate) function: Function,
    pub(crate) instruction_pointer: usize,
    pub(crate) slots: Rc<RefCell<Stack<Value>>>,
}

impl CallFrame {
    pub(crate) fn get_chunk(&self) -> &Chunk {
        self.function.get_chunk()
    }

    pub(crate) fn read_slot(&self, index: usize) -> Value {
        self.slots.borrow_mut().get(index).clone()
    }

    pub(crate) fn set_slot(&mut self, index: usize, value: Value) {
        self.slots.borrow_mut().set(index, value);
    }
}
