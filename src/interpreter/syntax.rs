#[derive(Debug)]
pub enum Expression {
    Assignment(String, Box<Expression>),
    Boolean(bool),
    Identifier(String),
    Number(f64),
    Operation(Box<Expression>, Operation, Box<Expression>),
    String(String),
    ParseError,
}

#[derive(Debug)]
pub enum Operation {
    And,
    Or,

    Equals,
    NotEquals,

    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,

    GreaterThan,
    LessThan,
}

#[derive(Debug)]
pub struct Function;

#[derive(Debug)]
pub struct Field;

#[derive(Debug)]
pub enum Statement {
    Expression(Box<Expression>),
    For,
    If,
    Print(Box<Expression>),
    Return(Option<Box<Expression>>),
    While(Box<Expression>, Box<Statement>),
    Block(Vec<Box<Declaration>>),
}

#[derive(Debug)]
pub enum Declaration {
    Function(Function),
    Record(Vec<Field>),
    Statement(Box<Statement>),
    Variable(String, Box<Expression>),
}
