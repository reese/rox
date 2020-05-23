use crate::roxc::{Expression, Operation, Param, RoxType, Stack, Statement};
use cranelift::prelude::*;
use cranelift_module::{Linkage, Module};
use cranelift_object::ObjectBackend;
use im::HashMap;
use std::borrow::Borrow;

pub struct FunctionTranslator<'func> {
    builder: &'func mut FunctionBuilder<'func>,
    pub variables: &'func mut Stack<HashMap<String, Variable>>,
    pub module: &'func mut Module<ObjectBackend>,
}

impl<'func> FunctionTranslator<'func> {
    pub fn new(
        builder: &'func mut FunctionBuilder<'func>,
        variables: &'func mut Stack<HashMap<String, Variable>>,
        module: &'func mut Module<ObjectBackend>,
    ) -> Self {
        FunctionTranslator {
            builder,
            variables,
            module,
        }
    }

    pub fn translate_function(
        &mut self,
        params: &[Param],
        _return_type: &Option<String>,
        block: &Vec<Box<Statement>>,
    ) {
        self.initialize_block(params);
        self.translate_block(block);

        // TODO: Figure out return values from semantic analysis pass?
        // TODO: Or can I do this when evaluating return values?
        self.builder.ins().return_(&[]);
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
            Statement::Variable(variable_name, expression) => {
                let value = self.translate_expression(expression);
                let variable = Variable::new(self.variables.top().len());
                self.builder.declare_var(variable, types::F64); // TODO: Pass expression type from semantic type
                self.builder.def_var(variable, value[0]);
            }
            Statement::FunctionDeclaration(..) => {
                panic!("For right now, functions can only be declared at the top level.")
            }
            _ => {}
        }
    }

    pub fn translate_expression(
        &mut self,
        expression: &Expression,
    ) -> Vec<Value> {
        match expression {
            Expression::Boolean(bool) => {
                vec![self.builder.ins().bconst(types::B1, *bool)]
            }
            Expression::FunctionCall(function_name, args) => {
                // TODO: Determine function types in semantics pass
                // Functions need to know (1) name, (2) arg types, and (3) return type(s)
                let mut signature = self.module.make_signature();
                args.iter().for_each(|_| {
                    signature.params.push(AbiParam::new(types::F64));
                });
                // TODO: Support return types
                // signature.returns.push(AbiParam::new(types::F64));

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
                self.builder.ins().call(local_callee, &argument_values);
                Vec::new()
                // builder.inst_results(call)[0] // TODO: Support multiple returns
            }
            Expression::Number(num) => vec![self.builder.ins().f64const(*num)],
            Expression::Identifier(name) => {
                let variables = self.variables.top();
                let variable =
                    variables.get(name).expect("Variable not defined");
                vec![self.builder.use_var(*variable)]
            }
            Expression::Operation(left, operation, right) => {
                let lval = self.translate_expression(left)[0];
                let rval = self.translate_expression(right)[0];
                let result = match operation {
                    Operation::Add => self.builder.ins().fadd(lval, rval),
                    Operation::Subtract => self.builder.ins().fsub(lval, rval),
                    Operation::Multiply => self.builder.ins().fmul(lval, rval),
                    Operation::Divide => self.builder.ins().fdiv(lval, rval),
                    Operation::Equals => {
                        let bool =
                            self.builder.ins().fcmp(FloatCC::Equal, lval, rval);
                        self.builder.ins().bint(types::B1, bool)
                    }
                    Operation::NotEquals => {
                        let bool = self.builder.ins().fcmp(
                            FloatCC::NotEqual,
                            lval,
                            rval,
                        );
                        self.builder.ins().bint(types::B1, bool)
                    }
                    Operation::GreaterThan => {
                        let bool = self.builder.ins().fcmp(
                            FloatCC::GreaterThan,
                            lval,
                            rval,
                        );
                        self.builder.ins().bint(types::B1, bool)
                    }
                    Operation::LessThan => {
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

    fn initialize_block(&mut self, params: &[(String, String)]) {
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
        "int" => RoxType::Int,
        "float" => RoxType::Float,
        "str" => RoxType::String,
        _ => unimplemented!(),
    };
    get_codegen_type(&rox_type)
}

fn get_codegen_type(rox_type: &RoxType) -> types::Type {
    match rox_type {
        RoxType::Int => types::I64,
        RoxType::Float => types::F64,
        _ => unimplemented!(),
    }
}
