use crate::roxc::{semant, ArenaType};

#[derive(Clone, Debug)]
#[allow(clippy::vec_box)]
pub enum Expression {
    And(Box<Expression>, Box<Expression>),
    Array(Vec<Box<Expression>>),
    Assignment(String, Box<Expression>),
    Boolean(bool),
    FunctionCall(String, Vec<Box<Expression>>),
    Identifier(String),
    Number(f64),
    Operation(Box<Expression>, Operation, Box<Expression>),
    Or(Box<Expression>, Box<Expression>),
    String(String),
    Unary(Unary, Box<Expression>),
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

#[derive(Clone, Debug)]
pub enum RoxType {
    Void,
    Bool,
    Number,
    String,
    // TODO: Support user-defined types
    // UserType(String),
}

impl From<ArenaType> for RoxType {
    fn from(arena_type: ArenaType) -> Self {
        match arena_type {
            semant::NUMBER_TYPE_VAL => RoxType::Number,
            semant::BOOL_TYPE_VAL => RoxType::Bool,
            semant::STRING_TYPE_VAL => RoxType::String,
            _ => panic!("Rox does not yet support user-defined types"),
        }
    }
}

impl Into<ArenaType> for RoxType {
    fn into(self) -> usize {
        match self {
            RoxType::Void => semant::VOID_TYPE_VAL,
            RoxType::Number => semant::NUMBER_TYPE_VAL,
            RoxType::Bool => semant::BOOL_TYPE_VAL,
            RoxType::String => semant::STRING_TYPE_VAL,
        }
    }
}

#[derive(Clone, Debug)]
pub struct FunctionDeclaration {
    pub name: String,
    pub params: Vec<Param>,
    pub return_type: Option<String>,
}

#[derive(Clone, Debug)]
pub enum Statement {
    Expression(Box<Expression>),
    Return(Option<Box<Expression>>),
    Block(Block),
    IfElse(Box<Expression>, Block, Option<Block>),
    FunctionDeclaration(String, Vec<Param>, Option<String>, Block),
}

#[derive(Debug)]
/// Declarations are top-level statements that define a function or data type.
/// As of now, declarations cannot happen inside other declarations, i.e.
/// you cannot define a function inside of a function.
/// This should be changed in future versions of Rox.
pub enum Declaration {
    // TODO: Allow user defined types
    // Record(Vec<Field>),
    Function(Box<Statement>),
}
