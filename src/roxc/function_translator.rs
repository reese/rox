use crate::roxc::semant;
use crate::roxc::{
    get_cranelift_type, parser, FunctionDeclaration, Identifier, Stack,
    TaggedExpression, TaggedStatement,
};
use cranelift::prelude::*;
use cranelift_module::{Backend, DataContext, Linkage, Module, ModuleError};
use std::borrow::Borrow;
use std::collections::HashMap;

pub struct FunctionTranslator<'func, T: Backend> {
    builder: &'func mut FunctionBuilder<'func>,
    data_context: &'func mut DataContext,
    pub variables: &'func mut Stack<HashMap<Identifier, Variable>>,
    pub functions: &'func mut Stack<HashMap<Identifier, FunctionDeclaration>>,
    pub module: &'func mut Module<T>,
}

impl<'func, T: Backend> FunctionTranslator<'func, T> {
    pub fn new(
        builder: &'func mut FunctionBuilder<'func>,
        data_context: &'func mut DataContext,
        variables: &'func mut Stack<HashMap<Identifier, Variable>>,
        functions: &'func mut Stack<HashMap<Identifier, FunctionDeclaration>>,
        module: &'func mut Module<T>,
    ) -> Self {
        FunctionTranslator {
            builder,
            data_context,
            variables,
            functions,
            module,
        }
    }

