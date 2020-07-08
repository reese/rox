//! This module, unsurprisingly, performs Rox's type checking.
//! At a very high level, this consists of a two core functions: `substitute` and `unify`.
//!
//! ## Substitution
//!
//! Substitution refers to the substitution of formal type parameters with concrete types
//! in polymorphic type constructors. As an example, let's take the following function:
//!
//! #### TODO: Wrap this in a `rox::execute_source_string` to check during testing
//! `type List<T> = { head: T, tail: List<T> };`
//!
//! This function has a list of formal arguments -- here, a single argument, `T` -- and a type
//! constructor.
//! When we eventually instantiate this type -- say, in some function that returns a
//! `List<Number>` -- `substitute is responsible for replacing the type `T` of the type constructor
//! with the concrete type that the user provides: `Number`
//!
//! ## Unify
//!
//! Once we've constructed all of our types, we now need to verify that those types create
//! valid programs.
//! To do this, we need to verify that all the application of our types are equal, or "unified."
use crate::roxc::semant::types::{Type, TypeConstructor};
use crate::roxc::semant::{
    TaggedDeclaration, TaggedExpression, TaggedStatement,
};
use crate::roxc::{
    builtins, Declaration, Expression, FunctionDeclaration, Identifier, Result,
    RoxError, Statement, TypeName, Unary,
};
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub(crate) enum TypeValue {
    Type(Type),
    Constructor(TypeConstructor),
}

impl TypeValue {
    fn get_rox_type(&self) -> Type {
        match self {
            TypeValue::Type(t) => t.clone(),
            TypeValue::Constructor(_) => {
                panic!("Expected a type variable, not a type constructor")
            }
        }
    }
}

impl From<&Type> for TypeValue {
    fn from(t: &Type) -> TypeValue {
        TypeValue::Type(t.clone())
    }
}

pub(crate) type TypeEnv = HashMap<Identifier, TypeValue>;
pub(crate) type VariableEnv = HashMap<Identifier, Type>;

fn substitute(ty: Type, env: &mut TypeEnv) -> Type {
    match ty {
        Type::Apply(type_constructor, type_arguments) => match type_constructor
        {
            TypeConstructor::FunctionType(formal_types, type_) => {
                let mut type_mapping: TypeEnv =
                    zip_argument_types(formal_types, type_arguments);
                substitute(
                    substitute(type_.as_ref().clone(), &mut type_mapping),
                    env,
                )
            }
            _ => Type::Apply(
                type_constructor,
                type_arguments
                    .iter()
                    .map(|t| substitute(t.clone(), env))
                    .collect(),
            ),
        },
        Type::Variable(identifier) => match env.get(&identifier) {
            Some(type_) => type_.clone().get_rox_type(),
            None => Type::Variable(identifier),
        },
        Type::PolymorphicType(formal_parameters, type_) => {
            // NOTICE: This could totally be wrong because I have no idea what I'm doing.
            // Use with caution.
            let new_formal_parameters = formal_parameters.clone();
            let mut inner_scope: TypeEnv = zip_argument_types(
                formal_parameters,
                new_formal_parameters
                    .iter()
                    .cloned()
                    .map(Type::Variable)
                    .collect::<Vec<_>>(),
            );
            let new_type = substitute(*type_, &mut inner_scope);
            Type::PolymorphicType(new_formal_parameters, Box::new(new_type))
        }
    }
}

