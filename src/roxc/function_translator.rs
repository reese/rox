use crate::roxc::{create_entry_block_allocation, semant};
use crate::roxc::{
    get_cranelift_type, parser, FunctionDeclaration, Identifier, Result, Stack,
    TaggedExpression, TaggedStatement,
};
use inkwell::basic_block::BasicBlock;
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::types::{ArrayType, VectorType, VoidType};
use inkwell::values::{
    BasicValue, BasicValueEnum, FloatValue, FunctionValue, PointerValue,
};
use inkwell::FloatPredicate;
use std::borrow::Borrow;
use std::collections::HashMap;

pub struct FunctionTranslator<'func> {
    builder: &'func mut Builder<'func>,
    context: &'func mut Context,
    function: &'func mut FunctionValue<'func>,
    pub variables: &'func mut Stack<HashMap<Identifier, PointerValue<'func>>>,
    pub functions: &'func mut Stack<HashMap<Identifier, FunctionDeclaration>>,
    pub module: &'func mut Module<'func>,
}

impl<'func> FunctionTranslator<'func> {
    pub fn new(
        builder: &'func mut Builder<'func>,
        context: &'func mut Context,
        function: &'func mut FunctionValue<'func>,
        variables: &'func mut Stack<HashMap<Identifier, PointerValue<'func>>>,
        functions: &'func mut Stack<HashMap<Identifier, FunctionDeclaration>>,
        module: &'func mut Module<'func>,
    ) -> Self {
        FunctionTranslator {
            builder,
            context,
            function,
            variables,
            functions,
            module,
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
                self.functions.top_mut().insert(decl.name.clone(), decl.clone());
            },
            TaggedStatement::Return(maybe_expression) => {
                if let Some(expression) = maybe_expression {
                    let return_ = self.translate_expression(expression);
                    self.builder.build_return(Some(&return_));
                } else {
                    self.builder.build_return(None);
                }
            }
            TaggedStatement::IfElse(conditional, if_statements, else_statements_maybe) => {
                let if_block = self.context.append_basic_block(*self.function, "if");
                let else_block = self.context.append_basic_block(*self.function, "else");
                let merge_block = self.context.append_basic_block(*self.function, "continue");

                let conditional_value = self.translate_expression(conditional).into_float_value();
                let zero_const = self.context.f64_type().const_zero();
                let conditional_value = self.builder.build_float_compare(FloatPredicate::ONE, conditional_value, zero_const, "ifcond");

                self.builder.build_conditional_branch(conditional_value, if_block, else_block);

                self.read_into_block(Some(if_statements.clone()), if_block, merge_block);
                self.read_into_block(else_statements_maybe.clone(), else_block, merge_block);

                self.builder.position_at_end(merge_block);
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
    ) -> BasicBlock<'func> {
        self.builder.position_at_end(conditional_block);
        if let Some(statements) = maybe_statements {
            self.translate_block(statements.as_slice());
        }
        self.builder.build_unconditional_branch(merge_block);

        self.builder.get_insert_block().unwrap()
    }

    pub fn translate_expression(
        &mut self,
        expression: &TaggedExpression,
    ) -> BasicValueEnum {
        match expression {
            TaggedExpression::Boolean(bool) => self
                .context
                .bool_type()
                .const_int(*bool as u64, false)
                .into(),
            TaggedExpression::FunctionCall(function_name, args, _rox_type) => {
                if let Some(function) = self.module.get_function(function_name)
                {
                    let argument_values = args
                        .iter()
                        .map(|arg| self.translate_expression(arg).clone())
                        .collect::<Vec<_>>();
                    let argument_types = argument_values
                        .iter()
                        .by_ref()
                        .map(|&val| val.into())
                        .collect::<Vec<_>>();

                    match self
                        .builder
                        .build_call(function, argument_types.as_slice(), "tmp")
                        .try_as_basic_value()
                        .left()
                    {
                        Some(value) => value.into(),
                        None => todo!("Handle returning void"),
                    }
                } else {
                    panic!("Attempted to build a function not in this module.")
                }
            }
            TaggedExpression::Number(num) => {
                self.context.f64_type().const_float(*num).into()
            }
            // TODO: We should consider renaming Array to Vector (for all types), since it's not technically an array
            TaggedExpression::Array(tagged_expressions, type_) => {
                let expression_values = tagged_expressions
                    .iter()
                    .map(|e| self.translate_expression(t))
                    .collect::<Vec<_>>();
                VectorType::const_vector(expression_values.as_slice()).into()
            }
            TaggedExpression::String(string) => {
                self.context.const_string(string.as_bytes(), false).into()
            }
            TaggedExpression::Variable(name, expression, type_) => {
                let value = self.translate_expression(expression);
                let allocation = create_entry_block_allocation(
                    self.builder,
                    name,
                    self.function,
                    value.get_type(),
                );
                self.builder.build_store(allocation, value);
                let variable_env = self.variables.top_mut();
                variable_env.insert(name.clone(), value.into_pointer_value());
                value
            }
            TaggedExpression::Identifier(name, _rox_type) => {
                let variables = self.variables.top();
                let variable =
                    variables.get(name).expect("Variable not defined");
                self.builder.build_load(*variable, name.as_str())
            }
            TaggedExpression::Operation(left, operation, right) => {
                use parser::Operation::*;
                let lval = self.translate_expression(left).into_float_value();
                let rval = self.translate_expression(right).into_float_value();
                match operation {
                    Add => self
                        .builder
                        .build_float_add(lval, rval, "tmpadd")
                        .into(),
                    Subtract => self
                        .builder
                        .build_float_sub(lval, rval, "tmpsub")
                        .into(),
                    Multiply => self
                        .builder
                        .build_float_mul(lval, rval, "tmpmul")
                        .into(),
                    Divide => self
                        .builder
                        .build_float_div(lval, rval, "tmpdiv")
                        .into(),
                    Equals => {
                        self.binary_comparison(FloatPredicate::OEQ, lval, rval)
                    }
                    NotEquals => {
                        self.binary_comparison(FloatPredicate::ONE, lval, rval)
                    }
                    GreaterThan => {
                        self.binary_comparison(FloatPredicate::OGT, lval, rval)
                    }
                    LessThan => {
                        self.binary_comparison(FloatPredicate::OLT, lval, rval)
                    }
                }
            }
            TaggedExpression::StructInstantiation(_struct_type, _fields) => {
                todo!()
            }
            x => unimplemented!("{:?}", x),
        }
    }

    fn binary_comparison(
        &self,
        predicate: FloatPredicate,
        lval: FloatValue,
        rval: FloatValue,
    ) -> BasicValueEnum {
        let comparison = self.builder.build_float_compare(
            predicate,
            lval.clone(),
            rval.clone(),
            "tmpcmp",
        );
        self.builder
            .build_unsigned_int_to_float(
                comparison,
                self.context.f64_type(),
                "tmpbool",
            )
            .into()
    }
}
