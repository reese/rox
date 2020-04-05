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
    Variable,
}

#[derive(Clone, Copy)]
pub struct ParseRule {
    pub prefix: ParseOp,
    pub infix: ParseOp,
    pub precedence: Precedence,
}

pub const RULES: [ParseRule; 39] = [
    ParseRule {
        prefix: ParseOp::Grouping,
        infix: ParseOp::Noop,
        precedence: Precedence::PrecedenceNone,
    }, // Left Paren
    ParseRule {
        prefix: ParseOp::Noop,
        infix: ParseOp::Noop,
        precedence: Precedence::PrecedenceNone,
    }, // Right Paren
    ParseRule {
        prefix: ParseOp::Noop,
        infix: ParseOp::Noop,
        precedence: Precedence::PrecedenceNone,
    }, // Left Brace
    ParseRule {
        prefix: ParseOp::Noop,
        infix: ParseOp::Noop,
        precedence: Precedence::PrecedenceNone,
    }, // Right Brace
    ParseRule {
        prefix: ParseOp::Noop,
        infix: ParseOp::Noop,
        precedence: Precedence::PrecedenceNone,
    }, // Comma
    ParseRule {
        prefix: ParseOp::Noop,
        infix: ParseOp::Noop,
        precedence: Precedence::PrecedenceNone,
    }, // Dot
    ParseRule {
        prefix: ParseOp::Unary,
        infix: ParseOp::Binary,
        precedence: Precedence::PrecedenceTerm,
    }, // Minus
    ParseRule {
        prefix: ParseOp::Noop,
        infix: ParseOp::Binary,
        precedence: Precedence::PrecedenceTerm,
    }, // Plus
    ParseRule {
        prefix: ParseOp::Noop,
        infix: ParseOp::Noop,
        precedence: Precedence::PrecedenceNone,
    }, // Semicolon
    ParseRule {
        prefix: ParseOp::Noop,
        infix: ParseOp::Binary,
        precedence: Precedence::PrecedenceFactor,
    }, // Slash
    ParseRule {
        prefix: ParseOp::Noop,
        infix: ParseOp::Binary,
        precedence: Precedence::PrecedenceFactor,
    }, // Star
    ParseRule {
        prefix: ParseOp::Unary,
        infix: ParseOp::Noop,
        precedence: Precedence::PrecedenceNone,
    }, // Bang
    ParseRule {
        prefix: ParseOp::Noop,
        infix: ParseOp::Binary,
        precedence: Precedence::PrecedenceEquality,
    }, // BangEqual
    ParseRule {
        prefix: ParseOp::Noop,
        infix: ParseOp::Noop,
        precedence: Precedence::PrecedenceNone,
    }, // Equal
    ParseRule {
        prefix: ParseOp::Noop,
        infix: ParseOp::Binary,
        precedence: Precedence::PrecedenceEquality,
    }, // Double Equal
    ParseRule {
        prefix: ParseOp::Noop,
        infix: ParseOp::Binary,
        precedence: Precedence::PrecedenceComparison,
    }, // Greater
    ParseRule {
        prefix: ParseOp::Noop,
        infix: ParseOp::Binary,
        precedence: Precedence::PrecedenceComparison,
    }, // Greater Equal
    ParseRule {
        prefix: ParseOp::Noop,
        infix: ParseOp::Binary,
        precedence: Precedence::PrecedenceEquality,
    }, // Less
    ParseRule {
        prefix: ParseOp::Noop,
        infix: ParseOp::Binary,
        precedence: Precedence::PrecedenceEquality,
    }, // Less Equal
    ParseRule {
        prefix: ParseOp::Variable,
        infix: ParseOp::Noop,
        precedence: Precedence::PrecedenceNone,
    }, // Identifier
    ParseRule {
        prefix: ParseOp::String,
        infix: ParseOp::Noop,
        precedence: Precedence::PrecedenceNone,
    }, // String
    ParseRule {
        prefix: ParseOp::Number,
        infix: ParseOp::Noop,
        precedence: Precedence::PrecedenceNone,
    }, // Number
    ParseRule {
        prefix: ParseOp::Noop,
        infix: ParseOp::Noop,
        precedence: Precedence::PrecedenceNone,
    }, // And
    ParseRule {
        prefix: ParseOp::Noop,
        infix: ParseOp::Noop,
        precedence: Precedence::PrecedenceNone,
    }, // Class
    ParseRule {
        prefix: ParseOp::Noop,
        infix: ParseOp::Noop,
        precedence: Precedence::PrecedenceNone,
    }, // Else
    ParseRule {
        prefix: ParseOp::Literal,
        infix: ParseOp::Noop,
        precedence: Precedence::PrecedenceNone,
    }, // False
    ParseRule {
        prefix: ParseOp::Noop,
        infix: ParseOp::Noop,
        precedence: Precedence::PrecedenceNone,
    }, // For
    ParseRule {
        prefix: ParseOp::Noop,
        infix: ParseOp::Noop,
        precedence: Precedence::PrecedenceNone,
    }, // Fn
    ParseRule {
        prefix: ParseOp::Noop,
        infix: ParseOp::Noop,
        precedence: Precedence::PrecedenceNone,
    }, // If
    ParseRule {
        prefix: ParseOp::Noop,
        infix: ParseOp::Noop,
        precedence: Precedence::PrecedenceNone,
    }, // Or
    ParseRule {
        prefix: ParseOp::Noop,
        infix: ParseOp::Noop,
        precedence: Precedence::PrecedenceNone,
    }, // Print
    ParseRule {
        prefix: ParseOp::Noop,
        infix: ParseOp::Noop,
        precedence: Precedence::PrecedenceNone,
    }, // Return
    ParseRule {
        prefix: ParseOp::Noop,
        infix: ParseOp::Noop,
        precedence: Precedence::PrecedenceNone,
    }, // Super
    ParseRule {
        prefix: ParseOp::Noop,
        infix: ParseOp::Noop,
        precedence: Precedence::PrecedenceNone,
    }, // This
    ParseRule {
        prefix: ParseOp::Literal,
        infix: ParseOp::Noop,
        precedence: Precedence::PrecedenceNone,
    }, // True
    ParseRule {
        prefix: ParseOp::Noop,
        infix: ParseOp::Noop,
        precedence: Precedence::PrecedenceNone,
    }, // Let
    ParseRule {
        prefix: ParseOp::Noop,
        infix: ParseOp::Noop,
        precedence: Precedence::PrecedenceNone,
    }, // While
    ParseRule {
        prefix: ParseOp::Noop,
        infix: ParseOp::Noop,
        precedence: Precedence::PrecedenceNone,
    }, // Error
    ParseRule {
        prefix: ParseOp::Noop,
        infix: ParseOp::Noop,
        precedence: Precedence::PrecedenceNone,
    }, // EOF
];
