#[derive(Debug)]
pub(crate) enum OpCode {
    Return,
    /// A literal value.
    /// The `usize` value here is the index in the VM's `constants` Vec.
    Constant(usize),
    Pop,

    /* Unary Operators */
    Negate,
    Not,

    /* Comparison Operations */
    Equal,
    Greater,
    Less,

    /* Binary Operators */
    Add,
    Subtract,
    Multiply,
    Divide,

    /* Booleans */
    True,
    False,

    /* Variables */
    ReadVariable(usize),
    AssignVariable(usize),
}
