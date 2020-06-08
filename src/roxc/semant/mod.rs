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
use crate::roxc::{Declaration, Expression, Statement};
use std::collections::{HashMap, HashSet};
use types::*;

pub static NUMBER_TYPE_VAL: ArenaType = 0;
pub static BOOL_TYPE_VAL: ArenaType = 1;
pub static STRING_TYPE_VAL: ArenaType = 2;

type Env = HashMap<String, ArenaType>;

fn get_builtin_types() -> Vec<Type> {
    let initial_types = vec![NUMBER_TYPE_VAL, BOOL_TYPE_VAL, STRING_TYPE_VAL];
    initial_types
        .iter()
        .map(|arena_type| Type::new_variable(*arena_type))
        .collect()
}

pub fn analyse_program(declarations: &[Declaration]) {
    let mut types = get_builtin_types();
    let mut env = HashMap::new();
    let non_generics = HashSet::new();
    declarations
        .iter()
        .for_each(|declaration| match declaration {
            Declaration::Function(statement) => analyse_statement(
                &mut types,
                *statement.clone(),
                &mut env,
                &non_generics,
            ),
        })
}

fn analyse_statement(
    types: &mut Vec<Type>,
    node: Statement,
    env: &mut Env,
    non_generic: &HashSet<ArenaType>,
) {
    use Statement::*;
    match node {
        Expression(expression) => {
            analyse_expression(types, expression, env, non_generic);
        }
        FunctionDeclaration(name, params, return_type, statements) => {
            let param_types = params
                .iter()
                .map(|(param_name, param_type_name)| {
                    let variable = new_variable(types);
                    let param_arena_type =
                        env.get(param_type_name.as_str()).unwrap();
                    types.push(Type::Variable {
                        id: variable,
                        instance: Some(*param_arena_type),
                    });
                    (param_name.clone(), *param_arena_type)
                })
                .collect::<Vec<_>>();
            let result_type = new_variable(types);
            let mut new_env = env.clone();
            let mut new_non_generic = non_generic.clone();
            let arg_types = param_types
                .iter()
                .map(|(name, arg_type)| {
                    new_env.insert(name.parse().unwrap(), *arg_type);
                    new_non_generic.insert(*arg_type);
                    *arg_type
                })
                .collect::<Vec<_>>();

            statements.iter().for_each(|statement| {
                match statement.as_ref() {
                    Return(maybe_expression) => {
                        if let Some(expression) = maybe_expression {
                            let expression_type =  analyse_expression(
                                types,
                                expression.clone(),
                                env,
                                non_generic,
                            );
                            unify(
                                types,
                                expression_type,
                                result_type,
                            );
                        } else if return_type.is_some() {
                            panic!("Type mismatch: expected a value to be returned");
                        }
                    }
                    _ => analyse_statement(
                        types,
                        *statement.clone(),
                        env,
                        non_generic,
                    ),
                }
            });

            let new_arena_type =
                new_function(types, arg_types.as_ref(), &[result_type]);
            env.insert(name, new_arena_type);
        }
        _ => panic!(""),
    }
}

