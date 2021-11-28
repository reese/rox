use crate::roxc::compiler_state::CompilerState;
use crate::roxc::{
    FunctionDeclaration, Identifier, TaggedExpression, TaggedStatement,
};
use inkwell::types::BasicTypeEnum;
use inkwell::values::{BasicValueEnum, PointerValue};
use inkwell::{basic_block::BasicBlock, values::IntValue};
use std::borrow::Borrow;
use std::collections::HashMap;

use super::{TaggedLValue, Type, TypeConstructor};

pub struct FunctionTranslator<'func, 'context> {
    current_state: &'func CompilerState<'func, 'context>,
    pub variables: &'func mut HashMap<Identifier, PointerValue<'context>>,
    pub functions: &'func mut HashMap<Identifier, FunctionDeclaration>,
}

impl<'func, 'ctx> FunctionTranslator<'func, 'ctx> {
    pub fn new(
        current_state: &'func CompilerState<'func, 'ctx>,
        variables: &'func mut HashMap<Identifier, PointerValue<'ctx>>,
        functions: &'func mut HashMap<Identifier, FunctionDeclaration>,
    ) -> Self {
        FunctionTranslator {
            current_state,
            variables,
            functions,
        }
    }

    pub(crate) fn translate_function(&mut self, block: &[TaggedStatement]) {
        self.translate_block(block);
    }

    fn translate_block(&mut self, block: &[TaggedStatement]) {
        block.iter().for_each(|statement| {
            self.translate_statement(statement);
        })
    }

    fn translate_statement(&mut self, statement: &TaggedStatement) {
        match statement.borrow() {
            TaggedStatement::StructDeclaration => {}
            TaggedStatement::Expression(expression) => {
                self.translate_expression(expression.to_owned());
            }
            TaggedStatement::FunctionDeclaration(..) => {
                panic!("For right now, functions can only be declared at the top level.")
            }
            // The `extern` tag merely declares the function to the type checker
            // The linker will then try to dynamically link the function call
            // if one exists. For the most part, we use this as a way to use
            // `libc` functions, but this could potentially be used to link a
            // Rust runtime library, but that's still undetermined.
            TaggedStatement::ExternFunctionDeclaration(decl) => {
                self.functions.insert(decl.name.clone(), decl.clone());
            }
            TaggedStatement::Return(maybe_expression) => {
                if let Some(expression) = maybe_expression {
                    if let Some(return_) = self.translate_expression(expression.to_owned())
                    {
                        self.current_state.build_return(Some(&return_));
                    } else {
                        self.current_state.build_return(None);
                    }
                } else {
                    self.current_state.build_return(None);
                }
            }
            TaggedStatement::IfElse(
                conditional,
                if_statements,
                else_statements_maybe,
            ) => {
                let if_block = self.current_state.append_basic_block("if");
                let else_block = self.current_state.append_basic_block("else");
                let merge_block =
                    self.current_state.append_basic_block("continue");
                let conditional_value = self
                    .translate_expression(conditional.as_ref().to_owned())
                    .expect("Cannot evaluate condition with void value")
                    .into_float_value();
                self.current_state.build_conditional(
                    conditional_value,
                    "ifcond",
                    if_block,
                    else_block,
                );

                self.read_into_block(
                    Some(if_statements.clone()),
                    if_block,
                    merge_block,
                );
                self.read_into_block(
                    else_statements_maybe.clone(),
                    else_block,
                    merge_block,
                );

                self.current_state.position_at_end(merge_block);
                // N.B. I left out the `phi` value since I don't intend
                // to return anything from these values, but that may
                // change in the future
            }
        }
    }

    fn read_into_block(
        &mut self,
        maybe_statements: Option<Vec<TaggedStatement>>,
        conditional_block: BasicBlock,
        merge_block: BasicBlock,
    ) {
        self.current_state.position_at_end(conditional_block);
        if let Some(statements) = maybe_statements {
            self.translate_block(statements.as_slice());
        }
        self.current_state.build_fallback_branch(merge_block);
    }

