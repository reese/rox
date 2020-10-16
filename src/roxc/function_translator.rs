use crate::roxc::compiler_state::CompilerState;
use crate::roxc::{
    FunctionDeclaration, Identifier, TaggedExpression, TaggedStatement,
};
use inkwell::basic_block::BasicBlock;
use inkwell::types::{BasicType, BasicTypeEnum};
use inkwell::values::{BasicValueEnum, PointerValue};
use std::borrow::Borrow;
use std::collections::HashMap;

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
            TaggedStatement::StructDeclaration => {},
            TaggedStatement::Expression(expression) => {
                self.translate_expression(expression);
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
            },
            TaggedStatement::Return(maybe_expression) => {
                if let Some(expression) = maybe_expression {
                    if let Some(return_) = self.translate_expression(expression) {
                        self.current_state.build_return(Some(&return_));
                    }
                    else {
                        self.current_state.build_return(None);
                    }
                } else {
                    self.current_state.build_return(None);
                }
            }
            TaggedStatement::IfElse(conditional, if_statements, else_statements_maybe) => {
                let if_block = self.current_state.append_basic_block("if");
                let else_block = self.current_state.append_basic_block("else");
                let merge_block = self.current_state.append_basic_block("continue");
                let conditional_value = self.translate_expression(conditional).expect("Cannot evaluate condition with void value").into_float_value();
                self.current_state.build_conditional(conditional_value, "ifcond", if_block, else_block);

                self.read_into_block(Some(if_statements.clone()), if_block, merge_block);
                self.read_into_block(else_statements_maybe.clone(), else_block, merge_block);

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
        expression: &TaggedExpression,
    ) -> Option<BasicValueEnum<'ctx>> {
        match expression {
            TaggedExpression::Boolean(bool) => {
                Some(self.current_state.bool_literal(*bool))
            }
            TaggedExpression::FunctionCall(function_name, args, _rox_type) => {
                if let Some(function) =
                    self.current_state.get_function(function_name)
                {
                    let argument_values: Vec<BasicValueEnum<'ctx>> = args
                        .iter()
                        .map(|arg| {
                            self.translate_expression(arg).expect(
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
            TaggedExpression::Number(num) => {
                Some(self.current_state.number_literal(*num))
            }
            // TODO: We should consider renaming Array to Vector (for all types), since it's not technically an array
            TaggedExpression::Array(tagged_expressions, type_) => {
                let expression_values = tagged_expressions
                    .iter()
                    .map(|t| {
                        self.translate_expression(t)
                            .expect("Cannot create array from void value")
                            .into_array_value()
                    })
                    .collect::<Vec<_>>();
                let llvm_type: BasicTypeEnum = CompilerState::get_type(
                    self.current_state.get_context(),
                    type_.as_ref(),
                )
                .expect("Unexpected void expression type");
                Some(
                    llvm_type
                        .array_type(expression_values.len() as u32)
                        .const_array(expression_values.as_slice())
                        .into(),
                )
            }
            TaggedExpression::String(string) => {
                Some(self.current_state.string_literal(string))
            }
            TaggedExpression::Variable(name, expression, _type_) => {
                let value: BasicValueEnum<'ctx> = self
                    .translate_expression(expression)
                    .expect("Cannot define variable with void expression");
                let allocation = self.current_state.store_variable(name, value);
                self.variables.insert(name.clone(), allocation);
                Some(value)
            }
            TaggedExpression::Identifier(name, _rox_type) => {
                let variable =
                    self.variables.get(name).expect("Variable not defined");
                Some(self.current_state.load_variable(*variable, name))
            }
            TaggedExpression::Operation(left, operation, right) => {
                let lval = self
                    .translate_expression(left)
                    .expect("Cannot perform operation on void value")
                    .into_float_value();
                let rval = self
                    .translate_expression(right)
                    .expect("Cannot perform operation on void value")
                    .into_float_value();
                Some(self.current_state.build_operation(lval, rval, operation))
            }
            TaggedExpression::StructInstantiation(_struct_type, _fields) => {
                todo!()
            }
            x => unimplemented!("{:?}", x),
        }
    }
}
