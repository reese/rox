use crate::roxc::{
    syntax, Expression, FunctionDeclaration, Param, RoxType, Stack, Statement,
};
use cranelift::prelude::*;
use cranelift_module::{Linkage, Module};
use cranelift_object::ObjectBackend;
use im::HashMap;
use std::borrow::Borrow;

pub struct FunctionTranslator<'func> {
    builder: &'func mut FunctionBuilder<'func>,
    pub variables: &'func mut Stack<HashMap<String, Variable>>,
    pub functions: &'func mut Stack<HashMap<String, FunctionDeclaration>>,
    pub module: &'func mut Module<ObjectBackend>,
}

impl<'func> FunctionTranslator<'func> {
    pub fn new(
        builder: &'func mut FunctionBuilder<'func>,
        variables: &'func mut Stack<HashMap<String, Variable>>,
        functions: &'func mut Stack<HashMap<String, FunctionDeclaration>>,
        module: &'func mut Module<ObjectBackend>,
    ) -> Self {
        FunctionTranslator {
            builder,
            variables,
            functions,
            module,
        }
    }

    pub fn translate_function(
        &mut self,
        params: &[Param],
        return_type: &Option<String>,
        block: &Vec<Box<Statement>>,
    ) {
        self.initialize_block(params);
        self.translate_block(block);

        if return_type.is_none() {
            dbg!(return_type.clone());
            self.builder.ins().return_(&[]);
        }
        self.builder.finalize();
    }

    fn translate_block(&mut self, block: &[Box<Statement>]) {
        block.iter().for_each(|statement| {
            self.translate_statement(statement);
        })
    }

    fn translate_statement(&mut self, statement: &Box<Statement>) {
        match statement.borrow() {
            Statement::Expression(expression) => {
                self.translate_expression(expression);
            }
            Statement::FunctionDeclaration(..) => {
                panic!("For right now, functions can only be declared at the top level.")
            }
            Statement::Return(maybe_expression) => {
                if let Some(expression) = maybe_expression {
                    let returns = self.translate_expression(expression);
                    self.builder.ins().return_(&returns);
                } else {
                    self.builder.ins().return_(&[]);
                }
            }
            _ => {}
        }
    }

    pub fn translate_expression(
        &mut self,
        expression: &Expression,
    ) -> Vec<Value> {
        use Expression::*;

        match expression {
            Boolean(bool) => vec![self.builder.ins().bconst(types::B1, *bool)],
            FunctionCall(function_name, args) => {
                let FunctionDeclaration {
                    return_type,
                    params,
                    ..
                } = self.functions.top().get(function_name).unwrap();

                let mut signature = self.module.make_signature();
                params.iter().for_each(|(_, type_name)| {
                    signature
                        .params
                        .push(AbiParam::new(get_type_from_name(type_name)));
                });
                if let Some(return_) = return_type {
                    signature
                        .returns
                        .push(AbiParam::new(get_type_from_name(return_)));
                }

                let callee = self
                    .module
                    .declare_function(
                        &function_name,
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
            Number(num) => vec![self.builder.ins().f64const(*num)],
            Variable(name, expression) => {
                let expression = self.translate_expression(expression)[0];
                let variable_env = self.variables.top_mut();
                let variable =
                    cranelift::prelude::Variable::new(variable_env.len());
                variable_env.insert(name.clone(), variable);
                self.builder.declare_var(variable, types::F64); // TODO: Map ArenaTypes to concrete types
                self.builder.def_var(variable, expression);
                vec![expression]
            }
            Identifier(name) => {
                let variables = self.variables.top();
                let variable =
                    variables.get(name).expect("Variable not defined");
                vec![self.builder.use_var(*variable)]
            }
            Operation(left, operation, right) => {
                use syntax::Operation::*;
                let lval = self.translate_expression(left)[0];
                let rval = self.translate_expression(right)[0];
                let result = match operation {
                    Add => self.builder.ins().fadd(lval, rval),
                    Subtract => self.builder.ins().fsub(lval, rval),
                    Multiply => self.builder.ins().fmul(lval, rval),
                    Divide => self.builder.ins().fdiv(lval, rval),
                    Equals => {
                        let bool =
                            self.builder.ins().fcmp(FloatCC::Equal, lval, rval);
                        self.builder.ins().bint(types::B1, bool)
                    }
                    NotEquals => {
                        let bool = self.builder.ins().fcmp(
                            FloatCC::NotEqual,
                            lval,
                            rval,
                        );
                        self.builder.ins().bint(types::B1, bool)
                    }
                    GreaterThan => {
                        let bool = self.builder.ins().fcmp(
                            FloatCC::GreaterThan,
                            lval,
                            rval,
                        );
                        self.builder.ins().bint(types::B1, bool)
                    }
                    LessThan => {
                        let bool = self.builder.ins().fcmp(
                            FloatCC::LessThan,
                            lval,
                            rval,
                        );
                        self.builder.ins().bint(types::B1, bool)
                    }
                };
                vec![result]
            }
            x => {
                dbg!(x);
                unimplemented!()
            }
        }
    }

    fn initialize_block(&mut self, params: &[Param]) {
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
            self.builder
                .declare_var(variable, get_type_from_name(&type_));
            self.builder.def_var(variable, *param);
        });
    }
}

pub(crate) fn get_type_from_name(type_str: &str) -> Type {
    let rox_type = match type_str {
        "bool" => RoxType::Bool,
        "number" => RoxType::Number,
        "str" => RoxType::String,
        x => {
            dbg!(x);
            unimplemented!()
        }
    };
    get_codegen_type(&rox_type)
}

fn get_codegen_type(rox_type: &RoxType) -> types::Type {
    match rox_type {
        RoxType::Bool => types::B1,
        RoxType::Number => types::F64,
        _ => unimplemented!(),
    }
}
