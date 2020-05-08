#[derive(Debug)]
pub enum Expression {
    And(Box<Expression>, Box<Expression>),
    Assignment(String, Box<Expression>),
    Boolean(bool),
    FunctionCall(String, Option<Vec<Box<Expression>>>),
    Identifier(String),
    Number(f64),
    Operation(Box<Expression>, Operation, Box<Expression>),
    Or(Box<Expression>, Box<Expression>),
    String(String),
    Unary(Unary, Box<Expression>),
    ParseError,
}

#[derive(Debug)]
pub enum Operation {
    Equals,
    NotEquals,
    GreaterThan,
    LessThan,

    // TODO: Add support for +=, -=, /=, and *=
    Add,
    Subtract,
    Multiply,
    Divide,
}

#[derive(Debug)]
pub enum Unary {
    Not,
    Negate,
}

#[derive(Debug)]
pub struct Function;

#[derive(Debug)]
pub struct Field;

pub type Block = Vec<Box<Statement>>;

#[derive(Debug)]
pub enum Statement {
    Expression(Box<Expression>),
    IfElse(Box<Expression>, Block, Option<Block>),
    Print(Box<Expression>),
    Return(Option<Box<Expression>>),
    While(Box<Expression>, Block),
    Block(Block),
    Variable(String, Box<Expression>),
}

#[derive(Debug)]
pub enum Declaration {
    // Record(Vec<Field>),
    // Statement(Box<Statement>),
    Function(String, Vec<String>, Block),
    // ,
}
