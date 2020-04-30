#[derive(Debug)]
pub enum Expression {
    And(Box<Expression>, Box<Expression>),
    Assignment(String, Box<Expression>),
    Boolean(bool),
    Identifier(String),
    Number(f64),
    Operation(Box<Expression>, Operation, Box<Expression>),
    Or(Box<Expression>, Box<Expression>),
    String(String),
    ParseError,
}

#[derive(Debug)]
pub enum Operation {
    Equals,
    NotEquals,
    GreaterThan,
    LessThan,

    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
}

#[derive(Debug)]
pub struct Function;

#[derive(Debug)]
pub struct Field;

pub type Block = Vec<Box<Declaration>>;

#[derive(Debug)]
pub enum Statement {
    Expression(Box<Expression>),
    IfElse(Box<Expression>, Block, Option<Block>),
    Print(Box<Expression>),
    Return(Option<Box<Expression>>),
    While(Box<Expression>, Block),
    Block(Block),
}

#[derive(Debug)]
pub enum Declaration {
    Function(Function),
    Record(Vec<Field>),
    Statement(Box<Statement>),
    Variable(String, Box<Expression>),
}
