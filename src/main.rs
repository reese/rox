mod lib;
use lib::*;

fn main() {
    let mut chunk = Chunk::new();
    chunk.push_line(OpCode::OpConstant, 123);
    chunk.push_line(Value { f: 1.2 }, 123);
    chunk.push_line(OpCode::OpConstant, 7);
    chunk.push_line(Value { f: 78 as f64 }, 7);
    chunk.push_line(OpCode::OpNegate, 123);
    chunk.push_line(OpCode::OpConstant, 7);
    chunk.push_line(Value { f: 4 as f64 }, 7);
    chunk.push_line(OpCode::OpConstant, 7);
    chunk.push_line(Value { f: 5 as f64 }, 7);
    chunk.push_line(OpCode::OpSubtract, 7);
    chunk.push_line(OpCode::OpReturn, 7);
    let mut vm = VM::new(chunk);
    vm.interpret();
    println!("{:?}", vm)
}
