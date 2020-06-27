//! # Tagged Syntax
//! Tagged syntax is syntax such as `roxc`'s  [Expression](crate::roxc::Expression)
//! and [Statement](crate::roxc::Statement) enums wrapped with additional type information
//! necessary to tag things later in the compilation process, such as return types
//! and variable types.

use crate::roxc::semant::types::ArenaType;
use crate::roxc::{
    syntax, FunctionDeclaration, Identifier, Operation, RoxType, Unary,
    BOOL_TYPE_VAL, NUMBER_TYPE_VAL, STRING_TYPE_VAL,
};

#[derive(Clone, Debug)]
#[allow(clippy::vec_box)]
pub enum TaggedExpression {
    And(Box<TaggedExpression>, Box<TaggedExpression>),
    Array(Vec<Box<TaggedExpression>>),
    Assignment(Identifier, Box<TaggedExpression>),
    Boolean(bool),
    #[allow(clippy::vec_box)]
    FunctionCall(Identifier, Vec<Box<TaggedExpression>>, RoxType),
    Identifier(Identifier, RoxType),
    Number(f64),
    Operation(Box<TaggedExpression>, Operation, Box<TaggedExpression>),
    Or(Box<TaggedExpression>, Box<TaggedExpression>),
    String(String),
    Unary(Unary, Box<TaggedExpression>),
    Variable(Identifier, Box<TaggedExpression>),
}

impl Into<ArenaType> for TaggedExpression {
    fn into(self) -> ArenaType {
        use TaggedExpression::*;
        match self {
            And(_, _) | Boolean(_) | Or(_, _) => BOOL_TYPE_VAL,
            Assignment(_, expression) => (*expression).into(),
            FunctionCall(_, _, rox_type) | Identifier(_, rox_type) => {
                rox_type.into()
            }
            Number(_) | Unary(_, _) => NUMBER_TYPE_VAL,
            Operation(_, operation, _) => match operation {
                syntax::Operation::Equals
                | syntax::Operation::NotEquals
                | syntax::Operation::GreaterThan
                | syntax::Operation::LessThan => BOOL_TYPE_VAL,
                syntax::Operation::Add
                | syntax::Operation::Subtract
                | syntax::Operation::Multiply
                | syntax::Operation::Divide => NUMBER_TYPE_VAL,
            },
            String(_) => STRING_TYPE_VAL,
            Variable(_, expression) => (*expression).into(),
            Array(_) => todo!(),
        }
    }
}

type TaggedBlock = Vec<Box<TaggedStatement>>;

#[derive(Clone, Debug)]
pub(crate) enum TaggedStatement {
    Block(TaggedBlock),
    Expression(TaggedExpression),
    Return(Option<TaggedExpression>),
    IfElse(Box<TaggedExpression>, TaggedBlock, Option<TaggedBlock>),
    FunctionDeclaration(FunctionDeclaration, TaggedBlock),
}

#[derive(Debug)]
pub(crate) enum TaggedDeclaration {
    Function(TaggedStatement),
}
