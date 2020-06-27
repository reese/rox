use crate::roxc::{semant, ArenaType};
use cranelift::prelude::{types, Type};
use std::hash::{Hash, Hasher};

#[derive(Clone, Debug)]
#[allow(clippy::vec_box)]
pub enum Expression {
    And(Box<Expression>, Box<Expression>),
    Array(Vec<Box<Expression>>),
    Assignment(Identifier, Box<Expression>),
    Boolean(bool),
    FunctionCall(Identifier, Vec<Box<Expression>>),
    Identifier(Identifier),
    Number(f64),
    Operation(Box<Expression>, Operation, Box<Expression>),
    Or(Box<Expression>, Box<Expression>),
    String(String),
    Unary(Unary, Box<Expression>),
    Variable(Identifier, Box<Expression>),
    ParseError,
}

pub type Block = Vec<Box<Statement>>;
pub type Param = (Identifier, Identifier);

#[derive(Clone, Debug)]
pub struct Identifier {
    name: String,
    generic_fields: Vec<Identifier>,
}

impl Identifier {
    pub fn new(name: String, generic_fields: Vec<Identifier>) -> Self {
        Identifier {
            name,
            generic_fields,
        }
    }

    pub fn new_non_generic(name: String) -> Self {
        Identifier::new(name, Vec::new())
    }

    pub fn get_name(&self) -> String {
        self.name.clone()
    }

    pub fn get_generic_fields(&self) -> &Vec<Identifier> {
        &self.generic_fields
    }

    pub fn get_type(&self, pointer_type: Type) -> Type {
        let name = self.get_name();
        let rox_type = match name.as_str() {
            "Bool" => RoxType::Bool,
            "Number" => RoxType::Number,
            "String" => RoxType::String,
            x => {
                dbg!(x);
                unimplemented!()
            }
        };
        rox_type.get_codegen_type(pointer_type)
    }
}

impl From<Identifier> for String {
    fn from(ident: Identifier) -> Self {
        let mut final_string = String::new();
        final_string.push_str(ident.get_name().as_str());
        ident.get_generic_fields().iter().for_each(|field| {
            final_string.push_str(String::from(field.clone()).as_str());
        });
        final_string
    }
}

impl Hash for Identifier {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
        self.generic_fields.hash(state);
    }
}

impl PartialEq for Identifier {
    fn eq(&self, other: &Self) -> bool {
        self.get_name() == other.get_name()
            && self.get_generic_fields() == other.get_generic_fields()
    }
}

impl Eq for Identifier {}

impl From<&str> for Identifier {
    fn from(s: &str) -> Self {
        Identifier::new(s.to_string(), Vec::new())
    }
}

impl From<String> for Identifier {
    fn from(s: String) -> Self {
        Identifier::new(s, Vec::new())
    }
}

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
    Array,
    // TODO: Support user-defined types
    // UserType(String),
}

impl RoxType {
    pub fn get_codegen_type(&self, pointer_type: Type) -> Type {
        match self {
            RoxType::Void => types::INVALID,
            RoxType::Bool => types::B1,
            RoxType::Number => types::F64,
            RoxType::String => pointer_type,
            RoxType::Array => pointer_type,
        }
    }
}

impl From<ArenaType> for RoxType {
    fn from(arena_type: ArenaType) -> Self {
        match arena_type {
            semant::NUMBER_TYPE_VAL => RoxType::Number,
            semant::BOOL_TYPE_VAL => RoxType::Bool,
            semant::STRING_TYPE_VAL => RoxType::String,
            semant::VOID_TYPE_VAL => RoxType::Void,
            x => {
                dbg!(x);
                panic!("Rox does not yet support user-defined types")
            }
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
            RoxType::Array => semant::ARRAY_TYPE_VAL,
        }
    }
}

#[derive(Clone, Debug)]
pub struct FunctionDeclaration {
    pub name: Identifier,
    pub params: Vec<Param>,
    pub return_type: Option<Identifier>,
}

#[derive(Clone, Debug)]
pub enum Statement {
    Expression(Box<Expression>),
    Return(Option<Box<Expression>>),
    Block(Block),
    IfElse(Box<Expression>, Block, Option<Block>),
    FunctionDeclaration(Identifier, Vec<Param>, Option<Identifier>, Block),
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
