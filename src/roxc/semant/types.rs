use crate::roxc::Identifier;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
#[allow(dead_code)]
pub enum TypeConstructor {
    Bool,
    Float,
    Int,
    String,
    Void,
    Arrow,
    Array(Box<Type>),
    /// The Record type constructor takes a list of field name identifiers and their types
    Record(Vec<(Identifier, Type)>),
    /// Takes the list of formal type parameters and the return type
    FunctionType(Vec<Identifier>, Box<Type>),
    /// `Unique` type constructors exist to differentiate two named types with the same fields.
    /// For example, two types of `type first = {a: String}` and `type second = {a: String}` should
    /// not be considered equal types, even though they have the same field names and types.
    ///
    /// To get around this equality check, we check that both type declarations point to a
    /// unique type constructor, so we can always check that the pointers refer to different locations
    /// to determine that they refer to different type constructors.
    Unique(Box<TypeConstructor>),
}

impl TypeConstructor {
    pub fn get_record_fields(&self) -> Vec<(Identifier, Type)> {
        match self {
            TypeConstructor::Record(fields) => fields.clone(),
            _ => panic!("Tried to get records for non-record type"), // TODO: improve error handling
        }
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum Type {
    Apply(TypeConstructor, Vec<Type>),
    Variable(String),
    /// (0) is the list of formal arguments,
    /// and (1) is the rest of the type declaration
    Polymorphic(Vec<Identifier>, Box<Type>),
}

impl Type {
    pub fn get_record_fields(&self) -> Vec<(Identifier, Type)> {
        match self {
            Type::Apply(constructor, _) => constructor.get_record_fields(),
            _ => panic!("Tried to get record fields for non-record type"), // TODO: improve error handling
        }
    }
}
