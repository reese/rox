mod call_frame;
mod call_stack;
pub(crate) mod chunk;
pub(crate) mod function;
pub(crate) mod object;
pub(crate) mod opcode;
pub(crate) mod value;

use super::errors::Result;
use crate::roxc::stack::Stack;
use crate::roxc::vm::call_frame::CallFrame;
use crate::roxc::vm::call_stack::CallStack;
use crate::roxc::vm::function::Function;
pub(crate) use crate::roxc::vm::opcode::OpCode;
pub(crate) use crate::roxc::vm::value::Value;
pub(crate) use chunk::Chunk;

pub(crate) struct VM {
    stack: CallStack,
    frames: Stack<CallFrame>,
}

impl VM {
    pub(crate) fn new() -> Self {
        VM {
            stack: CallStack::new(),
            frames: Stack::new(),
        }
    }

    pub(crate) fn interpret(&mut self, function: Function) -> Result<()> {
        // self.push(Value::Obj(Rc::new(Object::Function(function.clone()))));
        let frame = CallFrame {
            function,
            instruction_pointer: 0,
            slots_start_offset: 0,
        };
        self.frames.push(frame);
        while self.get_instruction_pointer() < self.get_chunk_ip_length() {
            let ip = self.get_current_instruction();
            match ip {
                OpCode::Return => {
                    let result = self.pop();
                    self.frames.pop();
                    if self.frames.is_empty() {
                        self.pop();
                        return Ok(());
                    }

                    self.stack.push(result)
                }
                OpCode::Constant(index) => {
                    let constant = self.get_constant(index);
                    self.push(constant.clone())
                }
                OpCode::Negate => {
                    let val = self.stack.pop();
                    self.stack.push(val * Value::Number(-1.0));
                }
                OpCode::Add => {
                    let right = self.pop();
                    let left = self.pop();
                    self.push(left + right);
                }
                OpCode::Subtract => {
                    let right = self.pop();
                    let left = self.pop();
                    self.push(left - right);
                }
                OpCode::Multiply => {
                    let right = self.pop();
                    let left = self.pop();
                    self.push(left * right);
                }
                OpCode::Divide => {
                    let right = self.pop();
                    let left = self.pop();
                    self.push(left / right);
                }
                OpCode::True => self.push(Value::Bool(true)),
                OpCode::False => self.push(Value::Bool(false)),
                OpCode::Not => {
                    let val = self.pop();
                    self.push(!val);
                }
                OpCode::Equal => {
                    let right = self.pop();
                    let left = self.pop();
                    self.push(Value::Bool(left == right));
                }
                OpCode::Greater => {
                    let right = self.pop();
                    let left = self.pop();
                    self.push(Value::Bool(left > right));
                }
                OpCode::Less => {
                    println!("{:?}", self.stack);
                    let right = self.pop();
                    let left = self.pop();
                    self.push(Value::Bool(left < right));
                }
                OpCode::Pop => {
                    self.pop();
                }
                OpCode::ReadVariable(index) => {
                    self.push(self.read_slot(index));
                }
                OpCode::AssignVariable(index) => {
                    // N.B. We might want to read, store, then pop in case of garbage collection
                    let value = self.pop().clone();
                    self.set_slot(index, value);
                }
                OpCode::Placeholder => unreachable!(
                    "The jump offset placeholder was never replaced."
                ),
                OpCode::JumpIfFalse => {
                    let conditional = self.pop().read_bool();
                    self.advance_instruction_pointer();
                    // Offset is in next instruction
                    match self.get_current_instruction() {
                        OpCode::Offset(offset) => {
                            if !conditional {
                                self.advance_instruction_pointer_by_offset(
                                    offset,
                                )
                            }
                        }
                        _ => unreachable!(),
                    }
                }
                OpCode::Jump => {
                    self.advance_instruction_pointer();
                    match self.get_current_instruction() {
                        OpCode::Offset(offset) => {
                            self.advance_instruction_pointer_by_offset(offset)
                        }
                        _ => unreachable!(),
                    }
                }
                OpCode::Loop => {
                    self.advance_instruction_pointer();

                    match self.get_current_instruction() {
                        OpCode::Offset(offset) => {
                            self.reduce_instruction_pointer_by_offset(offset)
                        }
                        _ => unreachable!(),
                    }
                }
                OpCode::Offset(_offset) => unreachable!(
                    "Offsets are only read in other OpCode implementations"
                ),
                OpCode::Call(arg_count) => {
                    let func_value = self.read_slot(arg_count);
                    let function = func_value.read_function().clone();
                    let index_to_borrow_after =
                        self.stack.len() - arg_count - 1;
                    let frame = CallFrame {
                        function,
                        instruction_pointer: self.get_instruction_pointer(),
                        slots_start_offset: index_to_borrow_after,
                    };
                    self.frames.push(frame)
                }
            }
            self.advance_instruction_pointer();
        }
        Ok(())
    }

    fn get_constant(&self, index: usize) -> &Value {
        self.frames.top().get_chunk().constants.get(index).unwrap()
    }

    fn get_current_instruction(&self) -> OpCode {
        self.frames
            .top()
            .get_chunk()
            .opcodes
            .get(self.get_instruction_pointer())
            .unwrap()
            .clone()
    }

    fn get_chunk_ip_length(&self) -> usize {
        self.frames.top().get_chunk().opcodes.len()
    }

    fn get_instruction_pointer(&self) -> usize {
        self.frames.top().instruction_pointer
    }

    fn advance_instruction_pointer(&mut self) {
        self.advance_instruction_pointer_by_offset(1)
    }

    fn advance_instruction_pointer_by_offset(&mut self, offset: usize) {
        self.frames.top_mut().instruction_pointer += offset
    }

    fn reduce_instruction_pointer_by_offset(&mut self, offset: usize) {
        self.frames.top_mut().instruction_pointer -= offset
    }

    fn push(&mut self, value: Value) {
        self.stack.push(value);
    }

    fn pop(&mut self) -> Value {
        self.stack.pop()
    }

    fn read_slot(&self, index: usize) -> Value {
        self.stack.get(self.frames.top().slots_start_offset + index)
    }

    fn set_slot(&mut self, index: usize, value: Value) {
        self.stack
            .set(self.frames.top().slots_start_offset + index, value)
    }
}
