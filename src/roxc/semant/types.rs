use crate::roxc::Identifier;
use std::collections::HashMap;

pub type ArenaType = usize;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Type {
    /// A `Function` is much like an `Operator` in the sense
    /// that it is the implementation of mapping from some type
    /// to another type. However, what those types are
    /// may not be known until the function is called, which
    /// is why we have the optional types here.
    Function {
        id: ArenaType,
        name: Identifier,
        arg_types: Vec<ArenaType>,
        return_types: Vec<ArenaType>,
    },
    Struct {
        name: Identifier,
        generic_types: Vec<String>,
        fields: HashMap<Identifier, ArenaType>,
        // TODO: support `traits: Vec<Trait>` for this and with generics
    },
    Variable {
        id: ArenaType,
        instance: Option<ArenaType>,
    },
}

impl Type {
    pub(crate) fn new_struct(
        name: Identifier,
        generic_types: Vec<String>,
        fields: HashMap<Identifier, ArenaType>,
    ) -> Type {
        Type::Struct {
            name,
            generic_types,
            fields,
        }
    }

    pub(crate) fn new_variable(id: ArenaType) -> Type {
        Type::Variable { id, instance: None }
    }

    pub(crate) fn new_function(
        id: ArenaType,
        name: Identifier,
        arg_types: &[ArenaType],
        return_types: &[ArenaType],
    ) -> Type {
        Type::Function {
            id,
            name,
            arg_types: arg_types.to_vec(),
            return_types: return_types.to_vec(),
        }
    }

    pub(crate) fn set_instance(&mut self, instance: ArenaType) {
        match *self {
            Type::Variable {
                instance: ref mut inst,
                ..
            } => *inst = Some(instance),
            _ => unimplemented!(),
        }
    }
}

pub fn new_function(
    name: Identifier,
    types: &mut Vec<Type>,
    from_type: &[ArenaType],
    to_type: &[ArenaType],
) -> ArenaType {
    let type_ = Type::new_function(types.len(), name, from_type, to_type);
    types.push(type_);
    types.len() - 1
}

pub fn new_variable(types: &mut Vec<Type>) -> ArenaType {
    let type_ = Type::new_variable(types.len());
    types.push(type_);
    types.len() - 1
}

pub fn new_struct(
    types: &mut Vec<Type>,
    name: Identifier,
    generic_types: Vec<String>,
    operator_types: HashMap<Identifier, ArenaType>,
) -> ArenaType {
    let type_ = Type::new_struct(name, generic_types, operator_types);
    types.push(type_);
    types.len() - 1
}
