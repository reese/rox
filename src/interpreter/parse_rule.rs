use super::precedence::Precedence;

pub enum ParseOp {
    Number,
    Grouping,
    Unary,
    Binary,
    Noop,
}

pub struct ParseRule {
    pub prefix: ParseOp,
    pub infix: ParseOp,
    pub precedence: Precedence,
}
