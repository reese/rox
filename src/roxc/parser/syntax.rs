#![allow(clippy::vec_box)]
use crate::roxc::semant;

#[derive(Clone, Debug)]
pub enum Expression {
    Access(Box<Expression>, Box<Expression>),
    And(Box<Expression>, Box<Expression>),
    Array(Vec<Box<Expression>>),
    Assignment(Box<Expression>, Box<Expression>),
    Boolean(bool),
    FunctionCall(Identifier, Vec<Box<TypeName>>, Vec<Box<Expression>>),
    Identifier(Identifier),
    Float(f64),
    Int(i32),
    Operation(Box<Expression>, Operation, Box<Expression>),
    Or(Box<Expression>, Box<Expression>),
    String(String),
    StructInstantiation(
        Identifier,
        Option<Vec<Box<TypeName>>>,
        Vec<(Identifier, Box<Expression>)>,
    ),
    Unary(Unary, Box<Expression>),
    Variable(Identifier, Box<Expression>),
    ParseError,
}

#[derive(Clone, Debug)]
pub enum TypeName {
    Type(Identifier),
    GenericType(Identifier, Vec<Box<TypeName>>),
    Function(Vec<Box<TypeName>>, Box<TypeName>),
}

pub type Block = Vec<Box<Statement>>;
pub type Param = (Identifier, Box<TypeName>);
pub type Identifier = String;

#[derive(Clone, Debug)]
pub enum Operation {
    // TODO: >=, <=
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

#[derive(Clone, Debug)]
pub struct FunctionDeclaration {
    pub name: Identifier,
    pub params: Vec<(Identifier, semant::Type)>,
    pub return_type: semant::Type,
}

#[derive(Clone, Debug)]
pub enum Statement {
    Expression(Box<Expression>),
    Return(Option<Box<Expression>>),
    IfElse(Box<Expression>, Block, Option<Block>),
    ExternFunctionDeclaration(
        Identifier,
        Vec<Box<TypeName>>,
        Option<Box<TypeName>>,
    ),
    FunctionDeclaration(
        Identifier,
        Option<Vec<Identifier>>,
        Vec<Param>,
        Option<Box<TypeName>>,
        Block,
    ),
    StructDeclaration(Identifier, Option<Vec<Identifier>>, Vec<Param>),
}