fn unify(type_one: Type, type_two: Type) -> Result<()> {
    match (type_one.clone(), type_two.clone()) {
        (Type::Variable(first), Type::Variable(second)) => {
            if first == second {
                Ok(())
            } else {
                Err(RoxError::with_file_placeholder(
                    format!(
                        "Type mismatch: attempted to unify {:?} and {:?}",
                        first, second
                    )
                    .as_ref(),
                ))
            }
        }
        (
            Type::PolymorphicType(original_formal_parameters, original_type),
            Type::PolymorphicType(other_format_parameters, other_type),
        ) => {
            let other_env: &mut TypeEnv = &mut zip_argument_types(
                other_format_parameters,
                original_formal_parameters
                    .iter()
                    .map(|p| Type::Variable(p.clone()))
                    .collect(),
            );
            unify(*original_type, substitute(*other_type, other_env))
        }
        (
            Type::Apply(first_type_constructor, first_type_arguments),
            Type::Apply(second_type_constructor, second_type_arguments),
        ) => match (first_type_constructor, second_type_constructor) {
            (TypeConstructor::FunctionType(formal_types, type_), _) => unify(
                substitute(
                    *type_,
                    &mut formal_types
                        .iter()
                        .cloned()
                        .zip(
                            second_type_arguments
                                .iter()
                                .map(TypeValue::from)
                                .collect::<Vec<_>>(),
                        )
                        .collect::<HashMap<_, _>>(),
                ),
                type_two,
            ),
            (_, TypeConstructor::FunctionType(formal_types, type_)) => unify(
                type_one,
                substitute(
                    *type_,
                    &mut zip_argument_types(formal_types, first_type_arguments),
                ),
            ),
            (
                TypeConstructor::Unique(first_constructor),
                TypeConstructor::Unique(second_constructor),
            ) => {
                if first_constructor != second_constructor {
                    Err(RoxError::with_file_placeholder(
                        format!(
                            "Type mismatch: attempted to unify {:?} and {:?}",
                            first_constructor, second_constructor
                        )
                        .as_ref(),
                    ))
                } else {
                    first_type_arguments
                        .iter()
                        .zip(second_type_arguments)
                        .map(|(first, second)| unify(first.clone(), second))
                        .collect()
                }
            }
            (_, _) => first_type_arguments
                .iter()
                .zip(second_type_arguments)
                .map(|(first, second)| unify(first.clone(), second))
                .collect(),
        },
        (x, y) => Err(RoxError::with_file_placeholder(
            format!("Type mismatch: attempted to unify {:?} and {:?}", x, y)
                .as_ref(),
        )),
    }
}

fn expand(t: Type) -> Type {
    match t.clone() {
        Type::Apply(constructor, types) => match constructor {
            TypeConstructor::FunctionType(formal_arguments, type_) => {
                expand(substitute(
                    *type_,
                    &mut zip_argument_types(formal_arguments, types),
                ))
            }
            TypeConstructor::Unique(constructor) => {
                expand(Type::Apply(*constructor, types))
            }
            _ => t,
        },
        _ => t,
    }
}

fn zip_argument_types(names: Vec<Identifier>, types: Vec<Type>) -> TypeEnv {
    names
        .iter()
        .cloned()
        .zip(types.iter().map(TypeValue::from).collect::<Vec<_>>())
        .collect::<HashMap<_, _>>()
}

fn translate_declaration(
    type_env: &mut TypeEnv,
    variable_env: &mut VariableEnv,
    declaration: &Declaration,
) -> Result<TaggedDeclaration> {
    match declaration {
        Declaration::Function(statement) => {
            Ok(TaggedDeclaration::Function(translate_statement(
                type_env,
                variable_env,
                statement.as_ref().clone(),
            )?))
        }
    }
}