    pub fn translate_expression(
        &mut self,
        expression: TaggedExpression,
    ) -> Option<BasicValueEnum<'ctx>> {
        match expression {
            TaggedExpression::Boolean(bool) => {
                Some(self.current_state.bool_literal(bool))
            }
            TaggedExpression::FunctionCall(function_name, args, _rox_type) => {
                if let Some(function) =
                    self.current_state.get_function(&function_name.value)
                {
                    let argument_values: Vec<BasicValueEnum<'ctx>> = args
                        .iter()
                        .map(|arg| {
                            self.translate_expression(arg.to_owned()).expect(
                                "Cannot pass void expression as argument",
                            )
                        })
                        .collect::<Vec<_>>();

                    self.current_state
                        .function_call(function, argument_values.as_slice())
                } else {
                    panic!("Attempted to build a function not in this module.")
                }
            }
            TaggedExpression::Int(number) => {
                Some(self.current_state.int_literal(number.value))
            }
            TaggedExpression::Float(num) => {
                Some(self.current_state.float_literal(num.value))
            }
            TaggedExpression::Array(tagged_expressions, type_) => {
                let array_pointer = self.create_array(
                    tagged_expressions,
                    type_.as_ref().to_owned(),
                );

                Some(self.current_state.build_load(array_pointer))
            }
            TaggedExpression::String(string) => {
                Some(self.current_state.string_literal(&string.value))
            }
            TaggedExpression::Variable(name, expression, _type_) => {
                let value: BasicValueEnum<'ctx> = self
                    .translate_expression(expression.as_ref().to_owned())
                    .expect("Cannot define variable with void expression");
                let allocation =
                    self.current_state.store_variable(&name.value, value);
                self.variables.insert(name.value.clone(), allocation);
                Some(value)
            }
            TaggedExpression::Identifier(name, _rox_type) => {
                let variable = self
                    .variables
                    .get(&name.value)
                    .expect("Variable not defined");
                Some((*variable).into())
            }
            TaggedExpression::Operation(lval, operation, rval, rox_type) => {
                let left = self
                    .translate_expression(lval.as_ref().to_owned())
                    .expect("Cannot perform operation on void value");
                let right = self
                    .translate_expression(rval.as_ref().to_owned())
                    .expect("Cannot perform operation on void value");
                match rox_type.as_ref() {
                    Type::Apply(constructor, _) => match constructor {
                        TypeConstructor::Float => {
                            let left = left.into_float_value();
                            let right = right.into_float_value();
                            Some(self.current_state.build_float_operation(
                                left,
                                right,
                                &operation.value,
                            ))
                        }
                        TypeConstructor::Int => {
                            let left = left.into_int_value();
                            let right = right.into_int_value();
                            Some(self.current_state.build_int_operation(
                                left,
                                right,
                                &operation.value,
                            ))
                        }
                        _ => unreachable!(),
                    },
                    Type::Variable(_) | Type::PolymorphicType(_, _) => {
                        unreachable!()
                    }
                }
            }
            TaggedExpression::BracketAccess(
                array_value,
                index_value,
                _inner_array_type,
            ) => {
                let lval_expr = self
                    .translate_expression(array_value.as_ref().to_owned())
                    .unwrap()
                    .into_pointer_value();
                let index_value = self
                    .translate_expression(index_value.as_ref().to_owned())
                    .unwrap()
                    .into_int_value();
                let value_pointer =
                    dbg!(self.index_array(lval_expr, index_value));
                Some(self.current_state.build_load(value_pointer))
            }
            TaggedExpression::Assignment(lval, value_expr, _rox_type) => {
                let rval = self
                    .translate_expression(*value_expr)
                    .expect("Cannot assign Void to variable");
                let pointer = self.translate_lvalue(lval.as_ref().to_owned());

                self.current_state.build_store(pointer, rval);
                Some(rval)
            }
            TaggedExpression::StructInstantiation(_, _)
            | TaggedExpression::And(_, _)
            | TaggedExpression::Or(_, _)
            | TaggedExpression::Unary(_, _, _) => todo!(),
        }
    }

    fn translate_lvalue(&mut self, lval: TaggedLValue) -> PointerValue<'ctx> {
        match lval.0 {
            TaggedExpression::BracketAccess(array_value, index_expr, _type) => {
                let array_pointer = self.translate_lvalue(TaggedLValue(array_value.as_ref().to_owned()));
                let index = self.translate_expression(index_expr.as_ref().to_owned()).unwrap().into_int_value();
                self.index_array(array_pointer, index)
            },
            TaggedExpression::FunctionCall(..) => {
                // Note for future @reese -- is this actually a correct assumption?
                // i.e. can we confidentally assert that return arrays/structs from a function
                // works correctly this way?
                self.translate_expression(lval.0).unwrap().into_pointer_value()
            },
            TaggedExpression::Identifier(ident_span, _) => {
                *self.variables.get(&ident_span.value).unwrap()
            },
            TaggedExpression::Array(values, inner_type) => {
                self.create_array(values, inner_type.as_ref().to_owned())
            },
            TaggedExpression::And(_, _) |
            TaggedExpression::Boolean(_) |
            TaggedExpression::Float(_) |
            TaggedExpression::Int(_) |
            TaggedExpression::Operation(_, _, _, _) |
            TaggedExpression::Or(_, _) |
            TaggedExpression::String(_) |
            TaggedExpression::StructInstantiation(_, _) |
            TaggedExpression::Unary(_, _, _) |
            TaggedExpression::Assignment(..) |
            TaggedExpression::Variable(_, _, _) => {
                unreachable!("Values cannot be assigned to this expression ({:?}) and should have caused errors during parsing or typechecking.", lval.0) }
        }
    }

    fn create_array(
        &mut self,
        tagged_expressions: Vec<TaggedExpression>,
        inner_type: Type,
    ) -> PointerValue<'ctx> {
        let expression_values = tagged_expressions
            .iter()
            .map(|t| {
                self.translate_expression(t.to_owned())
                    .expect("Cannot create array from void value")
            })
            .collect::<Vec<_>>();
        let llvm_type: BasicTypeEnum = CompilerState::get_type(
            self.current_state.get_context(),
            &inner_type,
            self.variables,
            Some(expression_values.len()),
        )
        .expect("Unexpected void expression type");

        self.current_state.build_array_allocation_with_values(
            expression_values.as_slice(),
            llvm_type,
        )
    }

    fn index_array(
        &mut self,
        lval_expr: PointerValue<'ctx>,
        index_value: IntValue<'ctx>,
    ) -> PointerValue<'ctx> {
        unsafe {
            self.current_state
                .build_array_access(lval_expr, index_value)
        }
    }
}
