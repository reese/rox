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

pub(crate) mod tagged_syntax;
pub(crate) mod types;
use crate::roxc::semant::tagged_syntax::{
    TaggedDeclaration, TaggedExpression, TaggedStatement,
};
use crate::roxc::{
    get_builtin_types, syntax, Declaration, Expression, Identifier, Param,
    Result, RoxError, RoxType, Statement,
};
use std::cmp::min;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
pub use types::ArenaType;
use types::*;

pub const VOID_TYPE_VAL: ArenaType = 0;
pub const NUMBER_TYPE_VAL: ArenaType = 1;
pub const BOOL_TYPE_VAL: ArenaType = 2;
pub const STRING_TYPE_VAL: ArenaType = 3;
pub const ARRAY_TYPE_VAL: ArenaType = 4;

pub(crate) type Env = HashMap<Identifier, ArenaType>;

/// # Type Checking
/// `analyse_program` is the top level function from which we
/// type check the entire program.
pub(crate) fn analyse_program(
    declarations: &[Declaration],
) -> Result<Vec<TaggedDeclaration>> {
    let (mut types, mut env) = get_builtin_types();
    let non_generics = HashSet::new();
    declarations
        .iter()
        .map(|declaration| {
            let tagged_statement = match declaration {
                Declaration::Function(statement) => analyse_statement(
                    &mut types,
                    *statement.clone(),
                    &mut env,
                    &non_generics,
                )?,
            };
            Ok(TaggedDeclaration::Function(tagged_statement))
        })
        .collect::<Result<Vec<_>>>()
}

fn analyse_statement(
    types: &mut Vec<Type>,
    node: Statement,
    env: &mut Env,
    non_generic: &HashSet<ArenaType>,
) -> Result<TaggedStatement> {
    use Statement::*;
    match node {
        Block(block) => Ok(TaggedStatement::Block(analyse_block(
            types,
            env,
            non_generic,
            block,
        )?)),
        Expression(expression) => {
            let tagged_expression =
                analyse_expression(types, *expression, env, non_generic)?;
            Ok(TaggedStatement::Expression(tagged_expression))
        }
        IfElse(if_expression, if_block, else_block) => {
            let tagged_if =
                analyse_expression(types, *if_expression, env, non_generic)?;
            unify(types, BOOL_TYPE_VAL, tagged_if.clone().into());
            let tagged_if_block =
                analyse_block(types, env, non_generic, if_block)?;
            let tagged_else_block = else_block
                .map(|block| analyse_block(types, env, non_generic, block));
            if let Some(result) = tagged_else_block.clone() {
                if result.is_err() {
                    return Err(result.err().unwrap());
                }
            }

            Ok(TaggedStatement::IfElse(
                Box::from(tagged_if),
                tagged_if_block,
                tagged_else_block.map(|r| r.unwrap()),
            ))
        }
        FunctionDeclaration(name, params, return_type, statements) => {
            let param_types = get_param_types(types, env, &params)?;
            let result_type =
                return_type.clone().map(|t| *env.get(&t).unwrap());
            let new_env = env.clone();
            if let Some(return_) = return_type.clone() {
                let type_ = new_env.get(&return_).unwrap();
                unify(types, result_type.unwrap(), *type_);
            }
            let new_non_generic = non_generic.clone();
            let arg_types =
                get_arg_types(param_types, new_env, new_non_generic)?;

            let tagged_block: Vec<Box<TaggedStatement>> = statements
                .iter()
                .map(|statement| {
                    let tagged_statement = match statement.as_ref() {
                        Return(maybe_expression) => {
                            if let Some(expression) = maybe_expression {
                                let expression_type = analyse_expression(
                                    types,
                                    *expression.clone(),
                                    env,
                                    non_generic,
                                )?;
                                unify(
                                    types,
                                    expression_type.clone().into(),
                                    result_type.unwrap(),
                                );
                                Ok(TaggedStatement::Return(Some(
                                    expression_type,
                                )))
                            } else {
                                Ok(TaggedStatement::Return(None))
                            }
                        }
                        _ => Ok(analyse_statement(
                            types,
                            *statement.clone(),
                            env,
                            non_generic,
                        )?),
                    }?;
                    Ok(Box::new(tagged_statement))
                })
                .collect::<Result<Vec<_>>>()?;

            let return_type_vec =
                result_type.map_or_else(Vec::new, |t| vec![t]);

            let new_arena_type = new_function(
                name.clone(),
                types,
                arg_types.as_ref(),
                return_type_vec.as_slice(),
            );
            env.insert(name.clone(), new_arena_type);
            let declaration = syntax::FunctionDeclaration {
                name,
                params,
                return_type,
            };
            Ok(TaggedStatement::FunctionDeclaration(
                declaration,
                tagged_block,
            ))
        }
        Return(maybe_expression) => {
            let maybe_tagged_return = maybe_expression
                .map(|exp| analyse_expression(types, *exp, env, non_generic));
            if let Some(result) = maybe_tagged_return.clone() {
                if result.is_err() {
                    return Err(result.err().unwrap());
                }
            }
            Ok(TaggedStatement::Return(
                maybe_tagged_return.map(|r| r.unwrap()),
            ))
        }
    }
}

