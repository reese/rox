use std::str;

use super::chunk::{Byte, Chunk};
use super::op_code::OpCode;
use super::value::Value;
use crate::interpreter::{
    Declaration, Expression, InterpretError, Operation, Push, RoxResult,
    Statement,
};
use lalrpop_util::lexer::Token;
use lalrpop_util::ErrorRecovery;

lalrpop_mod!(pub rox_parser);

type LalrpopParseError<'input> =
    ErrorRecovery<usize, Token<'input>, &'static str>;

#[derive(Debug)]
pub struct Compiler<'a> {
    compiling_chunk: &'a mut Chunk,
}

impl<'compiler> Compiler<'compiler> {
    pub fn new(chunk: &'compiler mut Chunk) -> Compiler<'compiler> {
        Compiler {
            compiling_chunk: chunk,
        }
    }

    pub fn compile(
        &'compiler mut self,
        source: &'compiler String,
    ) -> RoxResult<()> {
        match self.parse_source_code(source) {
            Err(errors) => {
                println!("{:?}", errors);
                InterpretError::compile_error()
            } // TODO: Properly convert errors
            Ok(declarations) => self.compile_declarations(declarations),
        }
    }

    fn parse_source_code(
        &self,
        source: &'compiler String,
    ) -> Result<Vec<Box<Declaration>>, Vec<LalrpopParseError>> {
        let mut errors = Vec::new();
        let declarations: Vec<Box<Declaration>> =
            rox_parser::ProgramParser::new()
                .parse(&mut errors, source)
                .unwrap();
        match errors.clone() {
            empty_vec if empty_vec.is_empty() => Ok(declarations),
            error_vec => Err(error_vec),
        }
    }

    fn compile_declarations(
        &mut self,
        declarations: Vec<Box<Declaration>>,
    ) -> RoxResult<()> {
        declarations.iter().for_each(|declaration| {
            match declaration.as_ref() {
                Declaration::Statement(statement) => self.statement(&statement),
                Declaration::Variable(identifier, expression) => {
                    self.variable_declaration(identifier, expression)
                }
                Declaration::Function(..) => {
                    panic!("Sorry, I haven't implemented functions yet.")
                }
                Declaration::Class(..) => {
                    panic!("Sorry, I haven't implemented classes yet.")
                }
            }
        });
        Ok(())
    }

    fn current_chunk(&mut self) -> &mut Chunk {
        self.compiling_chunk
    }

    fn emit_byte(&mut self, byte: Byte) {
        let chunk = self.current_chunk();
        chunk.push(byte);
    }

    fn emit_bytes(&mut self, byte_one: Byte, byte_two: Byte) {
        self.emit_byte(byte_one);
        self.emit_byte(byte_two);
    }

    fn emit_constant(&mut self, value: Value) {
        let constant_index = self.make_constant(value);
        self.emit_bytes(
            Byte::Op(OpCode::Constant),
            Byte::Constant(constant_index),
        );
    }

    fn expression(&mut self, expression: &Expression) {
        match expression {
            Expression::Boolean(boolean) => self.boolean(boolean),
            Expression::Number(number) => self.number(number),
            Expression::Identifier(identifier) => {
                self.retrieve_variable_value(identifier)
            }
            Expression::String(string) => self.string(string),
            Expression::Operation(left, operation, right) => {
                self.execute_operation(left, operation, right)
            }
            Expression::ParseError => {
                panic!("Somehow the parse errors got through to execution.")
            }
        }
    }

    fn statement(&mut self, statement: &Statement) {
        match statement {
            Statement::Expression(expression) => self.expression(expression),
            Statement::Print(expression) => self.print_statement(expression),
            Statement::Return(maybe_expression) => {
                match maybe_expression {
                    None => {}
                    Some(expression) => {
                        self.expression(expression);
                    }
                }
                self.emit_byte(Byte::Op(OpCode::Return))
            }
            Statement::While(..)
            | Statement::For
            | Statement::If
            | Statement::Block(..) => {
                panic!("This statement type has not yet been implemented")
            }
        }
    }

    fn make_constant(&mut self, value: Value) -> u8 {
        let chunk = self.current_chunk();
        chunk.push(value)
    }

    fn boolean(&mut self, val: &bool) {
        self.emit_constant(Value::Bool(*val))
    }

    fn number(&mut self, number: &f64) {
        let value = Value::Float(*number);
        self.emit_constant(value);
    }

    fn execute_operation(
        &mut self,
        left: &Box<Expression>,
        operation: &Operation,
        right: &Box<Expression>,
    ) {
        // The order of these is important so that they are popped off the stack in order
        self.expression(right);
        self.expression(left);
        match operation {
            Operation::Add => self.emit_byte(Byte::Op(OpCode::Add)),
            Operation::Subtract => self.emit_byte(Byte::Op(OpCode::Subtract)),
            Operation::Multiply => self.emit_byte(Byte::Op(OpCode::Multiply)),
            Operation::Divide => self.emit_byte(Byte::Op(OpCode::Divide)),
            Operation::Modulo => self.emit_byte(Byte::Op(OpCode::Modulo)),
        }
    }

    fn string(&mut self, string: &String) {
        let val = Value::create_string(string.clone());
        self.emit_constant(val)
    }

    fn print_statement(&mut self, expression: &Expression) {
        self.expression(expression);
        self.emit_byte(Byte::Op(OpCode::Print))
    }

    fn variable_declaration(
        &mut self,
        identifier: &String,
        expression: &Box<Expression>,
    ) {
        self.expression(expression);
        let variable_constant = self.identifier_constant(identifier);
        self.define_variable(variable_constant);
    }

    fn define_variable(&mut self, variable_constant: u8) {
        self.emit_bytes(
            Byte::Constant(variable_constant),
            Byte::Op(OpCode::DefineGlobal),
        );
    }

    fn retrieve_variable_value(&mut self, identifier: &String) {
        let identifier_constant = self.identifier_constant(identifier);
        self.emit_bytes(
            Byte::Constant(identifier_constant),
            Byte::Op(OpCode::GetGlobal),
        )
    }

    fn identifier_constant(&mut self, identifier_text: &String) -> u8 {
        self.make_constant(Value::create_string(identifier_text.clone()))
    }
}
