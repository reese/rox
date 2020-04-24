#[derive(Copy, Clone, Debug)]
pub enum OpCode {
    Constant,
    True,
    False,
    Not,

    Negate,
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,

    LessThan,
    GreaterThan,
    Equal,

    Pop,

    DefineGlobal,
    GetGlobal,

    Print,
    Return,
}
