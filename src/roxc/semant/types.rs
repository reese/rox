use crate::roxc::RoxType;
use std::collections::HashMap;

pub type ArenaType = usize;

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum Type {
    Variable {
        id: ArenaType,
        instance: Option<ArenaType>,
    },
    Function {
        id: ArenaType,
        name: String,
        arg_types: Vec<ArenaType>,
        return_types: Vec<ArenaType>,
    },
}

struct Namer {
    value: char,
    set: HashMap<ArenaType, String>,
}

impl Namer {
    fn next(&mut self) -> String {
        let v = self.value;
        self.value = ((self.value as u8) + 1) as char;
        format!("{}", v)
    }

    fn name(&mut self, arena_type: ArenaType) -> String {
        let name = { self.set.get(&arena_type).map(|x| x.clone()) };
        if let Some(value) = name {
            value.clone()
        } else {
            let val = self.next();
            self.set.insert(arena_type, val.clone());
            val
        }
    }
}

impl Type {
    pub fn new_variable(id: ArenaType) -> Type {
        Type::Variable { id, instance: None }
    }

    pub fn new_function(
        id: ArenaType,
        name: &str,
        arg_types: &[ArenaType],
        return_types: &[ArenaType],
    ) -> Type {
        Type::Function {
            id,
            name: name.to_string(),
            arg_types: arg_types.to_vec(),
            return_types: return_types.to_vec(),
        }
    }

    fn get_id(&self) -> usize {
        match self {
            &Type::Variable { id, .. } => id,
            &Type::Function { id, .. } => id,
        }
    }

    pub(crate) fn set_instance(&mut self, instance: ArenaType) {
        match self {
            &mut Type::Variable {
                instance: mut inst, ..
            } => inst = Some(instance),
            _ => unimplemented!(),
        }
    }
}

pub fn new_function(
    types: &mut Vec<Type>,
    from_type: &[ArenaType],
    to_type: &[ArenaType],
) -> ArenaType {
    let type_ = Type::new_function(types.len(), "->", from_type, to_type);
    types.push(type_);
    types.len() - 1
}

pub fn new_variable(types: &mut Vec<Type>) -> ArenaType {
    let type_ = Type::new_variable(types.len());
    types.push(type_);
    types.len() - 1
}
