#[derive(Clone, Debug)]
pub enum Expression {
    And(Box<Expression>, Box<Expression>),
    Assignment(String, Box<Expression>),
    Boolean(bool),
    FunctionCall(String, Vec<Box<Expression>>),
    Identifier(String),
    Number(f64),
    Operation(Box<Expression>, Operation, Box<Expression>),
    Or(Box<Expression>, Box<Expression>),
    String(String),
    Unary(Unary, Box<Expression>),
    Print(Box<Expression>),
    IfElse(Box<Expression>, Block, Option<Block>),
    Variable(String, Box<Expression>),
    ParseError,
}

pub type Block = Vec<Box<Statement>>;
pub type Param = (String, String);

#[derive(Clone, Debug)]
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

#[derive(Clone, Debug)]
pub enum Unary {
    Not,
    Negate,
}

#[derive(Debug)]
pub enum RoxType {
    Int,
    Float,
    String,
    // TODO: Support user-defined types
    // UserType(String),
}

#[derive(Clone, Debug)]
pub enum Statement {
    Expression(Box<Expression>),
    Return(Option<Box<Expression>>),
    Block(Block),
    FunctionDeclaration(String, Vec<Param>, Option<String>, Block),
}

#[derive(Debug)]
pub enum Declaration {
    // Record(Vec<Field>),
    Function(Box<Statement>),
}
