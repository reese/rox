use super::chunk::{Byte, Chunk};
use super::compile::Compiler;
use super::interpret_result::{InterpretError, RoxResult};
use super::op_code::OpCode;
use super::value::Value;
use crate::interpreter::{Operation, Stack};
use im::HashMap;

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
        let ips = self.chunk.codes.to_vec();
        self.run(&ips)
    }

    fn run(&mut self, instructions: &[Byte]) -> RoxResult<Value> {
        let mut result = None;
        instructions
            .iter()
            .for_each(|instruction| match instruction {
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
                    if !self.peek_next().is_number() {
                        result =
                            Some(self.runtime_error(
                                "Could not negate non-number type",
                            ))
                    } else {
                        let next_constant = self.get_next_constant();
                        self.constant_stack.push(-next_constant)
                    }
                }
                Byte::Op(OpCode::Not) => {
                    if !self.peek_next().is_bool() {
                        result =
                            Some(self.runtime_error(
                                "Could not negate non-bool type.",
                            ))
                    } else {
                        let next_constant = self.get_next_constant();
                        self.constant_stack.push(!next_constant);
                    }
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
                    match current_scope.get(name.get_string_value()) {
                        Some(value) => self.constant_stack.push(value.clone()),
                        None => {
                            result = Some(
                                self.runtime_error(
                                    format!("Undefined variable: {}", name)
                                        .as_ref(),
                                ),
                            );
                        }
                    }
                }
                Byte::Op(OpCode::SetVariable) => {
                    let name = self.get_next_constant();
                    let value = self.get_next_constant();
                    let current_scope = self.scope_stack.pop().unwrap();
                    let new_scope = current_scope
                        .update(name.get_string_value().clone(), value);
                    self.scope_stack.push(new_scope)
                }
                Byte::Op(OpCode::Pop) => {
                    self.constant_stack.pop();
                }
                Byte::Constant(index) => {
                    let constant = self.chunk.constant_at(*index as usize);
                    self.constant_stack.push(constant.clone());
                }
                byte_code => unreachable!(
                    "Encountered unexpected operation: {:?}",
                    byte_code
                ),
            });

        match result {
            Some(result) => result,
            None => Ok(Value::Bool(true)),
        }
    }

    fn binary_operation(&mut self, operation: Operation) {
        let first = self.get_next_constant();
        let second = self.get_next_constant();
        let result = match operation {
            Operation::Add => second.add(first),
            Operation::Subtract => second.subtract(first),
            Operation::Multiply => second.multiply(first),
            Operation::Divide => second.divide(first),
            Operation::Equals => second.equals(first),
            Operation::GreaterThan => second.greater_than(first),
            Operation::LessThan => second.less_than(first),
            _ => panic!("Unknown binary operation attempted."),
        };
        match result {
            Ok(val) => self.constant_stack.push(val),
            Err(error) => panic!(error.message),
        }
    }

    fn get_next_constant(&mut self) -> Value {
        match self.constant_stack.pop() {
            Some(x) => x,
            None => panic!("Nothing on the constants stack to pop"),
        }
    }

    fn bool(&mut self, val: bool) {
        self.constant_stack.push(Value::Bool(val));
    }

    fn peek_next(&self) -> &Value {
        self.constant_stack.top()
    }

    fn print(&mut self) {
        println!("{}", self.get_next_constant())
    }

    fn runtime_error<T>(&self, message: &str) -> RoxResult<T> {
        InterpretError::runtime_error(message)
    }
}
