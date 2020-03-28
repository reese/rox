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
        return VM {
            chunk: chunk,
            ips: vec![],
            stack: vec![],
        };
    }

    pub fn interpret(&mut self, source: Vec<u8>) -> RoxResult<Value> {
        let mut compiler = Compiler::new(&source, self.chunk);
        if !compiler.compile() {
            return Err(InterpretError::compile_error());
        }
        self.ips = self.chunk.codes.to_vec();
        self.run()
    }

    fn run(&mut self) -> RoxResult<Value> {
        let mut code_index = 0;
        let mut constant_index = 0;
        while code_index < self.ips.len() {
            match &self.ips[code_index] {
                Byte::Op(OpCode::OpReturn) => {
                    return Ok(self.get_next_constant());
                }
                Byte::Op(OpCode::OpNegate) => {
                    let next_constant = self.get_next_constant();
                    self.stack.push(-next_constant)
                }
                Byte::Op(OpCode::OpAdd) => self.binary_operation("+"),
                Byte::Op(OpCode::OpSubtract) => self.binary_operation("-"),
                Byte::Op(OpCode::OpMultiply) => self.binary_operation("*"),
                Byte::Op(OpCode::OpDivide) => self.binary_operation("/"),
                Byte::Op(OpCode::OpConstant) => {
                    let constant = &self.chunk.constants.values[constant_index];
                    constant_index += 1;
                    code_index += 1;
                    &self.stack.push(constant.clone());
                }
                byte_code => unreachable!(
                    "Encountered unexpected operation: {:?}",
                    byte_code
                ),
            }
            code_index += 1;
        }
        return Err(InterpretError::compile_error());
    }

    fn binary_operation(&mut self, operation: &str) {
        let first = self.get_next_constant();
        let second = self.get_next_constant();
        let result = match operation {
            "+" => first + second,
            "-" => second - first,
            "*" => first * second,
            "/" => second / first,
            _ => panic!("Unknown binary operation attempted."),
        };
        self.stack.push(result)
    }

    fn get_next_constant(&mut self) -> Value {
        match self.stack.pop() {
            Some(x) => return x,
            None => panic!("Nothing on the constants stack to pop"),
        }
    }
}
