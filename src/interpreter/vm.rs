use super::chunk::{Byte, Chunk};
use super::compile::Compiler;
use super::interpret_result::{InterpretError, RoxResult};
use super::op_code::OpCode;
use super::value::Value;
use crate::interpreter::{Operation, Stack};
use im::HashMap;
use std::ops::Add;

type Environment = HashMap<String, Value>;

#[derive(Debug)]
pub struct VM<'a> {
    pub chunk: &'a mut Chunk,
    scope_stack: Stack<Environment>,
    constant_stack: Stack<Value>,
}

impl<'vm, 'chunk> VM<'vm> {
    pub fn new(chunk: &'chunk mut Chunk) -> VM {
        let initial_scope: Environment = HashMap::new();
        let mut initial_scope_stack = Stack::new();
        initial_scope_stack.push(initial_scope);

        VM {
            chunk,
            scope_stack: initial_scope_stack,
            constant_stack: Stack::new(),
        }
    }

    pub fn interpret(&mut self, source: &str) -> RoxResult<Value> {
        let mut compiler = Compiler::new(self.chunk);
        if compiler.compile(source).is_err() {
            return InterpretError::compile_error();
        }
        let instructions = self.chunk.codes.to_vec();
        self.run(&instructions)
    }

    fn run(&mut self, instructions: &[Byte]) -> RoxResult<Value> {
        let mut instruction_pointer = 0;
        while instruction_pointer < instructions.len() {
            let instruction = instructions
                .get(instruction_pointer)
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
                    self.constant_stack.push(-next_constant)
                }
                Byte::Op(OpCode::Not) => {
                    let next_constant = self.get_next_constant();
                    self.constant_stack.push(!next_constant);
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
                        self.constant_stack.pop().unwrap(),
                    );
                    self.scope_stack.push(new_environment);
                }
                Byte::Op(OpCode::GetVariable) => {
                    let name = self.get_next_constant();
                    let current_scope = self.scope_stack.top();
                    let value =
                        current_scope.get(name.get_string_value()).unwrap();
                    self.constant_stack.push(value.clone());
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
                    self.constant_stack.pop();
                }
                Byte::Op(OpCode::JumpIfFalse) => {
                    let offset = self.get_next_location(
                        &mut instruction_pointer,
                        instructions,
                    );
                    if !self.get_next_constant().is_true() {
                        instruction_pointer += *offset;
                    }
                }
                Byte::Op(OpCode::Jump) => {
                    let offset = self.get_next_location(
                        &mut instruction_pointer,
                        instructions,
                    );
                    instruction_pointer += *offset;
                }
                Byte::Op(OpCode::Loop) => {
                    let offset = self.get_next_location(
                        &mut instruction_pointer,
                        instructions,
                    );
                    instruction_pointer -= *offset;
                }
                Byte::Constant(index) => {
                    let constant = self.chunk.constant_at(*index as usize);
                    self.constant_stack.push(constant.clone());
                }
                byte_code => unreachable!(
                    "Encountered unexpected operation: {:?}",
                    byte_code
                ),
            };

            instruction_pointer += 1;
        }
        Ok(Value::Bool(true))
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
        self.constant_stack.push(result);
    }

    fn get_next_constant(&mut self) -> Value {
        match self.constant_stack.pop() {
            Some(x) => x,
            None => panic!("Nothing on the constants stack to pop"),
        }
    }

    fn get_next_location<'a>(
        &mut self,
        instruction_pointer: &mut usize,
        instructions: &'a [Byte],
    ) -> &'a usize {
        *instruction_pointer += 1;
        let offset_byte = instructions.get(*instruction_pointer).unwrap();
        match offset_byte {
            Byte::Op(OpCode::OpLocation(offset)) => offset,
            _ => panic!("Unexpected byte found in if-statement expression"),
        }
    }

    fn bool(&mut self, val: bool) {
        self.constant_stack.push(Value::Bool(val));
    }

    fn print(&mut self) {
        println!("{}", self.get_next_constant())
    }
}
