//! # Tagged Syntax
//! Tagged syntax is syntax such as `roxc`'s  [Expression](crate::roxc::Expression)
//! and [Statement](crate::roxc::Statement) enums wrapped with additional type information
//! necessary to tag things later in the compilation process, such as return types
//! and variable types.

use crate::roxc::parser;
use crate::roxc::semant::types::{Type, TypeConstructor};
use crate::roxc::{semant, FunctionDeclaration, Identifier, Operation, Unary};
use parser::Spanned;

#[derive(Clone, Debug)]
#[allow(clippy::vec_box, dead_code)]
pub enum TaggedExpression {
    Access(Box<TaggedExpression>, Box<TaggedExpression>, Box<Type>),
    And(Box<TaggedExpression>, Box<TaggedExpression>),
    Array(Vec<TaggedExpression>, Box<Type>),
    Assignment(Box<TaggedExpression>, Box<TaggedExpression>, Box<Type>),
    Boolean(bool),
    FunctionCall(Spanned<Identifier>, Vec<TaggedExpression>, Box<Type>),
    Identifier(Spanned<Identifier>, Box<Type>),
    Float(f64),
    Int(i32),
    Operation(
        Box<TaggedExpression>,
        Operation,
        Box<TaggedExpression>,
        Box<Type>,
    ),
    Or(Box<TaggedExpression>, Box<TaggedExpression>),
    String(Spanned<String>),
    StructInstantiation(Box<Type>, Vec<(Identifier, Box<TaggedExpression>)>),
    Unary(Unary, Box<TaggedExpression>, Box<Type>),
    Variable(Identifier, Box<TaggedExpression>, Box<Type>),
}

// This sucks - @reese
impl Into<semant::Type> for TaggedExpression {
    fn into(self) -> semant::Type {
        (&self).into()
    }
}

impl Into<semant::Type> for &TaggedExpression {
    fn into(self) -> semant::Type {
        use TaggedExpression::*;
        match self {
            Access(_, _, t)
            | Array(_, t)
            | Assignment(_, _, t)
            | FunctionCall(_, _, t)
            | StructInstantiation(t, _)
            | Identifier(_, t) => t.as_ref().clone(),
            And(_, _) | Boolean(_) => {
                Type::Apply(TypeConstructor::Bool, Vec::new())
            }
            Float(_) => Type::Apply(TypeConstructor::Float, Vec::new()),
            Int(_) => Type::Apply(TypeConstructor::Int, Vec::new()),
            String(_) => Type::Apply(TypeConstructor::String, Vec::new()),
            Operation(_, operation, _, _) => {
                use parser::Operation::*;
                match operation {
                    Equals | NotEquals | GreaterThan | LessThan => {
                        Type::Apply(TypeConstructor::Bool, Vec::new())
                    }
                    Add | Subtract | Multiply | Divide => {
                        Type::Apply(TypeConstructor::Float, Vec::new())
                    }
                }
            }
            x => todo!("{:?}", x),
        }
    }
}

type TaggedBlock = Vec<TaggedStatement>;

#[derive(Clone, Debug)]
pub(crate) enum TaggedStatement {
    Expression(TaggedExpression),
    ExternFunctionDeclaration(FunctionDeclaration),
    FunctionDeclaration(FunctionDeclaration, TaggedBlock),
    StructDeclaration,
    IfElse(Box<TaggedExpression>, TaggedBlock, Option<TaggedBlock>),
    Return(Option<TaggedExpression>),
}
