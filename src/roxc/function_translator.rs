use crate::roxc::local::Local;
use crate::roxc::parser;
use crate::roxc::vm::object::Object;
use crate::roxc::vm::{Chunk, OpCode, Value};
use crate::roxc::{TaggedExpression, TaggedStatement};
use std::rc::Rc;

pub(crate) struct FunctionTranslator<'c> {
    chunk: &'c mut Chunk,
    locals: Vec<Local>,
    scope_depth: u8,
}

impl<'c> FunctionTranslator<'c> {
    pub(crate) fn new(
        chunk: &'c mut Chunk,
        locals: Vec<Local>,
        scope_depth: u8,
    ) -> Self {
        FunctionTranslator {
            chunk,
            locals,
            scope_depth,
        }
    }

    pub(crate) fn translate_function(&mut self, block: &[TaggedStatement]) {
        block.iter().for_each(|statement| {
            self.translate_statement(statement);
        })
    }

    fn translate_statement(&mut self, statement: &TaggedStatement) {
        use TaggedStatement::*;
        match &statement {
            StructDeclaration => {}
            Block(statements) => {
                statements.iter().for_each(|s| self.translate_statement(s));
            }
            Variable(name, expression, _type) => {
                self.translate_expression(expression);
                self.chunk
                    .add_constant(Value::create_string(name.to_string()));
                self.chunk.write(OpCode::DeclareVariable);
            }
            Assignment(_, _, _) => todo!(),
            Expression(expression) => {
                self.translate_expression(expression);
                // Pop residual value off the stack
                self.chunk.write(OpCode::Pop);
            }
            FunctionDeclaration(..) => todo!(),
            // TODO: Do we need external functions like this
            // if it's in a VM? I think we can provide all of that
            // directly from Rust.
            //
            // The `extern` tag merely declares the function to the type checker
            // The linker will then try to dynamically link the function call
            // if one exists. For the most part, we use this as a way to use
            // `libc` functions, but this could potentially be used to link a
            // Rust runtime library, but that's still undetermined.
            ExternFunctionDeclaration(decl) => {
                todo!()
                // self.functions.insert(decl.name.clone(), decl.clone());
            }
            Return(maybe_expression) => todo!(),
            IfElse(conditional, if_statements, else_statements_maybe) => {
                todo!()
            }
        }
    }

    pub fn translate_expression(&mut self, expression: &TaggedExpression) {
        use TaggedExpression::*;
        match expression {
            Or(_, _) | And(_, _) | Access(_, _, _) => todo!(),
            Boolean(bool) => match bool {
                true => self.chunk.write(OpCode::True),
                false => self.chunk.write(OpCode::False),
            },
            FunctionCall(function_name, args, _rox_type) => todo!(),
            Array(tagged_expressions, type_) => todo!(),
            // TODO: escape characters, template strings
            String(string) => {
                self.chunk
                    .add_constant(Value::create_string(string.clone()));
            }
            Identifier(name, _rox_type) => {
                self.chunk
                    .add_constant(Value::create_string(name.to_string()));
                self.chunk.write(OpCode::ReadVariable)
            }
            StructInstantiation(_struct_type, _fields) => todo!(),
            Operation(left, operation, right) => {
                self.translate_expression(left);
                self.translate_expression(right);
                let op = match operation {
                    parser::Operation::Add => OpCode::Add,
                    parser::Operation::Multiply => OpCode::Multiply,
                    parser::Operation::Subtract => OpCode::Subtract,
                    parser::Operation::Divide => OpCode::Divide,
                    parser::Operation::Equals => OpCode::Equal,
                    parser::Operation::GreaterThan => OpCode::Greater,
                    parser::Operation::LessThan => OpCode::Less,
                    parser::Operation::NotEquals => {
                        self.chunk.write(OpCode::Not);
                        OpCode::Equal
                    }
                };
                self.chunk.write(op);
            }
            Number(num) => {
                let value = Value::Number(*num);
                self.chunk.add_constant(value);
            }
            Unary(unary, expr, _type) => {
                self.translate_expression(expr);
                match unary {
                    parser::Unary::Negate => self.chunk.write(OpCode::Negate),
                    parser::Unary::Not => self.chunk.write(OpCode::Not),
                }
            }
        }
    }
}