    pub(crate) fn translate_function(
        &mut self,
        params: &[(Identifier, semant::Type)],
        block: &[TaggedStatement],
    ) {
        self.initialize_block(params);
        self.translate_block(block);

        // Add return if block doesn't have a return statement
        if !block.iter().any(|statement| match *statement {
            TaggedStatement::Return(_) => true,
            _ => false,
        }) {
            self.builder.ins().return_(&[]);
        }

        self.builder.finalize();
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
                    let returns = self.translate_expression(expression);
                    self.builder.ins().return_(&returns);
                } else {
                    self.builder.ins().return_(&[]);
                }
            }
            TaggedStatement::IfElse(conditional, if_statements, else_statements_maybe) => {
                let if_block = self.builder.create_block();
                let else_block = self.builder.create_block();
                let merge_block = self.builder.create_block();

                let conditional_value = self.translate_expression(conditional)[0];

                self.builder.ins().brz(conditional_value, else_block, &[]);
                self.builder.ins().jump(if_block, &[]);

                self.read_into_block(Some(if_statements.clone()), if_block, merge_block);
                self.read_into_block(else_statements_maybe.clone(), else_block, merge_block);

                self.builder.switch_to_block(merge_block);
                self.builder.seal_block(merge_block);
            }
        }
    }

    #[allow(clippy::vec_box)]
    fn read_into_block(
        &mut self,
        maybe_statements: Option<Vec<TaggedStatement>>,
        conditional_block: Block,
        merge_block: Block,
    ) {
        let mut has_return = false;
        self.builder.switch_to_block(conditional_block);
        self.builder.seal_block(conditional_block);
        if let Some(if_statements) = maybe_statements {
            if_statements.iter().for_each(|statement| {
                if let TaggedStatement::Return(_) = statement {
                    has_return = true;
                }
                self.translate_statement(statement);
            });
        }
        if !has_return {
            self.builder.ins().jump(merge_block, &[]);
        }
    }

    pub fn translate_expression(
        &mut self,
        expression: &TaggedExpression,
    ) -> Vec<Value> {
        match expression {
            TaggedExpression::Boolean(bool) => {
                vec![self.builder.ins().bconst(types::B1, *bool)]
            }
            TaggedExpression::FunctionCall(function_name, args, _rox_type) => {
                let FunctionDeclaration {
                    return_type,
                    params,
                    ..
                } = self.functions.top().get(function_name).unwrap();

                let mut signature = self.module.make_signature();
                params.iter().for_each(|(_, type_name)| {
                    signature.params.push(AbiParam::new(get_cranelift_type(
                        type_name,
                        self.pointer_type(),
                    )));
                });
                let codegen_type =
                    get_cranelift_type(return_type, self.pointer_type());
                if codegen_type != types::INVALID {
                    signature.returns.push(AbiParam::new(codegen_type));
                }

                let callee = self
                    .module
                    .declare_function(
                        function_name.clone().as_str(),
                        Linkage::Import,
                        &signature,
                    )
                    .unwrap();
                let local_callee = self
                    .module
                    .declare_func_in_func(callee, &mut self.builder.func);

                let argument_values: Vec<Value> = args
                    .iter()
                    .map(|arg| *self.translate_expression(arg).get(0).unwrap())
                    .collect();
                let call =
                    self.builder.ins().call(local_callee, &argument_values);
                let returns = self.builder.inst_results(call); // TODO: Support multiple returns
                if !returns.is_empty() {
                    vec![returns[0]]
                } else {
                    returns.to_vec()
                }
            }

            TaggedExpression::Number(num) => {
                vec![self.builder.ins().f64const(*num)]
            }
            TaggedExpression::Array(tagged_expressions, _type_) => {
                self.data_context.define_zeroinit(
                    (get_cranelift_type(
                        &tagged_expressions.first().unwrap().clone().into(),
                        self.pointer_type(),
                    )
                    .bytes()
                        * (tagged_expressions.len() as u32))
                        as usize,
                );
                let data_id = self
                    .module
                    .declare_data(
                        format!("{:?}", tagged_expressions).as_ref(), // TODO: Surely there's a better way to do this?
                        Linkage::Export,
                        true,
                        false,
                        None,
                    )
                    .unwrap();
                self.module
                    .define_data(data_id, &self.data_context)
                    .unwrap();
                let value = self
                    .module
                    .declare_data_in_func(data_id, self.builder.func);
                self.data_context.clear();
                self.module.finalize_definitions();
                let pointer_type = self.pointer_type();
                vec![self.builder.ins().global_value(pointer_type, value)]
            }
            TaggedExpression::String(string) => {
                self.define_null_terminated_string(string);
                let id = self
                    .module
                    .declare_data(
                        string.as_ref(),
                        Linkage::Export,
                        false,
                        false,
                        None,
                    )
                    .unwrap();
                match self.module.define_data(id, &self.data_context) {
                    Ok(_) => Ok(()),
                    Err(error) => match error {
                        ModuleError::DuplicateDefinition(_) => Ok(()),
                        err => Err(err),
                    },
                }
                .expect("Could not define string in module");
                let value =
                    self.module.declare_data_in_func(id, self.builder.func);
                self.data_context.clear();
                self.module.finalize_definitions();
                let pointer_type = self.pointer_type();

                vec![self.builder.ins().global_value(pointer_type, value)]
            }
            TaggedExpression::Variable(name, expression, type_) => {
                let value = self.translate_expression(expression)[0];
                let variable_env = self.variables.top_mut();
                let variable =
                    cranelift::prelude::Variable::new(variable_env.len());
                variable_env.insert(name.clone(), variable);
                self.builder.declare_var(
                    variable,
                    get_cranelift_type(type_, self.pointer_type()),
                );
                self.builder.def_var(variable, value);
                vec![value]
            }
            TaggedExpression::Identifier(name, _rox_type) => {
                let variables = self.variables.top();
                let variable =
                    variables.get(name).expect("Variable not defined");
                vec![self.builder.use_var(*variable)]
            }
            TaggedExpression::Operation(left, operation, right) => {
                use parser::Operation::*;
                let lval = self.translate_expression(left)[0];
                let rval = self.translate_expression(right)[0];
                let result = match operation {
                    Add => self.builder.ins().fadd(lval, rval),
                    Subtract => self.builder.ins().fsub(lval, rval),
                    Multiply => self.builder.ins().fmul(lval, rval),
                    Divide => self.builder.ins().fdiv(lval, rval),
                    Equals => {
                        self.builder.ins().fcmp(FloatCC::Equal, lval, rval)
                    }
                    NotEquals => {
                        self.builder.ins().fcmp(FloatCC::NotEqual, lval, rval)
                    }
                    GreaterThan => self.builder.ins().fcmp(
                        FloatCC::GreaterThan,
                        lval,
                        rval,
                    ),
                    LessThan => {
                        self.builder.ins().fcmp(FloatCC::LessThan, lval, rval)
                    }
                };
                vec![result]
            }
            TaggedExpression::StructInstantiation(_struct_type, _fields) => {
                todo!()
            }
            x => unimplemented!("{:?}", x),
        }
    }

    fn initialize_block(&mut self, params: &[(Identifier, semant::Type)]) {
        let entry_block = self.builder.create_block();
        self.builder
            .append_block_params_for_function_params(entry_block);
        self.builder.switch_to_block(entry_block);
        self.builder.seal_block(entry_block);
        let block_params = self.builder.block_params(entry_block).to_vec();
        block_params.iter().enumerate().for_each(|(index, param)| {
            let (name, type_) = params.get(index).unwrap().clone();
            let variable = Variable::new(index);
            self.variables.top_mut().insert(name, variable);
            self.builder.declare_var(
                variable,
                get_cranelift_type(&type_, self.pointer_type()),
            );
            self.builder.def_var(variable, *param);
        });
    }

    /// Note that reading a string into bytes with `string.into_bytes()`
    /// does _not_ include the null terminator. If we don't add it
    /// here, multiple strings co-located in the same function
    /// will read together as one giant string since we
    /// store the pointer, not the actual string
    fn define_null_terminated_string(&mut self, original_string: &str) {
        let mut null_terminated_string = original_string.to_string();
        null_terminated_string.push('\0');
        self.data_context
            .define(null_terminated_string.into_bytes().into_boxed_slice());
    }

    fn pointer_type(&self) -> Type {
        self.module.target_config().pointer_type()
    }
}
