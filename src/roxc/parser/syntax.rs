use crate::roxc::semant;
use crate::roxc::{Type, TypeConstructor};
use cranelift::prelude::types;

#[derive(Clone, Debug)]
#[allow(clippy::vec_box)]
pub enum Expression {
    Access(Box<Expression>, Box<Expression>),
    And(Box<Expression>, Box<Expression>),
    Array(Vec<Box<Expression>>),
    Assignment(Box<Expression>, Box<Expression>),
    Boolean(bool),
    FunctionCall(Identifier, Vec<Identifier>, Vec<Box<Expression>>),
    Identifier(Identifier),
    Number(f64),
    Operation(Box<Expression>, Operation, Box<Expression>),
    Or(Box<Expression>, Box<Expression>),
    String(String),
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
pub type Param = (Identifier, TypeName);
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

pub fn get_cranelift_type(
    type_: &semant::Type,
    pointer_type: types::Type,
) -> types::Type {
    match type_ {
        Type::Variable(_) => {
            panic!("Cannot get cranelift type for type variable")
        }
        Type::Apply(constructor, types) => {
            use TypeConstructor::*;
            match constructor {
                Bool => types::B1,
                Number => types::F64,
                String => pointer_type,
                Void => types::INVALID,
                Arrow => get_cranelift_type(
                    types.iter().last().unwrap(),
                    pointer_type,
                ),
                Array => pointer_type,
                Record(_) => unimplemented!("Implement record type"),
                FunctionType(_, _) => pointer_type,
                Unique(_) => unimplemented!("Implement unique type"),
            }
        }
        Type::PolymorphicType(_, _type_) => {
            unimplemented!("Implement polymorphic type")
        }
    }
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
    Block(Block),
    IfElse(Box<Expression>, Block, Option<Block>),
    FunctionDeclaration(
        Identifier,
        Option<Vec<Identifier>>,
        Vec<Param>,
        Box<TypeName>,
        Block,
    ),
}

#[derive(Clone, Debug)]
/// Declarations are top-level statements that define a function or data type.
/// As of now, declarations cannot happen inside other declarations, i.e.
/// you cannot define a function inside of a function.
/// This should be changed in future versions of Rox.
pub enum Declaration {
    // TODO: Allow user defined types
    // Record(Vec<Field>),
    Function(Box<Statement>),
}