fn translate_statement(
    type_env: &mut TypeEnv,
    variable_env: &mut VariableEnv,
    statement: Statement,
) -> Result<TaggedStatement> {
    match statement {
        Statement::Block(statements) => Ok(TaggedStatement::Block(
            statements
                .iter()
                .map(|s| {
                    translate_statement(
                        type_env,
                        variable_env,
                        s.as_ref().clone(),
                    )
                })
                .collect::<Result<Vec<_>>>()?,
        )),
        Statement::ExternFunctionDeclaration(
            func_name,
            parameters,
            return_type_name,
        ) => {
            let function_decl_types = parameters
                .iter()
                .map(|type_name| {
                    translate_type_identifier(
                        type_env,
                        type_name.as_ref().clone(),
                    )
                })
                .collect::<Result<Vec<Type>>>()?;
            let mut parameter_types = function_decl_types.clone();
            let return_type = translate_type_identifier(
                type_env,
                return_type_name
                    .unwrap_or(Box::new(TypeName::Type("Void".to_string())))
                    .as_ref()
                    .clone(),
            )?;
            parameter_types.push(return_type.clone());
            variable_env.insert(
                func_name.clone(),
                Type::PolymorphicType(
                    Vec::new(),
                    Box::new(Type::Apply(
                        TypeConstructor::Arrow,
                        parameter_types.clone(),
                    )),
                ),
            );
            Ok(TaggedStatement::ExternFunctionDeclaration(
                FunctionDeclaration {
                    name: func_name.clone(),
                    params: function_decl_types
                        .iter()
                        .map(|t| (String::new(), t.clone()))
                        .collect(),
                    return_type,
                },
            ))
        }
        Statement::FunctionDeclaration(
            func_name,
            maybe_formal_arguments,
            parameters,
            return_type_name,
            func_body,
        ) => {
            let mut local_type_env = type_env.clone();
            if let Some(formal_arguments) = maybe_formal_arguments.clone() {
                formal_arguments.iter().for_each(|ty| {
                    local_type_env.insert(
                        ty.clone(),
                        TypeValue::Type(Type::Variable(ty.clone())),
                    );
                });
            };
            let function_decl_types = parameters
                .iter()
                .map(|(param_name, type_name)| {
                    let type_ = translate_type_identifier(
                        &mut local_type_env,
                        type_name.clone(),
                    )?;
                    variable_env.insert(param_name.clone(), type_.clone());
                    Ok(type_)
                })
                .collect::<Result<Vec<Type>>>()?;
            let mut parameter_types = function_decl_types;

            let return_type = translate_type_identifier(
                &mut local_type_env,
                return_type_name
                    .unwrap_or(Box::new(TypeName::Type("Void".to_string())))
                    .as_ref()
                    .clone(),
            )?;
            parameter_types.push(return_type.clone());

            variable_env.insert(
                func_name.clone(),
                Type::PolymorphicType(
                    maybe_formal_arguments.unwrap_or_else(Vec::new),
                    Box::new(Type::Apply(
                        TypeConstructor::Arrow,
                        parameter_types.clone(),
                    )),
                ),
            );
            let tagged_statements = func_body
                .iter()
                .map(|stmt| {
                    translate_statement(
                        &mut local_type_env,
                        variable_env,
                        stmt.as_ref().clone(),
                    )
                })
                .collect::<Result<Vec<_>>>()?;
            let params = parameter_types
                .iter()
                .zip(parameters)
                .map(|(type_, (param_name, _))| (param_name, type_.clone()))
                .collect::<Vec<_>>();
            Ok(TaggedStatement::FunctionDeclaration(
                FunctionDeclaration {
                    name: func_name,
                    params,
                    return_type,
                },
                tagged_statements,
            ))
        }
        Statement::Expression(expression) => {
            Ok(TaggedStatement::Expression(translate_expression(
                type_env,
                variable_env,
                expression.as_ref().clone(),
            )?))
        }
        Statement::Return(maybe_expression) => {
            if let Some(expr) = maybe_expression {
                Ok(TaggedStatement::Return(Some(translate_expression(
                    type_env,
                    variable_env,
                    expr.as_ref().clone(),
                )?)))
            } else {
                Ok(TaggedStatement::Return(None))
            }
        }
        Statement::IfElse(if_expression, body, maybe_else_block) => {
            let tagged_if = Box::new(translate_expression(
                type_env,
                variable_env,
                if_expression.as_ref().clone(),
            )?);
            unify(
                tagged_if.as_ref().clone().into(),
                Type::Apply(TypeConstructor::Bool, Vec::new()),
            )?;
            let tagged_body = body
                .iter()
                .map(|s| {
                    translate_statement(
                        type_env,
                        variable_env,
                        s.as_ref().clone(),
                    )
                })
                .collect::<Result<Vec<_>>>()?;
            if let Some(block) = maybe_else_block {
                let tagged_else = block
                    .iter()
                    .map(|s| {
                        translate_statement(
                            type_env,
                            variable_env,
                            s.as_ref().clone(),
                        )
                    })
                    .collect::<Result<Vec<_>>>()?;
                Ok(TaggedStatement::IfElse(
                    tagged_if,
                    tagged_body,
                    Some(tagged_else),
                ))
            } else {
                Ok(TaggedStatement::IfElse(tagged_if, tagged_body, None))
            }
        }
    }
}

