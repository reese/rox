use crate::interpreter::function::Function;
use crate::Value;

#[derive(Debug)]
pub struct CallFrame {
    function: Box<Function>,
    instruction_pointer: usize,
    slots: Vec<Value>,
}

impl CallFrame {
    pub fn new(function: Box<Function>, instruction_pointer: usize) -> Self {
        CallFrame {
            function,
            instruction_pointer,
            slots: Vec::new(),
        }
    }

    pub fn get_instruction_pointer(&self) -> usize {
        self.instruction_pointer
    }

    pub fn increment_instruction_pointer(&mut self, offset: usize) {
        self.instruction_pointer += offset;
    }

    pub fn decrement_instruction_pointer(&mut self, offset: usize) {
        self.instruction_pointer -= offset;
    }

    pub fn push_constant(&mut self, value: Value) {
        self.slots.push(value);
    }
    
    pub fn pop_constant(&mut self) -> Option<Value> {
        self.slots.pop()
    }
}
