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

    pub fn interpret(&mut self, source: Vec<u8>) -> RoxResult<Value> {
        let mut compiler = Compiler::new(&source, self.chunk);
        if !compiler.compile() {
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
            .enumerate()
            .for_each(|(index, instruction)| match instruction {
                Byte::Op(OpCode::OpReturn) => {}
                Byte::Op(OpCode::OpEqual) => self.binary_operation("="),
                Byte::Op(OpCode::OpGreaterThan) => self.binary_operation(">"),
                Byte::Op(OpCode::OpLessThan) => self.binary_operation("<"),
                Byte::Op(OpCode::OpNegate) => {
                    if !self.peek(0).is_number() {
                        result = Some(self.runtime_error(
                            index,
                            "Could not negate non-number type",
                        ))
                    } else {
                        let next_constant = self.get_next_constant();
                        self.stack.push(-next_constant)
                    }
                }
                Byte::Op(OpCode::OpNot) => {
                    if !self.peek(0).is_bool() {
                        result = Some(self.runtime_error(
                            index,
                            "Could not negate non-bool type.",
                        ))
                    } else {
                        let next_constant = self.get_next_constant();
                        self.stack.push(!next_constant);
                    }
                }
                Byte::Op(OpCode::OpTrue) => self.bool(true),
                Byte::Op(OpCode::OpFalse) => self.bool(false),
                Byte::Op(OpCode::OpAdd) => self.binary_operation("+"), // TODO: All of these should be in some kind of enum
                Byte::Op(OpCode::OpSubtract) => self.binary_operation("-"),
                Byte::Op(OpCode::OpMultiply) => self.binary_operation("*"),
                Byte::Op(OpCode::OpDivide) => self.binary_operation("/"),
                Byte::Op(OpCode::OpPrint) => self.print(),
                Byte::Op(OpCode::OpConstant) => {}
                Byte::Op(OpCode::OpDefineGlobal) => {
                    let name = self.get_next_constant();
                    self.globals.insert(
                        name.get_string_value().clone(),
                        self.peek(0).clone(),
                    );
                    self.stack.pop();
                }
                Byte::Op(OpCode::OpGetGlobal) => {
                    let name = self.get_next_constant();
                    match self.globals.get(name.get_string_value()) {
                        Some(value) => self.stack.push(value.clone()),
                        None => {
                            result = Some(
                                self.runtime_error(
                                    index,
                                    format!("Undefined variable: {}", name)
                                        .as_ref(),
                                ),
                            );
                        }
                    }
                }
                Byte::Op(OpCode::OpPop) => {
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

    fn runtime_error<T>(&self, ip_index: usize, message: &str) -> RoxResult<T> {
        println!(
            "{}",
            format!(
                "Your line number might be {}",
                self.chunk.lines[ip_index] - 1
            )
        );
        InterpretError::runtime_error(message)
    }
}