fn translate_expression(
    type_env: &mut TypeEnv,
    variable_env: &mut VariableEnv,
    expression: Expression,
) -> Result<TaggedExpression> {
    match expression {
        Expression::Access(_left_expr, _right_expr) => todo!(),
        Expression::Or(left_expr, right_expr) => {
            let tagged_left = translate_expression(
                type_env,
                variable_env,
                left_expr.as_ref().clone(),
            )?;
            let tagged_right = translate_expression(
                type_env,
                variable_env,
                right_expr.as_ref().clone(),
            )?;
            unify(
                tagged_left.clone().into(),
                Type::Apply(TypeConstructor::Bool, Vec::new()),
            )?;
            unify(tagged_right.clone().into(), tagged_left.clone().into())?;
            Ok(TaggedExpression::Or(
                Box::new(tagged_left),
                Box::new(tagged_right),
            ))
        }
        Expression::And(left_expr, right_expr) => {
            let tagged_left = translate_expression(
                type_env,
                variable_env,
                left_expr.as_ref().clone(),
            )?;
            let tagged_right = translate_expression(
                type_env,
                variable_env,
                right_expr.as_ref().clone(),
            )?;
            unify(
                tagged_left.clone().into(),
                Type::Apply(TypeConstructor::Bool, Vec::new()),
            )?;
            unify(tagged_right.clone().into(), tagged_left.clone().into())?;
            Ok(TaggedExpression::And(
                Box::new(tagged_left),
                Box::new(tagged_right),
            ))
        }
        Expression::Array(mut expressions) => {
            // TODO: Handle starting with an empty array
            let first_expression =
                expressions.first().expect("Cannot initialize empty array");
            let first_tagged_expression = translate_expression(
                type_env,
                variable_env,
                first_expression.as_ref().clone(),
            )?;
            expressions.remove(0);
            let mut all_expressions = expressions
                .iter()
                .map(|e| {
                    let translated_expr = translate_expression(
                        type_env,
                        variable_env,
                        e.as_ref().clone(),
                    )?;
                    unify(
                        first_tagged_expression.clone().into(),
                        translated_expr.clone().into(),
                    )?;
                    Ok(translated_expr)
                })
                .collect::<Result<Vec<_>>>()?;
            all_expressions.push(first_tagged_expression.clone());
            Ok(TaggedExpression::Array(
                all_expressions,
                Box::new(Type::Apply(
                    TypeConstructor::Array,
                    vec![first_tagged_expression.into()],
                )),
            ))
        }
        Expression::Assignment(left_expr, right_expr) => {
            let tagged_left = translate_expression(
                type_env,
                variable_env,
                left_expr.as_ref().clone(),
            )?;
            let tagged_right = translate_expression(
                type_env,
                variable_env,
                right_expr.as_ref().clone(),
            )?;
            unify(tagged_left.clone().into(), tagged_right.clone().into())?;
            Ok(TaggedExpression::Assignment(
                Box::new(tagged_left.clone()),
                Box::new(tagged_right),
                Box::new(tagged_left.into()),
            ))
        }
        Expression::Boolean(b) => Ok(TaggedExpression::Boolean(b)),
        Expression::FunctionCall(ident, generic_type_idents, args) => {
            let instantiated_generics = generic_type_idents
                .iter()
                .map(|i| {
                    translate_type_identifier(
                        type_env,
                        TypeName::Type(i.clone()),
                    )
                })
                .collect::<Result<Vec<_>>>()?;
            let tagged_function_identifier = translate_expression(
                type_env,
                variable_env,
                Expression::Identifier(ident.clone()),
            )?;
            let tagged_argument_expressions = args
                .iter()
                .map(|a| {
                    translate_expression(
                        type_env,
                        variable_env,
                        a.as_ref().clone(),
                    )
                })
                .collect::<Result<Vec<_>>>()?;
            if let Type::PolymorphicType(generics, func_type_constructor) =
                expand(tagged_function_identifier.into())
            {
                let mut all_types: TypeEnv = type_env
                    .iter_mut()
                    .map(|(n, t)| (n.clone(), t.clone()))
                    .collect();
                generics
                    .iter()
                    .cloned()
                    .zip(
                        instantiated_generics
                            .iter()
                            .map(|t| TypeValue::Type(t.clone()))
                            .collect::<Vec<_>>(),
                    )
                    .for_each(|(ident, type_)| {
                        all_types.insert(
                            ident,
                            TypeValue::Type(type_.get_rox_type()),
                        );
                    });

                if let Type::Apply(_constructor, mut types) =
                    *func_type_constructor
                {
                    let return_type = types.pop().unwrap();
                    types
                        .iter()
                        .zip(tagged_argument_expressions.clone())
                        .map(|(t, expr)| {
                            unify(
                                expr.into(),
                                substitute(t.clone(), &mut all_types),
                            )
                        })
                        .collect::<Result<Vec<_>>>()?;
                    let function_return_type =
                        substitute(return_type, &mut all_types);
                    Ok(TaggedExpression::FunctionCall(
                        ident,
                        tagged_argument_expressions,
                        Box::new(function_return_type),
                    ))
                } else {
                    todo!("This is an error but I haven't made it actually useful, sorry!")
                }
            } else {
                todo!("This is an error but I haven't made it actually useful, sorry!")
            }
        }
        Expression::Identifier(x) => Ok(TaggedExpression::Identifier(
            x.clone(),
            Box::new(
                variable_env
                    .get(&x.clone())
                    .ok_or_else(|| {
                        RoxError::with_file_placeholder(
                            format!("Encountered unknown identifier: {}", x)
                                .as_ref(),
                        )
                    })?
                    .clone(),
            ),
        )),
        Expression::String(s) => Ok(TaggedExpression::String(s)),
        Expression::Operation(left, operation, right) => {
            let tagged_left = translate_expression(
                type_env,
                variable_env,
                left.as_ref().clone(),
            )?;
            let tagged_right = translate_expression(
                type_env,
                variable_env,
                right.as_ref().clone(),
            )?;
            Ok(TaggedExpression::Operation(
                Box::new(tagged_left),
                operation,
                Box::new(tagged_right),
            ))
        }
        Expression::Number(n) => Ok(TaggedExpression::Number(n)),
        Expression::Variable(ident, expr) => {
            let expr_value = translate_expression(
                type_env,
                variable_env,
                expr.as_ref().clone(),
            )?;
            variable_env.insert(ident.clone(), expr_value.clone().into());
            Ok(TaggedExpression::Variable(
                ident,
                Box::new(expr_value.clone()),
                Box::new(expr_value.into()),
            ))
        }
        Expression::Unary(unary, expr) => {
            let tagged_expression = translate_expression(
                type_env,
                variable_env,
                expr.as_ref().clone(),
            )?;
            match unary {
                Unary::Not => {
                    unify(
                        tagged_expression.clone().into(),
                        Type::Apply(TypeConstructor::Bool, Vec::new()),
                    )?;
                }
                Unary::Negate => {
                    unify(
                        tagged_expression.clone().into(),
                        Type::Apply(TypeConstructor::Number, Vec::new()),
                    )?;
                }
            }
            Ok(TaggedExpression::Unary(
                unary,
                Box::new(tagged_expression.clone()),
                Box::new(tagged_expression.into()),
            ))
        }
        Expression::ParseError => unreachable!(),
    }
}

