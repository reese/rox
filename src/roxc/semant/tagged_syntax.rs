//! # Tagged Syntax
//! Tagged syntax is syntax such as `roxc`'s  [Expression](crate::roxc::Expression)
//! and [Statement](crate::roxc::Statement) enums wrapped with additional type information
//! necessary to tag things later in the compilation process, such as return types
//! and variable types.

use crate::roxc::semant::types::ArenaType;
use crate::roxc::{
    FunctionDeclaration, Operation, RoxType, Unary, BOOL_TYPE_VAL,
    NUMBER_TYPE_VAL, STRING_TYPE_VAL,
};

#[derive(Clone, Debug)]
pub enum TaggedExpression {
    And(Box<TaggedExpression>, Box<TaggedExpression>),
    Assignment(String, Box<TaggedExpression>),
    Boolean(bool),
    #[allow(clippy::vec_box)]
    FunctionCall(String, Vec<Box<TaggedExpression>>, RoxType),
    Identifier(String, RoxType),
    Number(f64),
    Operation(Box<TaggedExpression>, Operation, Box<TaggedExpression>),
    Or(Box<TaggedExpression>, Box<TaggedExpression>),
    String(String),
    Unary(Unary, Box<TaggedExpression>),
    Variable(String, Box<TaggedExpression>),
}

impl Into<ArenaType> for TaggedExpression {
    fn into(self) -> ArenaType {
        use TaggedExpression::*;
        match self {
            And(_, _) | Boolean(_) | Or(_, _) => BOOL_TYPE_VAL.clone(),
            Assignment(_, expression) => (*expression).into(),
            FunctionCall(_, _, rox_type) | Identifier(_, rox_type) => {
                rox_type.into()
            }
            Number(_) | Operation(_, _, _) | Unary(_, _) => {
                NUMBER_TYPE_VAL.clone()
            }
            String(_) => STRING_TYPE_VAL.clone(),
            Variable(_, expression) => (*expression).into(),
        }
    }
}

type TaggedBlock = Vec<Box<TaggedStatement>>;

#[derive(Clone, Debug)]
pub(crate) enum TaggedStatement {
    Expression(TaggedExpression),
    Return(Option<TaggedExpression>),
    IfElse(Box<TaggedExpression>, TaggedBlock, Option<TaggedBlock>),
    FunctionDeclaration(FunctionDeclaration, TaggedBlock),
}

#[derive(Debug)]
pub(crate) enum TaggedDeclaration {
    Function(TaggedStatement),
}

impl TaggedDeclaration {}
