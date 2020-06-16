//! # Tagged Syntax
//! Tagged syntax is syntax such as `roxc`'s  [Expression](crate::roxc::Expression)
//! and [Statement](crate::roxc::Statement) enums wrapped with additional type information
//! necessary to tag things later in the compilation process, such as return types
//! and variable types.

use crate::roxc::semant::types::ArenaType;
use crate::roxc::{Block, Expression, FunctionDeclaration, Param, RoxType};

#[derive(Clone, Debug)]
pub(crate) struct TaggedExpression {
    expression: Box<Expression>,
    pub rox_type: RoxType,
}

impl TaggedExpression {
    pub fn new(expression: Box<Expression>, rox_type: RoxType) -> Self {
        TaggedExpression {
            expression,
            rox_type,
        }
    }
}

impl Into<ArenaType> for TaggedExpression {
    fn into(self) -> usize {
        self.rox_type.into()
    }
}

type TaggedBlock = Vec<Box<TaggedStatement>>;

#[derive(Clone, Debug)]
pub(crate) enum TaggedStatement {
    Expression(TaggedExpression),
    Return(Option<TaggedExpression>),
    Block(Block),
    IfElse(Box<TaggedExpression>, TaggedBlock, Option<TaggedBlock>),
    FunctionDeclaration(FunctionDeclaration, TaggedBlock),
}

#[derive(Debug)]
pub(crate) enum TaggedDeclaration {
    Function(TaggedStatement),
}

impl TaggedDeclaration {}