fn translate_type_identifier(
    type_env: &mut TypeEnv,
    ty: TypeName,
) -> Result<Type> {
    match ty {
        TypeName::Type(identifier) => {
            Ok(type_env.get(&identifier).unwrap().get_rox_type())
        }
        TypeName::GenericType(identifier, generic_types) => {
            match type_env.get(&identifier).unwrap() {
                TypeValue::Type(t) => Ok(t.clone()), // N.B. This probably shouldn't happen, since formal parameters shouldn't be passed to a concrete type?
                TypeValue::Constructor(c) => Ok(Type::Apply(
                    c.clone(),
                    generic_types
                        .iter()
                        .map(|t| {
                            translate_type_identifier(
                                type_env,
                                t.as_ref().clone(),
                            )
                        })
                        .collect::<Result<Vec<_>>>()?,
                )),
            }
        }
        TypeName::Function(mut argument_types, return_type) => {
            argument_types.push(return_type);
            Ok(Type::Apply(
                TypeConstructor::Arrow,
                argument_types
                    .iter()
                    .map(|a| {
                        translate_type_identifier(
                            type_env,
                            (*a).as_ref().clone(),
                        )
                    })
                    .collect::<Result<Vec<_>>>()?,
            ))
        }
    }
}

pub(crate) fn analyse_program(
    declarations: Vec<Declaration>,
) -> Result<Vec<TaggedDeclaration>> {
    let (mut type_env, mut variable_env, _) = builtins::get_builtin_types();
    declarations
        .iter()
        .map(|d| translate_declaration(&mut type_env, &mut variable_env, d))
        .collect()
}

#[cfg(test)]
mod tests {}
