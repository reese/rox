mod call_frame;
pub(crate) mod chunk;
pub(crate) mod function;
pub(crate) mod object;
pub(crate) mod opcode;
pub(crate) mod value;

use super::errors::Result;
use crate::roxc::stack::Stack;
use crate::roxc::vm::call_frame::CallFrame;
use crate::roxc::vm::function::Function;
pub(crate) use crate::roxc::vm::opcode::OpCode;
pub(crate) use crate::roxc::vm::value::Value;
pub(crate) use chunk::Chunk;
use std::cell::RefCell;
use std::rc::Rc;

pub(crate) struct VM {
    stack: Rc<RefCell<Stack<Value>>>,
    frames: Stack<CallFrame>,
}

impl VM {
    pub(crate) fn new() -> Self {
        VM {
            stack: Rc::new(RefCell::new(Stack::new())),
            frames: Stack::new(),
        }
    }

    pub(crate) fn interpret(&mut self, function: Function) -> Result<()> {
        // self.push(Value::Obj(Rc::new(Object::Function(function.clone()))));
        let mut call_frame = CallFrame {
            function,
            instruction_pointer: 0,
            // TODO: We should move this behind a shared
            // mutable reference so we don't have to clone
            // the whole stack :grimace:
            slots: self.stack.clone(),
        };
        while call_frame.instruction_pointer
            < call_frame.get_chunk().opcodes.len()
        {
            let ip = call_frame
                .get_chunk()
                .opcodes
                .get(call_frame.instruction_pointer)
                .unwrap()
                .clone();
            match ip {
                OpCode::Return => return Ok(()),
                OpCode::Constant(index) => {
                    let constant =
                        call_frame.get_chunk().constants.get(index).unwrap();
                    self.push(constant.clone())
                }
                OpCode::Negate => {
                    let val = self
                        .stack
                        .borrow_mut()
                        .pop()
                        .expect("No constants to pop from stack");
                    self.push(val * Value::Number(-1.0));
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
                    println!("{:?}", self.stack.borrow().get_inner_array());
                    let right = self.pop();
                    let left = self.pop();
                    self.push(Value::Bool(left < right));
                }
                OpCode::Pop => {
                    self.pop();
                }
                OpCode::ReadVariable(index) => {
                    self.push(call_frame.read_slot(index));
                }
                OpCode::AssignVariable(index) => {
                    // N.B. We might want to read, store, then pop in case of garbage collection
                    let value = self.pop().clone();
                    call_frame.set_slot(index, value);
                }
                OpCode::Placeholder => unreachable!(
                    "The jump offset placeholder was never replaced."
                ),
                OpCode::JumpIfFalse => {
                    let conditional = self.pop().read_bool();
                    call_frame.instruction_pointer += 1;
                    // Offset is in next instruction
                    match call_frame.get_chunk().opcodes
                        [call_frame.instruction_pointer]
                    {
                        OpCode::Offset(offset) => {
                            if !conditional {
                                call_frame.instruction_pointer += offset
                            }
                        }
                        _ => unreachable!(),
                    }
                }
                OpCode::Jump => {
                    call_frame.instruction_pointer += 1;
                    match call_frame.get_chunk().opcodes
                        [call_frame.instruction_pointer]
                    {
                        OpCode::Offset(offset) => {
                            call_frame.instruction_pointer += offset
                        }
                        _ => unreachable!(),
                    }
                }
                OpCode::Loop => {
                    call_frame.instruction_pointer += 1;

                    match call_frame.get_chunk().opcodes
                        [call_frame.instruction_pointer]
                    {
                        OpCode::Offset(offset) => {
                            call_frame.instruction_pointer -= offset
                        }
                        _ => unreachable!(),
                    }
                }
                OpCode::Offset(_offset) => unreachable!(
                    "Offsets are only read in other OpCode implementations"
                ),
            }
            call_frame.instruction_pointer += 1;
        }
        Ok(())
    }

    fn push(&self, value: Value) {
        self.stack.borrow_mut().push(value);
    }

    fn pop(&self) -> Value {
        self.stack.borrow_mut().pop().unwrap()
    }
}