#[allow(clippy::vec_box)]
fn analyse_block(
    types: &mut Vec<Type>,
    env: &mut Env,
    non_generic: &HashSet<usize>,
    if_block: Vec<Box<Statement>>,
) -> Result<Vec<Box<TaggedStatement>>> {
    let mut scoped_env = env.clone();
    if_block
        .iter()
        .map(|statement| {
            Ok(Box::new(analyse_statement(
                types,
                *statement.clone(),
                &mut scoped_env,
                non_generic,
            )?))
        })
        .collect::<Result<Vec<_>>>()
}

fn get_arg_types(
    param_types: Vec<(Identifier, ArenaType)>,
    mut new_env: Env,
    mut new_non_generic: HashSet<ArenaType>,
) -> Result<Vec<usize>> {
    param_types
        .iter()
        .map(|(name, arg_type)| {
            new_env.insert(name.clone(), *arg_type);
            new_non_generic.insert(*arg_type);
            Ok(*arg_type)
        })
        .collect::<Result<Vec<_>>>()
}

fn get_param_types(
    types: &mut Vec<Type>,
    env: &mut Env,
    params: &[Param],
) -> Result<Vec<(Identifier, usize)>> {
    params
        .iter()
        .map(|(param_name, param_type_name)| {
            let variable = new_variable(types);
            let param_arena_type = get_type(param_type_name, env)?;
            unify(types, variable, param_arena_type);
            env.insert(param_name.clone(), param_arena_type);
            Ok((param_name.clone(), param_arena_type))
        })
        .collect::<Result<Vec<_>>>()
}

