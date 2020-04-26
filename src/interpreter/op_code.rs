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
}
