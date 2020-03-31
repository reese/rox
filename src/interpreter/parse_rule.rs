use super::precedence::Precedence;

#[derive(Clone, Copy, Debug)]
pub enum ParseOp {
    Binary,
    Grouping,
    Literal,
    Noop,
    Number,
    String,
    Unary,
}

#[derive(Clone, Copy)]
pub struct ParseRule {
    pub prefix: ParseOp,
    pub infix: ParseOp,
    pub precedence: Precedence,
}
