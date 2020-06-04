use crate::roxc::{Expression, RoxType};
use std::collections::HashMap;

#[derive(Clone)]
pub enum Syntax {
    Function {
        name: String,
        body: Box<Syntax>,
    },
    Identifier {
        name: String,
    },
    Apply {
        function: Box<Syntax>,
        arg: Box<Syntax>,
    },
    Literal {
        value: Box<Expression>,
    },
}

pub type ArenaType = usize;

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum Type {
    Variable {
        id: ArenaType,
        instance: Option<ArenaType>,
    },
    Operator {
        id: ArenaType,
        name: String,
        types: Vec<ArenaType>,
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

    pub fn new_operator(
        id: ArenaType,
        name: &str,
        types: &[ArenaType],
    ) -> Type {
        Type::Operator {
            id,
            name: name.to_string(),
            types: types.to_vec(),
        }
    }

    fn get_id(&self) -> usize {
        match self {
            &Type::Variable { id, .. } => id,
            &Type::Operator { id, .. } => id,
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
    from_type: ArenaType,
    to_type: ArenaType,
) -> ArenaType {
    let type_ = Type::new_operator(types.len(), "->", &[from_type, to_type]);
    types.push(type_);
    types.len() - 1
}

pub fn new_variable(types: &mut Vec<Type>) -> ArenaType {
    let type_ = Type::new_variable(types.len());
    types.push(type_);
    types.len() - 1
}

pub fn new_operator(
    a: &mut Vec<Type>,
    name: &str,
    types: &[ArenaType],
) -> ArenaType {
    let t = Type::new_operator(a.len(), name, types);
    a.push(t);
    a.len() - 1
}
