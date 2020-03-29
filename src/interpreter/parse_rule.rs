use super::precedence::Precedence;

#[derive(Clone, Copy, Debug)]
pub enum ParseOp {
    Literal,
    Number,
    Grouping,
    Unary,
    Binary,
    Noop,
}

#[derive(Clone, Copy)]
pub struct ParseRule {
    pub prefix: ParseOp,
    pub infix: ParseOp,
    pub precedence: Precedence,
}