fn analyse_expression(
    types: &mut Vec<Type>,
    node: Expression,
    env: &mut Env,
    non_generic: &HashSet<ArenaType>,
) -> Result<TaggedExpression> {
    match node {
        Expression::Assignment(name, expression) => {
            let tagged_expression =
                analyse_expression(types, *expression, env, non_generic)?;
            let variable_type = env.get(&name).unwrap();
            unify(types, tagged_expression.clone().into(), *variable_type);
            Ok(TaggedExpression::Assignment(name, Box::new(tagged_expression)))
        }
        Expression::Identifier(ref name) => {
            let arena_type = get_type(name, env)?;
            Ok(TaggedExpression::Identifier(
                name.clone(),
                RoxType::from(arena_type),
            ))
        }
        Expression::String(string) => Ok(TaggedExpression::String(string)),
        Expression::Number(num) => Ok(TaggedExpression::Number(num)),
        Expression::Boolean(bool) => Ok(TaggedExpression::Boolean(bool)),
        Expression::Variable(name, expression) => {
            let tagged_expression =
                analyse_expression(types, *expression, env, non_generic)?;
            let variable = new_variable(types);
            unify(types, variable, tagged_expression.clone().into());
            env.insert(name.clone(), tagged_expression.clone().into());
            Ok(TaggedExpression::Variable(name, Box::new(tagged_expression)))
        }
        Expression::Or(left, right) => {
            let left_expression =
                analyse_expression(types, *left, env, non_generic)?;
            let right_expression =
                analyse_expression(types, *right, env, non_generic)?;
            unify(types, left_expression.clone().into(), BOOL_TYPE_VAL);
            unify(types, right_expression.clone().into(), BOOL_TYPE_VAL);
            Ok(TaggedExpression::Or(
                left_expression.into(),
                right_expression.into(),
            ))
        }
        Expression::And(left, right) => {
            let left_expression =
                analyse_expression(types, *left, env, non_generic)?;
            let right_expression =
                analyse_expression(types, *right, env, non_generic)?;
            unify(types, left_expression.clone().into(), BOOL_TYPE_VAL);
            unify(types, right_expression.clone().into(), BOOL_TYPE_VAL);
            Ok(TaggedExpression::And(
                left_expression.into(),
                right_expression.into(),
            ))
        }
        Expression::Operation(left, operation, right) => {
            let left_expression =
                analyse_expression(types, *left, env, non_generic)?;
            let right_expression =
                analyse_expression(types, *right, env, non_generic)?;
            unify(types, left_expression.clone().into(), NUMBER_TYPE_VAL);
            unify(types, right_expression.clone().into(), NUMBER_TYPE_VAL);
            Ok(TaggedExpression::Operation(
                left_expression.into(),
                operation,
                right_expression.into(),
            ))
        }
        Expression::Unary(operator, expression) => {
            let tagged_expression =
                analyse_expression(types, *expression, env, non_generic)?;
            unify(types, tagged_expression.clone().into(), NUMBER_TYPE_VAL);
            Ok(TaggedExpression::Unary(operator, tagged_expression.into()))
        }
        Expression::FunctionCall(name, arg_expressions) => {
            let function_arena_type = *env.get(&name).unwrap();
            let function_type_signature =
                types.get(function_arena_type).unwrap().clone();
            if let Type::Function { return_types, name, .. } = function_type_signature
            {
                let tagged_arg_expressions =
                    arg_expressions
                        .iter()
                        .map(|arg| {
                            analyse_expression(
                                types,
                                *arg.clone(),
                                env,
                                non_generic,
                            )
                        })
                        .collect::<Result<Vec<_>>>()?
                        .iter()
                        .map(|e| Box::new(e.clone()))
                        .collect::<Vec<_>>();

                let arg_arena_types = tagged_arg_expressions
                    .iter()
                    .map(|t| (*t.clone()).into())
                    .collect::<Vec<_>>();

                let func = new_function(
                    name.clone(),
                    types,
                    arg_arena_types.as_ref(),
                    return_types.as_ref(),
                );
                unify(types, func, function_arena_type);
                Ok(TaggedExpression::FunctionCall(
                    name,
                    tagged_arg_expressions,
                    return_types
                        .get(0)
                        .map(|t| RoxType::from(*t))
                        .unwrap_or(RoxType::Void),
                ))
            // TODO: ^^ This will probably need to be refactored to support multiple returns
            // since functions no longer resolve to one value}
            } else {
                panic!("Type mismatch: tried to call an object that is not a function")
            }
        }
        Expression::Array(_) => todo!(),
        Expression::ParseError => panic!("Encountered errors while parsing, cannot type check invalid syntax."),
    }
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
                    panic!("Cannot unify recursive type definitions.");
                }
                types
                    .get_mut(first_pruned)
                    .unwrap()
                    .set_instance(min(first_pruned, second_pruned));
            }
        }
        (
            Type::Operator {
                name: left_name,
                types: left_types,
            },
            Type::Operator {
                name: right_name,
                types: right_types,
            },
        ) => {
            if left_name != right_name || left_types.len() != right_types.len()
            {
                panic!("Type mismatch")
            } else {
                for (p, q) in left_types.iter().zip(right_types.iter()) {
                    unify(types, *p, *q);
                }
            }
        }
        (Type::Function { .. }, Type::Variable { .. })
        | (Type::Operator { .. }, Type::Variable { .. }) => {
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
                panic!(format!(
                    "Type mismatch: {:?} != {:?}",
                    a_types, b_types
                )); // TODO: improve error reporting
            }
            for (p, q) in a_types.iter().zip(b_types.iter()) {
                unify(types, *p, *q);
            }
        }
        (Type::Function { .. }, Type::Operator { .. })
        | (Type::Operator { .. }, Type::Function { .. }) => {
            panic!("Cannot unify function and operator")
        }
    }
}

fn get_type(name: &Identifier, env: &Env) -> Result<ArenaType> {
    match env.get(name) {
        Some(arena_type) => Ok(*arena_type),
        None => Err(RoxError::new(
            PathBuf::from("./src/roxc/semant/mod.rs"), // TODO: Actually pass the file in here
            format!("Undefined symbol: {:?}", name).as_ref(),
        )),
    }
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
