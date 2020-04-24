#[derive(Debug)]
pub enum Expression {
    Boolean(bool),
    Identifier(String),
    Number(f64),
    Operation(Box<Expression>, Operation, Box<Expression>),
    String(String),
    ParseError
}

#[derive(Debug)]
pub enum Operation {
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo
}

#[derive(Debug)]
pub struct Function;

#[derive(Debug)]
pub enum Statement {
    Expression(Box<Expression>),
    For,
    If,
    Print(Box<Expression>),
    Return(Option<Box<Expression>>),
    While(Box<Expression>, Box<Statement>),
    Block(Box<Declaration>)
}

#[derive(Debug)]
pub enum Declaration {
    Class(String, Option<String>),
    Variable(String, Box<Expression>),
    Function(Function),
    Statement(Box<Statement>),
}
