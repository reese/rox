#[derive(Copy, Clone, Debug)]
pub enum OpCode {
    OpConstant,
    OpTrue,
    OpFalse,
    OpNot,
    OpAdd,
    OpSubtract,
    OpMultiply,
    OpDivide,
    OpNegate,
    OpReturn,
}
