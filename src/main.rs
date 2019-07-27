mod lib;
use lib::*;

fn main() {
    let mut chunk = Chunk::new();
    chunk.push_line(OpCode::OpConstant, 123);
    chunk.push_line(Value { f: 1.2 }, 123);
    chunk.push_line(OpCode::OpConstant, 7);
    chunk.push_line(Value { f: 78 as f64 }, 7);
    chunk.push_line(OpCode::OpReturn, 123);
    let mut vm = VM::new(chunk);
    let result = vm.interpret();
    println!("{:?}", vm)
}
