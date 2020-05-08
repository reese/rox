use crate::roxc::{Declaration, Expression, Operation, Statement};
use cranelift::prelude::*;
use cranelift_module::Module;
use cranelift_object::ObjectBackend;
use im::HashMap;

pub struct FunctionTranslator<'func> {
    pub builder: FunctionBuilder<'func>,
    variables: &'func HashMap<String, Variable>,
    module: &'func mut Module<ObjectBackend>,
    index: &'func usize,
}

impl<'func> FunctionTranslator<'func> {
    pub fn new(
        builder: FunctionBuilder<'func>,
        variables: &'func mut HashMap<String, Variable>,
        module: &'func mut Module<ObjectBackend>,
        index: &'func mut usize,
    ) -> Self {
        FunctionTranslator {
            builder,
            variables,
            module,
            index,
        }
    }

    pub fn finalize(&mut self) {
        self.builder.finalize()
    }

    pub fn translate_expression(&mut self, expression: &Expression) -> Value {
        match expression {
            Expression::Boolean(bool) => {
                self.builder.ins().bconst(types::B1, *bool)
            }
            Expression::Number(num) => self.builder.ins().f64const(*num),
            Expression::Identifier(name) => {
                let variable =
                    self.variables.get(name).expect("Variable not defined");
                self.builder.use_var(*variable)
            }
            Expression::Operation(left, operation, right) => {
                let lval = self.translate_expression(left);
                let rval = self.translate_expression(right);
                match operation {
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
                }
            }
            _ => unimplemented!(),
        }
    }

    pub fn translate_statement(&mut self, statement: &Statement) -> Value {
        match statement {
            Statement::Expression(expression) => {
                self.translate_expression(expression)
            }
            other => panic!(format!("Expected Expression, got {:?}", other)),
        }
    }
}
