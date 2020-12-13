pub(crate) mod chunk;
pub(crate) mod object;
pub(crate) mod opcode;
pub(crate) mod value;

use super::errors::Result;
pub(crate) use crate::roxc::vm::opcode::OpCode;
pub(crate) use crate::roxc::vm::value::Value;
use crate::roxc::{RoxError, Stack};
pub(crate) use chunk::Chunk;
use im::HashMap;

pub(crate) struct VM {
    chunk: Chunk,
    stack: Stack<Value>,
    variables: Stack<HashMap<String, Value>>,
}

impl VM {
    pub(crate) fn new(chunk: Chunk) -> Self {
        let mut variables = Stack::new();
        variables.push(HashMap::new());
        VM {
            chunk,
            stack: Stack::new(),
            variables,
        }
    }

    pub(crate) fn interpret(&mut self) -> Result<()> {
        let mut instruction_pointer = self.chunk.opcodes.iter();
        while let Some(ip) = instruction_pointer.next() {
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
                OpCode::DeclareVariable => {
                    let name = self.stack.pop().unwrap().read_string().clone();
                    // Don't allow variable shadowing
                    if self.variables.top().contains_key(name.as_str()) {
                        return Err(RoxError::with_file_placeholder(
                            "This key is already defined",
                        ));
                    }
                    // N.B. We might want to read, store, then pop in case of garbage collection
                    let value = self.stack.pop().unwrap();
                    self.variables.top_mut().insert(name, value);
                }
                OpCode::ReadVariable => {
                    let name = self.stack.pop().unwrap().read_string().clone();
                    if let Some(value) = self.variables.top().get(&name) {
                        self.stack.push(value.clone())
                    } else {
                        return Err(RoxError::with_file_placeholder(
                            "Variable has not yet been defined",
                        ));
                    }
                }
                OpCode::AssignVariable => {
                    let name = self.stack.pop().unwrap().read_string().clone();
                    if !self.variables.top().contains_key(name.as_str()) {
                        return Err(RoxError::with_file_placeholder(
                            "Variable has not yet been defined",
                        ));
                    }
                    // N.B. We might want to read, store, then pop in case of garbage collection
                    let value = self.stack.pop().unwrap();
                    self.variables.top_mut().insert(name, value);
                }
            }
        }
        Ok(())
    }
}
