#[derive(Copy, Clone, Debug)]
pub enum OpCode {
    And,
    Or,

    Equals,
    NotEquals,

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

    DefineVariable,
    GetVariable,
    SetVariable,

    ScopeStart,
    ScopeEnd,

    Print,
    Return,

    JumpIfFalse,
    Jump,
    Placeholder, // This is a placeholder op that's replaced after a jump point is parsed
    OpLocation(usize),
    Loop,
}
