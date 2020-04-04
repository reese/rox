#[derive(Copy, Clone, Debug)]
pub enum OpCode {
    OpConstant,
    OpTrue,
    OpFalse,
    OpNot,

    OpNegate,
    OpAdd,
    OpSubtract,
    OpMultiply,
    OpDivide,

    OpLessThan,
    OpGreaterThan,
    OpEqual,

    OpPrint,
    OpReturn,
}
