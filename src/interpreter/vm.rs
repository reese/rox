use super::chunk::{Byte, Chunk};
use super::compile::Compiler;
use super::interpret_result::{InterpretError, RoxResult};
use super::op_code::OpCode;
use super::value::Value;
use crate::interpreter::{CallFrame, Operation, Stack};
use im::HashMap;
use std::borrow::BorrowMut;

type Environment = HashMap<String, Value>;

#[derive(Debug)]
pub struct VM {
    compiler: Box<Compiler>,
    scope_stack: Stack<Environment>,
    call_frame_stack: Stack<Box<CallFrame>>,
}

impl VM {
    pub fn new() -> VM {
        let initial_scope: Environment = HashMap::new();
        let mut initial_scope_stack = Stack::new();
        initial_scope_stack.push(initial_scope);

        VM {
            compiler: Box::new(Compiler::new()),
            scope_stack: initial_scope_stack,
            call_frame_stack: Stack::new(),
        }
    }

    pub fn interpret(&mut self, source: &str) -> RoxResult<Value> {
        let result = self.compiler.compile(source);
        if result.is_err() {
            return InterpretError::compile_error();
        }
        self.call_frame_stack.push(Box::new(CallFrame::new(
            Box::new(self.compiler.function.clone()),
            0,
        )));
        self.run()
    }

    fn run(&mut self) -> RoxResult<Value> {
        let instructions = self.compiler.function.get_chunk().codes.clone();

        while self.get_top_call_frame().get_instruction_pointer()
            < instructions.len()
        {
            let instruction = instructions
                .get(self.get_top_call_frame().get_instruction_pointer())
                .expect("Instruction pointer out of bounds");
            match instruction {
                Byte::Op(OpCode::Return) => {}
                Byte::Op(OpCode::Equal) => {
                    self.binary_operation(Operation::Equals)
                }
                Byte::Op(OpCode::GreaterThan) => {
                    self.binary_operation(Operation::GreaterThan)
                }
                Byte::Op(OpCode::LessThan) => {
                    self.binary_operation(Operation::LessThan)
                }
                Byte::Op(OpCode::Negate) => {
                    let next_constant = self.get_next_constant();
                    self.get_top_call_frame().push_constant(-next_constant)
                }
                Byte::Op(OpCode::Not) => {
                    let next_constant = self.get_next_constant();
                    self.get_top_call_frame().push_constant(!next_constant);
                }
                Byte::Op(OpCode::True) => self.bool(true),
                Byte::Op(OpCode::False) => self.bool(false),
                Byte::Op(OpCode::Add) => self.binary_operation(Operation::Add),
                Byte::Op(OpCode::Subtract) => {
                    self.binary_operation(Operation::Subtract)
                }
                Byte::Op(OpCode::Multiply) => {
                    self.binary_operation(Operation::Multiply)
                }
                Byte::Op(OpCode::Divide) => {
                    self.binary_operation(Operation::Divide)
                }
                Byte::Op(OpCode::Print) => self.print(),
                Byte::Op(OpCode::Constant) => {}
                Byte::Op(OpCode::DefineVariable) => {
                    let name = self.get_next_constant();
                    let environment = self.scope_stack.pop().unwrap();
                    let new_environment = environment.update(
                        name.get_string_value().clone(),
                        self.get_top_call_frame().pop_constant().unwrap(),
                    );
                    self.scope_stack.push(new_environment);
                }
                Byte::Op(OpCode::GetVariable) => {
                    let name = self.get_next_constant();
                    let current_scope = self.scope_stack.top();
                    let value = current_scope
                        .get(name.get_string_value())
                        .unwrap()
                        .clone();
                    self.get_top_call_frame().push_constant(value);
                }
                Byte::Op(OpCode::SetVariable) => {
                    let name = self.get_next_constant();
                    let value = self.get_next_constant();
                    let current_scope = self.scope_stack.pop().unwrap();
                    let new_scope = current_scope
                        .update(name.get_string_value().clone(), value);
                    self.scope_stack.push(new_scope)
                }
                Byte::Op(OpCode::ScopeStart) => {
                    let top_stack = self.scope_stack.top().clone();
                    self.scope_stack.push(top_stack);
                }
                Byte::Op(OpCode::ScopeEnd) => {
                    self.scope_stack.pop();
                }
                Byte::Op(OpCode::Pop) => {
                    self.get_top_call_frame().pop_constant();
                }
                Byte::Op(OpCode::JumpIfFalse) => {
                    let offset = self.get_next_location(&instructions);
                    if !self.get_next_constant().is_true() {
                        self.get_top_call_frame()
                            .increment_instruction_pointer(*offset);
                    }
                }
                Byte::Op(OpCode::Jump) => {
                    let offset = self.get_next_location(&instructions);
                    self.get_top_call_frame()
                        .increment_instruction_pointer(*offset);
                }
                Byte::Op(OpCode::Loop) => {
                    let offset = self.get_next_location(&instructions);
                    self.get_top_call_frame()
                        .decrement_instruction_pointer(*offset);
                }
                Byte::Constant(index) => {
                    let constant = self
                        .compiler
                        .function
                        .get_chunk()
                        .constant_at(*index)
                        .clone();
                    self.get_top_call_frame().push_constant(constant);
                }
                byte_code => unreachable!(
                    "Encountered unexpected operation: {:?}",
                    byte_code
                ),
            };

            self.get_top_call_frame().increment_instruction_pointer(1);
        }
        Ok(Value::Bool(true))
    }

    fn get_top_call_frame(&mut self) -> &mut CallFrame {
        self.call_frame_stack.top_mut()
    }

    fn binary_operation(&mut self, operation: Operation) {
        let first = self.get_next_constant();
        let second = self.get_next_constant();
        let result = match operation {
            Operation::Add => first + second,
            Operation::Subtract => first - second,
            Operation::Multiply => first * second,
            Operation::Divide => first / second,
            Operation::Equals => Value::Bool(first == second),
            Operation::GreaterThan => Value::Bool(first > second),
            Operation::LessThan => Value::Bool(first < second),
            Operation::NotEquals => Value::Bool(first != second),
            Operation::Modulo => first % second,
        };
        self.get_top_call_frame().push_constant(result);
    }

    fn get_next_constant(&mut self) -> Value {
        match self.get_top_call_frame().pop_constant() {
            Some(x) => x,
            None => panic!("Nothing on the constants stack to pop"),
        }
    }

    fn get_next_location<'a>(&mut self, instructions: &'a [Byte]) -> &'a usize {
        let call_frame = self.get_top_call_frame();
        call_frame.increment_instruction_pointer(1);
        let offset_byte = instructions
            .get(call_frame.get_instruction_pointer())
            .unwrap();
        match offset_byte {
            Byte::Op(OpCode::OpLocation(offset)) => offset,
            _ => panic!("Unexpected byte found in if-statement expression"),
        }
    }

    fn bool(&mut self, val: bool) {
        self.get_top_call_frame().push_constant(Value::Bool(val));
    }

    fn print(&mut self) {
        println!("{}", self.get_next_constant())
    }
}
