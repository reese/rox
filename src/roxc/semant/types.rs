use crate::roxc::Identifier;

pub type ArenaType = usize;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Type {
    /// A `Function` is much like an `Operator` in the sense
    /// that it is the concrete implementation of
    /// a map from some types to other types
    Function {
        id: ArenaType,
        name: Identifier,
        arg_types: Vec<ArenaType>,
        return_types: Vec<ArenaType>,
    },
    /// This `Operator` struct represents an `n`-ary
    /// constructor to create a new type from `n` existing types
    Operator {
        name: Identifier,
        types: Vec<ArenaType>,
    },
    Variable {
        id: ArenaType,
        instance: Option<ArenaType>,
    },
}

impl Type {
    pub(crate) fn new_operator(
        name: Identifier,
        types: Vec<ArenaType>,
    ) -> Type {
        Type::Operator { name, types }
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

pub fn new_operator(
    types: &mut Vec<Type>,
    name: Identifier,
    operator_types: Vec<ArenaType>,
) -> ArenaType {
    let type_ = Type::new_operator(name, operator_types);
    types.push(type_);
    types.len() - 1
}
