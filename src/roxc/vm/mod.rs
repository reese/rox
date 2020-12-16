pub(crate) mod chunk;
pub(crate) mod object;
pub(crate) mod opcode;
pub(crate) mod value;

use super::errors::Result;
use crate::roxc::stack::Stack;
pub(crate) use crate::roxc::vm::opcode::OpCode;
pub(crate) use crate::roxc::vm::value::Value;
pub(crate) use chunk::Chunk;
use std::hint::unreachable_unchecked;

pub(crate) struct VM {
    chunk: Chunk,
    stack: Stack<Value>,
}

impl VM {
    pub(crate) fn new(chunk: Chunk) -> Self {
        VM {
            chunk,
            stack: Stack::new(),
        }
    }

    pub(crate) fn interpret(&mut self) -> Result<()> {
        let mut instruction_pointer = 0;
        while instruction_pointer < self.chunk.opcodes.len() {
            let ip = self.chunk.opcodes.get(instruction_pointer).unwrap();
            match ip {
                OpCode::Return => return Ok(()),
                OpCode::Constant(index) => {
                    let constant = self.chunk.constants.get(*index).unwrap();
                    self.stack.push(constant.clone())
                }
                OpCode::Negate => {
                    let val = self
                        .stack
                        .pop()
                        .expect("No constants to pop from stack");
                    self.stack.push(val * Value::Number(-1.0));
                }
                OpCode::Add => {
                    let right = self
                        .stack
                        .pop()
                        .expect("No constants to pop from stack");
                    let left = self
                        .stack
                        .pop()
                        .expect("No constants to pop from stack");
                    self.stack.push(left + right);
                }
                OpCode::Subtract => {
                    let right = self
                        .stack
                        .pop()
                        .expect("No constants to pop from stack");
                    let left = self
                        .stack
                        .pop()
                        .expect("No constants to pop from stack");
                    self.stack.push(left - right);
                }
                OpCode::Multiply => {
                    let right = self
                        .stack
                        .pop()
                        .expect("No constants to pop from stack");
                    let left = self
                        .stack
                        .pop()
                        .expect("No constants to pop from stack");
                    self.stack.push(left * right);
                }
                OpCode::Divide => {
                    let right = self
                        .stack
                        .pop()
                        .expect("No constants to pop from stack");
                    let left = self
                        .stack
                        .pop()
                        .expect("No constants to pop from stack");
                    self.stack.push(left / right);
                }
                OpCode::True => self.stack.push(Value::Bool(true)),
                OpCode::False => self.stack.push(Value::Bool(false)),
                OpCode::Not => {
                    let val = self.stack.pop().unwrap();
                    self.stack.push(!val);
                }
                OpCode::Equal => {
                    let right = self.stack.pop().unwrap();
                    let left = self.stack.pop().unwrap();
                    self.stack.push(Value::Bool(left == right));
                }
                OpCode::Greater => {
                    let right = self.stack.pop().unwrap();
                    let left = self.stack.pop().unwrap();
                    self.stack.push(Value::Bool(left > right));
                }
                OpCode::Less => {
                    let right = self.stack.pop().unwrap();
                    let left = self.stack.pop().unwrap();
                    self.stack.push(Value::Bool(left < right));
                }
                OpCode::Pop => {
                    self.stack.pop().unwrap();
                }
                OpCode::ReadVariable(index) => {
                    self.stack.push(self.stack.get_unchecked(*index).clone());
                }
                OpCode::AssignVariable(index) => {
                    // N.B. We might want to read, store, then pop in case of garbage collection
                    let value = self.stack.pop().unwrap().clone();
                    self.stack.set(*index, value);
                }
                OpCode::Placeholder => unreachable!(
                    "The jump offset placeholder was never replaced."
                ),
                OpCode::JumpIfFalse => {
                    let conditional = self.stack.pop().unwrap().read_bool();
                    instruction_pointer += 1;
                    // Offset is in next instruction
                    match self.chunk.opcodes[instruction_pointer] {
                        OpCode::Offset(offset) => {
                            if !conditional {
                                instruction_pointer += offset
                            }
                        }
                        _ => unreachable!(),
                    }
                }
                OpCode::Jump => {
                    instruction_pointer += 1;
                    match self.chunk.opcodes[instruction_pointer] {
                        OpCode::Offset(offset) => instruction_pointer += offset,
                        _ => unreachable!(),
                    }
                }
                OpCode::Loop => {
                    instruction_pointer += 1;

                    match self.chunk.opcodes[instruction_pointer] {
                        OpCode::Offset(offset) => instruction_pointer -= offset,
                        _ => unreachable!(),
                    }
                }
                OpCode::Offset(offset) => unreachable!(),
            }
            instruction_pointer += 1;
        }
        Ok(())
    }
}