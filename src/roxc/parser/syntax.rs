#![allow(clippy::vec_box)]
use crate::roxc::semant;

#[derive(Clone, Debug)]
pub struct Span(pub usize, pub usize);

#[derive(Clone, Debug)]
pub struct Spanned<T> {
    pub value: T,
    pub span: Span,
}

impl<T> Spanned<T> {
    pub fn dummy_span(value: T) -> Self {
        Spanned {
            value,
            span: Span(0, 0),
        }
    }
}

#[derive(Clone, Debug)]
pub enum Expression {
    Access(Box<Expression>, Box<Expression>),
    And(Box<Expression>, Box<Expression>),
    Array(Vec<Box<Expression>>),
    Assignment(Box<Expression>, Box<Expression>),
    Boolean(bool),
    FunctionCall(
        Spanned<Identifier>,
        Vec<Box<TypeName>>,
        Vec<Box<Expression>>,
    ),
    Identifier(Spanned<Identifier>),
    Float(Spanned<f64>),
    Int(Spanned<i32>),
    Operation(Box<Expression>, Spanned<Operation>, Box<Expression>),
    Or(Box<Expression>, Box<Expression>),
    String(Spanned<String>),
    StructInstantiation(
        Spanned<Identifier>,
        Option<Vec<Box<TypeName>>>,
        Vec<(Identifier, Box<Expression>)>,
    ),
    Unary(Unary, Box<Expression>),
    Variable(Spanned<Identifier>, Box<Expression>),
    ParseError,
}

#[derive(Clone, Debug)]
pub enum TypeName {
    Type(Spanned<Identifier>),
    GenericType(Spanned<Identifier>, Vec<Box<TypeName>>),
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
