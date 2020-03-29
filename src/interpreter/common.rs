#[derive(Copy, Clone, Debug)]
pub enum OpCode {
    OpConstant,
    OpTrue,
    OpFalse,
    OpNil,
    OpAdd,
    OpSubtract,
    OpMultiply,
    OpDivide,
    OpNegate,
    OpReturn,
}
