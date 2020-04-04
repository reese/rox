use super::chunk::{Byte, Chunk};
use super::common::OpCode;
use super::compile::Compiler;
use super::interpret_result::{InterpretError, RoxResult};
use super::value::Value;

#[derive(Debug)]
pub struct VM<'a> {
    pub chunk: &'a mut Chunk,
    ips: Vec<Byte>,
    stack: Vec<Value>,
}

impl<'vm, 'chunk> VM<'vm> {
    pub fn new(chunk: &'chunk mut Chunk) -> VM {
        VM {
            chunk,
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
        let mut constant_index = 0;
        self.ips
            .clone()
            .iter()
            .enumerate()
            .for_each(|(index, instruction)| match instruction {
                Byte::Op(OpCode::OpReturn) => {
                    result = Some(Ok(Value::Bool(true)));
                }
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
                Byte::Op(OpCode::OpAdd) => self.binary_operation("+"),
                Byte::Op(OpCode::OpSubtract) => self.binary_operation("-"),
                Byte::Op(OpCode::OpMultiply) => self.binary_operation("*"),
                Byte::Op(OpCode::OpDivide) => self.binary_operation("/"),
                Byte::Op(OpCode::OpPrint) => self.print(),
                Byte::Op(OpCode::OpConstant) => {
                    let constant = self.chunk.constant_at(constant_index);
                    constant_index += 1;
                    self.stack.push(constant.clone());
                }
                Byte::Constant(x) => {}
                byte_code => unreachable!(
                    "Encountered unexpected operation: {:?}",
                    byte_code
                ),
            });
        result.unwrap_or(InterpretError::compile_error())
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