fn analyse_expression(
    types: &mut Vec<Type>,
    node: Box<Expression>,
    env: &mut Env,
    non_generic: &HashSet<ArenaType>,
) -> ArenaType {
    match *node {
        Expression::Assignment(name, expression) => {
            let expr_type =
                analyse_expression(types, expression, env, non_generic);
            let variable_type = env.get(&name).unwrap();
            unify(types, expr_type, *variable_type);
            expr_type
        }
        Expression::Identifier(ref name) => {
            get_type(types, name.as_ref(), *node.clone(), env, non_generic)
        }
        Expression::String(_) => STRING_TYPE_VAL,
        Expression::Number(_) => NUMBER_TYPE_VAL,
        Expression::Boolean(_) => BOOL_TYPE_VAL,
        Expression::Variable(name, expression) => {
            let expr_type =
                analyse_expression(types, expression, env, non_generic);
            let variable = new_variable(types);
            types.push(Type::Variable {
                id: variable,
                instance: Some(expr_type),
            });
            env.insert(name, variable);
            expr_type
        }
        Expression::Or(left, right) | Expression::And(left, right) => {
            let left_type = analyse_expression(types, left, env, non_generic);
            let right_type = analyse_expression(types, right, env, non_generic);
            unify(types, left_type, BOOL_TYPE_VAL);
            unify(types, right_type, BOOL_TYPE_VAL);
            BOOL_TYPE_VAL
        }
        Expression::Operation(left, _, right) => {
            let left_type = analyse_expression(types, left, env, non_generic);
            let right_type = analyse_expression(types, right, env, non_generic);
            unify(types, left_type, NUMBER_TYPE_VAL);
            unify(types, right_type, NUMBER_TYPE_VAL);
            NUMBER_TYPE_VAL
        }
        Expression::Unary(_, expression) => {
            let expr_type =
                analyse_expression(types, expression, env, non_generic);
            unify(types, expr_type, NUMBER_TYPE_VAL);
            NUMBER_TYPE_VAL
        }
        Expression::FunctionCall(name, arg_expressions) => {
            let function_arena_type = *env.get(&name).unwrap();
            let function_type_signature =
                types.get(function_arena_type).unwrap().clone();
            if let Type::Function { return_types, .. } = function_type_signature
            {
                let new_arg_types = arg_expressions
                    .iter()
                    .map(|arg| {
                        analyse_expression(types, arg.clone(), env, non_generic)
                    })
                    .collect::<Vec<_>>();

                let new_return_types = return_types
                    .iter()
                    .map(|_| new_variable(types))
                    .collect::<Vec<_>>();

                let func = new_function(
                    types,
                    new_arg_types.as_ref(),
                    new_return_types.as_ref(),
                );
                unify(types, func, function_arena_type);
                new_return_types[0] // TODO: This will probably need to be refactored to support multiple returns
                                    // since functions no longer resolve to one value}
            } else {
                panic!("Type mismatch: tried to call an object that is not a function")
            }
        }
        x => {
            println!("Got type: {:?}", x);
            panic!("This shouldn't have happened?");
        }
    }
}

fn get_type(
    types: &mut Vec<Type>,
    name: &str,
    expression: Expression,
    env: &Env,
    non_generic: &HashSet<ArenaType>,
) -> ArenaType {
    if let Some(value) = env.get(name) {
        let cloned_non_generics =
            &non_generic.iter().cloned().collect::<Vec<_>>();
        fresh(types, *value, cloned_non_generics)
    } else if let Some(literal_val) = maybe_get_literal(expression) {
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
            Type::Function {
                arg_types,
                return_types,
                ..
            } => {
                let fresh_args = arg_types
                    .iter()
                    .map(|type_| {
                        recursive_fresh(types, *type_, aliases, non_generics)
                    })
                    .collect::<Vec<_>>();
                let fresh_returns = return_types
                    .iter()
                    .map(|type_| {
                        recursive_fresh(types, *type_, aliases, non_generics)
                    })
                    .collect::<Vec<_>>();
                new_function(types, fresh_args.as_ref(), fresh_returns.as_ref())
            }
        }
    }

    recursive_fresh(types, arena_type, &mut mappings, non_generics)
}

/// Returns the currently defining instance of `type_`.
/// This returns an uninstantiated TypeVariable
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
        (Type::Function { .. }, Type::Variable { .. }) => {
            unify(types, second_pruned, first_pruned)
        }
        (
            Type::Function {
                name: ref a_name,
                return_types: ref a_types,
                ..
            },
            Type::Function {
                name: ref b_name,
                return_types: ref b_types,
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
        Type::Function {
            ref return_types, ..
        } => occurs_in(types, v, return_types),
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

fn maybe_get_literal(expression: Expression) -> Option<ArenaType> {
    use Expression::*;
    // TODO: Fix all of this
    match expression {
        Number(_) => Some(NUMBER_TYPE_VAL),
        String(_) => Some(STRING_TYPE_VAL),
        Boolean(_) => Some(BOOL_TYPE_VAL),
        _ => None,
    }
}
