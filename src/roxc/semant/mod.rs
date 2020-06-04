//! # Semantic Analysis
//! The `semant` module holds all of the semantic analysis
//! that happens as part of the compilation process.
//! The most important part of this analysis is
//! type inference.
//!
//! ## Type System
//! `Rox` uses an implementation of the [Hindley-Milner type system](https://en.wikipedia.org/wiki/Hindley%E2%80%93Milner_type_system),
//! the same type system used by several functional languages such as Haskell and ML.
//! The HM type system allows for most -- if not all -- types
//! to be inferred from their usage, allowing users to write
//! type-safe code with as little manual typing as possible.
//!
//! ### Hindley-Milner Implementation
//! This implementation of the HM type system is largely a port of
//! Rob Smallshire's [`hindley-milner-python`](https://github.com/rob-smallshire/hindley-milner-python)
//! implementation. It has been ported here in Rust and modified to match
//! the implementation of Rox's lexer.

mod types;
use crate::roxc::Expression;
use std::collections::{HashMap, HashSet};
use types::*;

lazy_static! {
    static ref FLOAT: Type = Type::new_operator(0, "float", &[]);
    static ref BOOL: Type = Type::new_operator(1, "bool", &[]);
    static ref STRING: Type = Type::new_operator(2, "str", &[]);
}

type Env = HashMap<String, ArenaType>;

pub fn analyse(
    types: &mut Vec<Type>,
    node: Box<Syntax>,
    env: &Env,
    non_generic: &HashSet<ArenaType>,
) -> ArenaType {
    use Syntax::*;
    match *node {
        Identifier { ref name } => {
            get_type(types, name.as_ref(), *node.clone(), env, non_generic)
        }
        Apply { function, arg } => {
            let function_type = analyse(types, function, env, non_generic);
            let arg_type = analyse(types, arg, env, non_generic);
            let result_type = new_variable(types);
            let first = new_function(types, arg_type, result_type.clone());
            unify(types, first, function_type);
            result_type
        }
        Function { ref name, ref body } => {
            let arg_type = new_variable(types);
            let mut new_env = env.clone();
            new_env.insert(name.clone(), arg_type);
            let mut new_non_generic = non_generic.clone();
            new_non_generic.insert(arg_type.clone());
            let result_type =
                analyse(types, body.clone(), &new_env, &new_non_generic);
            new_function(types, arg_type, result_type)
        }
        Literal { value } => match *value {
            Expression::Boolean(_) => 1,
            Expression::Number(_) => 2,
            Expression::String(_) => 0,
            _ => panic!("This isn't a literal"),
        },
    }
}

fn get_type(
    types: &mut Vec<Type>,
    name: &str,
    syntax: Syntax,
    env: &Env,
    non_generic: &HashSet<ArenaType>,
) -> ArenaType {
    if let Some(value) = env.get(name) {
        let cloned_non_generics =
            &non_generic.iter().cloned().collect::<Vec<_>>();
        fresh(types, *value, cloned_non_generics)
    } else if let Some(literal_val) = maybe_get_literal(syntax) {
        literal_val
    } else {
        panic!("Undefined symbol {:?}", name);
    }
}

fn fresh(
    types: &mut Vec<Type>,
    arena_type: ArenaType,
    non_generics: &[ArenaType],
) -> ArenaType {
    let mut mappings = HashMap::new();

    fn recursive_fresh(
        types: &mut Vec<Type>,
        arena_type: ArenaType,
        aliases: &mut HashMap<ArenaType, ArenaType>,
        non_generics: &[ArenaType],
    ) -> ArenaType {
        let pruned_type = prune(types, arena_type);
        match types.get(pruned_type).unwrap().clone() {
            Type::Variable { .. } => {
                if is_generic(types, pruned_type, non_generics) {
                    *aliases
                        .entry(pruned_type)
                        .or_insert_with(|| new_variable(types))
                } else {
                    pruned_type
                }
            }
            Type::Operator {
                name,
                types: operator_types,
                ..
            } => {
                let operator = operator_types
                    .iter()
                    .map(|type_| {
                        recursive_fresh(types, *type_, aliases, non_generics)
                    })
                    .collect::<Vec<_>>();
                new_operator(types, name.as_ref(), &operator)
            }
        }
    }

    recursive_fresh(types, arena_type, &mut mappings, non_generics)
}

/// Returns the currently defining instance of `type_`.
/// This returns an uninstantiated Type{Variable|Operator}
fn prune(types: &mut Vec<Type>, type_: ArenaType) -> ArenaType {
    let new_type = match *types.get(type_).unwrap() {
        Type::Variable { instance, .. } => {
            if let Some(value) = instance {
                value
            } else {
                return type_;
            }
        }
        _ => return type_,
    };

    let value = prune(types, new_type);
    match *types.get_mut(type_).unwrap() {
        Type::Variable {
            ref mut instance, ..
        } => {
            *instance = Some(value);
        }
        _ => {
            return type_;
        }
    }
    value
}

fn unify(types: &mut Vec<Type>, first_type: ArenaType, second_type: ArenaType) {
    let first_pruned = prune(types, first_type);
    let second_pruned = prune(types, second_type);
    match (
        types.get(first_pruned).unwrap().clone(),
        types.get(second_pruned).unwrap().clone(),
    ) {
        (Type::Variable { .. }, _) => {
            if first_pruned != second_pruned {
                if occurs_in_type(types, first_pruned, second_pruned) {
                    panic!("recursive unification");
                }
                types
                    .get_mut(first_pruned)
                    .unwrap()
                    .set_instance(second_pruned);
            }
        }
        (Type::Operator { .. }, Type::Variable { .. }) => {
            unify(types, second_pruned, first_pruned)
        }
        (
            Type::Operator {
                name: ref a_name,
                types: ref a_types,
                ..
            },
            Type::Operator {
                name: ref b_name,
                types: ref b_types,
                ..
            },
        ) => {
            if a_name != b_name || a_types.len() != b_types.len() {
                //raise InferenceError("Type mismatch: {0} != {1}".format(str(a), str(b)))
                panic!("type mismatch");
            }
            for (p, q) in a_types.iter().zip(b_types.iter()) {
                unify(types, *p, *q);
            }
        }
    }
}

fn is_generic(
    types: &mut Vec<Type>,
    arena_type: ArenaType,
    non_generics: &[ArenaType],
) -> bool {
    !occurs_in(types, arena_type, non_generics)
}

fn occurs_in_type(
    types: &mut Vec<Type>,
    v: ArenaType,
    type2: ArenaType,
) -> bool {
    let pruned_type2 = prune(types, type2);
    if pruned_type2 == v {
        return true;
    }

    match types.get(pruned_type2).unwrap().clone() {
        Type::Operator {
            types: ref operator_types,
            ..
        } => occurs_in(types, v, operator_types),
        _ => false,
    }
}

fn occurs_in(
    types: &mut Vec<Type>,
    arena_type: ArenaType,
    non_generics: &[ArenaType],
) -> bool {
    non_generics
        .iter()
        .any(|t| occurs_in_type(types, arena_type, *t))
}

fn maybe_get_literal(syntax: Syntax) -> Option<ArenaType> {
    use Syntax::*;
    match syntax {
        Function { .. } => None,
        Identifier { .. } => None,
        Apply { .. } => None,
        Literal { value } => match *value {
            Expression::String(..) => Some(2),
            Expression::Number(_float) => Some(0),
            Expression::Boolean(_bool) => Some(1),
            _ => None,
        },
    }
}
