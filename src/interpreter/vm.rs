use super::chunk::{Byte, Chunk};
use super::compile::Compiler;
use super::interpret_result::{InterpretError, RoxResult};
use super::op_code::OpCode;
use super::value::Value;
use std::collections::HashMap;

#[derive(Debug)]
pub struct VM<'a> {
    pub chunk: &'a mut Chunk,
    globals: HashMap<String, Value>,
    ips: Vec<Byte>,
    stack: Vec<Value>,
}

impl<'vm, 'chunk> VM<'vm> {
    pub fn new(chunk: &'chunk mut Chunk) -> VM {
        VM {
            chunk,
            globals: HashMap::new(),
            ips: vec![],
            stack: vec![],
        }
    }

    pub fn interpret(&mut self, source: &String) -> RoxResult<Value> {
        let mut compiler = Compiler::new(self.chunk);
        if compiler.compile(source).is_err() {
            return InterpretError::compile_error();
        }
        self.ips = self.chunk.codes.to_vec();
        self.run()
    }

    fn run(&mut self) -> RoxResult<Value> {
        let mut result = None;
        self.ips
            .clone()
            .iter()
            .for_each(|instruction| match instruction {
                Byte::Op(OpCode::Return) => {}
                Byte::Op(OpCode::Equal) => self.binary_operation("="),
                Byte::Op(OpCode::GreaterThan) => self.binary_operation(">"),
                Byte::Op(OpCode::LessThan) => self.binary_operation("<"),
                Byte::Op(OpCode::Negate) => {
                    if !self.peek(0).is_number() {
                        result = Some(self.runtime_error(
                            "Could not negate non-number type",
                        ))
                    } else {
                        let next_constant = self.get_next_constant();
                        self.stack.push(-next_constant)
                    }
                }
                Byte::Op(OpCode::Not) => {
                    if !self.peek(0).is_bool() {
                        result = Some(self.runtime_error(
                            "Could not negate non-bool type.",
                        ))
                    } else {
                        let next_constant = self.get_next_constant();
                        self.stack.push(!next_constant);
                    }
                }
                Byte::Op(OpCode::True) => self.bool(true),
                Byte::Op(OpCode::False) => self.bool(false),
                Byte::Op(OpCode::Add) => self.binary_operation("+"), // TODO: All of these should be in some kind of enum
                Byte::Op(OpCode::Subtract) => self.binary_operation("-"),
                Byte::Op(OpCode::Multiply) => self.binary_operation("*"),
                Byte::Op(OpCode::Divide) => self.binary_operation("/"),
                Byte::Op(OpCode::Print) => self.print(),
                Byte::Op(OpCode::Constant) => {}
                Byte::Op(OpCode::DefineGlobal) => {
                    let name = self.get_next_constant();
                    self.globals.insert(
                        name.get_string_value().clone(),
                        self.peek(0).clone(),
                    );
                    self.stack.pop();
                }
                Byte::Op(OpCode::GetGlobal) => {
                    let name = self.get_next_constant();
                    match self.globals.get(name.get_string_value()) {
                        Some(value) => self.stack.push(value.clone()),
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
                Byte::Op(OpCode::Pop) => {
                    self.stack.pop();
                }
                Byte::Constant(index) => {
                    let constant = self.chunk.constant_at(*index as usize);
                    self.stack.push(constant.clone());
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

    fn binary_operation(&mut self, operation: &str) {
        let first = self.get_next_constant();
        let second = self.get_next_constant();
        let result = match operation {
            "+" => second.add(first),
            "-" => second.subtract(first),
            "*" => second.multiply(first),
            "/" => second.divide(first),
            "=" => second.equals(first),
            ">" => second.greater_than(first),
            "<" => second.less_than(first),
            _ => panic!("Unknown binary operation attempted."),
        };
        match result {
            Ok(val) => self.stack.push(val),
            Err(error) => panic!(error.message),
        }
    }

    fn get_next_constant(&mut self) -> Value {
        match self.stack.pop() {
            Some(x) => x,
            None => panic!("Nothing on the constants stack to pop"),
        }
    }

    fn bool(&mut self, val: bool) {
        self.stack.push(Value::Bool(val));
    }

    fn peek(&self, distance: usize) -> &Value {
        self.stack
            .get(distance)
            .expect("Could not peek into stack.")
    }

    fn print(&mut self) {
        println!("{}", self.get_next_constant())
    }

    fn runtime_error<T>(&self, message: &str) -> RoxResult<T> {
        InterpretError::runtime_error(message)
    }
}
