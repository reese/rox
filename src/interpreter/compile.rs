use std::str;

use super::chunk::{Byte, Chunk};
use super::op_code::OpCode;
use super::value::Value;
use crate::interpreter::{
    Block, Declaration, Expression, InterpretError, Operation, Push, RoxResult,
    Statement, Unary,
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
        source: &'compiler str,
    ) -> RoxResult<()> {
        match self.parse_source_code(source) {
            Err(errors) => {
                println!("{:?}", errors);
                InterpretError::compile_error()
            } // TODO: Properly convert errors
            Ok(declarations) => self.compile_declarations(&declarations),
        }
    }

    fn parse_source_code(
        &self,
        source: &'compiler str,
    ) -> Result<Vec<Box<Declaration>>, Vec<LalrpopParseError>> {
        let mut errors = Vec::new();
        let declarations = rox_parser::ProgramParser::new()
            .parse(&mut errors, source)
            .unwrap();
        match errors.clone() {
            empty_vec if empty_vec.is_empty() => Ok(declarations),
            error_vec => Err(error_vec),
        }
    }

    fn compile_declarations(
        &mut self,
        declarations: &[Box<Declaration>],
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
                Declaration::Record(..) => {
                    panic!("Sorry, I haven't implemented records yet.")
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
            Expression::And(left, right) => self.and_expression(left, right),
            Expression::Assignment(identifier, expression) => {
                self.assignment(identifier, expression)
            }
            Expression::Boolean(boolean) => self.boolean(boolean),
            Expression::Number(number) => self.number(number),
            Expression::Identifier(identifier) => {
                self.retrieve_variable_value(identifier)
            }
            Expression::String(string) => self.string(string),
            Expression::Operation(left, operation, right) => {
                self.execute_operation(left, operation, right)
            }
            Expression::Or(left, right) => self.or_expression(left, right),
            Expression::Unary(unary, expression) => {
                self.unary(unary, expression)
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
            Statement::Block(declarations) => {
                self.emit_byte(Byte::Op(OpCode::ScopeStart));
                self.compile_declarations(declarations).unwrap();
                self.emit_byte(Byte::Op(OpCode::ScopeEnd));
            }
            Statement::IfElse(dependent, if_block, else_block) => {
                self.if_statement(dependent, if_block, else_block);
            }
            Statement::While(expression, block) => {
                self.while_statement(expression, block);
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

    fn emit_jump(&mut self, op: OpCode) -> usize {
        self.emit_byte(Byte::Op(op));
        self.emit_byte(Byte::Op(OpCode::Placeholder));
        self.current_chunk().codes.len() - 1
    }

    /// # Control flow
    /// Control flow in the VM can be a little tough to wrap your head around.
    /// For some reference, it may be useful to read the
    /// [section](https://craftinginterpreters.com/jumping-back-and-forth.html#if-statements)
    /// in "Crafting Interpreters" on them. In short, what we do is emit a placeholder
    /// byte, parse the body of the block, and then once we finish the block, replace
    /// the placeholder with the length of the block's instructions.
    ///
    /// For example, if we start an if statement at instruction 5, instruction 6 will be a placeholder.
    /// Then, we will parse a block, say of length 7. We're now at instruction 13, so
    /// to get the offset we get the length of the block, which is the current instruction location
    /// minus the location of the placeholder (so here, `13 - 6`), and replace the placeholder
    /// with that value. We then do essentially the same thing if there's an `else` statement.
    fn if_statement(
        &mut self,
        dependent_expression: &Expression,
        if_block: &Block,
        optional_else_block: &Option<Block>,
    ) {
        self.expression(dependent_expression);
        let if_placeholder_index = self.emit_jump(OpCode::JumpIfFalse);
        self.emit_byte(Byte::Op(OpCode::Pop));
        self.compile_declarations(if_block).unwrap();

        let else_placeholder_index = self.emit_jump(OpCode::Jump);
        self.patch_jump(if_placeholder_index);

        if let Some(else_block) = optional_else_block {
            self.compile_declarations(else_block).unwrap()
        }
        self.patch_jump(else_placeholder_index);
        self.emit_byte(Byte::Op(OpCode::Pop));
    }

    fn and_expression(
        &mut self,
        left_side: &Expression,
        right_side: &Expression,
    ) {
        self.expression(left_side);
        let end_expression_index = self.emit_jump(OpCode::JumpIfFalse);
        self.emit_byte(Byte::Op(OpCode::Pop));
        self.expression(right_side);
        self.patch_jump(end_expression_index);
    }

    fn or_expression(
        &mut self,
        left_side: &Expression,
        right_side: &Expression,
    ) {
        self.expression(left_side);
        let else_jump_index = self.emit_jump(OpCode::JumpIfFalse);
        let end_jump = self.emit_jump(OpCode::Jump);

        self.patch_jump(else_jump_index);
        self.emit_byte(Byte::Op(OpCode::Pop));

        self.expression(right_side);

        self.patch_jump(end_jump);
    }

    fn while_statement(
        &mut self,
        expression: &Expression,
        block: &[Box<Declaration>],
    ) {
        let loop_start_index = self.current_chunk().codes.len();
        self.expression(expression);

        let end_jump_index = self.emit_jump(OpCode::JumpIfFalse);
        self.emit_byte(Byte::Op(OpCode::Pop));
        self.compile_declarations(block).unwrap();

        self.emit_loop(loop_start_index);
        self.patch_jump(end_jump_index);
        self.emit_byte(Byte::Op(OpCode::Pop));
    }

    fn emit_loop(&mut self, loop_start_index: usize) {
        self.emit_byte(Byte::Op(OpCode::Loop));
        let offset = self.current_chunk().codes.len() - loop_start_index + 1;

        self.emit_byte(Byte::Op(OpCode::OpLocation(offset)));
    }

    fn patch_jump(&mut self, offset: usize) {
        let current_location = self.current_chunk().codes.len();
        self.current_chunk().codes[offset] =
            Byte::Op(OpCode::OpLocation(current_location - offset));
    }

    fn number(&mut self, number: &f64) {
        let value = Value::Float(*number);
        self.emit_constant(value);
    }

    fn execute_operation(
        &mut self,
        left: &Expression,
        operation: &Operation,
        right: &Expression,
    ) {
        // The order of these is important so that they are popped off the stack in order
        self.expression(right);
        self.expression(left);

        match operation {
            Operation::Equals => self.emit_byte(Byte::Op(OpCode::Equal)),
            Operation::NotEquals => self.emit_byte(Byte::Op(OpCode::NotEquals)),
            Operation::Add => self.emit_byte(Byte::Op(OpCode::Add)),
            Operation::Subtract => self.emit_byte(Byte::Op(OpCode::Subtract)),
            Operation::Multiply => self.emit_byte(Byte::Op(OpCode::Multiply)),
            Operation::Divide => self.emit_byte(Byte::Op(OpCode::Divide)),
            Operation::Modulo => self.emit_byte(Byte::Op(OpCode::Modulo)),
            Operation::GreaterThan => {
                self.emit_byte(Byte::Op(OpCode::GreaterThan))
            }
            Operation::LessThan => self.emit_byte(Byte::Op(OpCode::LessThan)),
        }
    }

    fn assignment(&mut self, identifier: &str, expression: &Expression) {
        let identifier_constant = self.identifier_constant(identifier);
        self.expression(expression);
        self.emit_bytes(
            Byte::Constant(identifier_constant),
            Byte::Op(OpCode::SetVariable),
        )
    }

    fn string(&mut self, string: &str) {
        let val = Value::create_string(string.to_string());
        self.emit_constant(val)
    }

    fn print_statement(&mut self, expression: &Expression) {
        self.expression(expression);
        self.emit_byte(Byte::Op(OpCode::Print))
    }

    fn variable_declaration(
        &mut self,
        identifier: &str,
        expression: &Expression,
    ) {
        self.expression(expression);
        let variable_constant = self.identifier_constant(identifier);
        self.define_variable(variable_constant);
    }

    fn define_variable(&mut self, variable_constant: u8) {
        self.emit_bytes(
            Byte::Constant(variable_constant),
            Byte::Op(OpCode::DefineVariable),
        );
    }

    fn retrieve_variable_value(&mut self, identifier: &String) {
        let identifier_constant = self.identifier_constant(identifier);
        self.emit_bytes(
            Byte::Constant(identifier_constant),
            Byte::Op(OpCode::GetVariable),
        )
    }

    fn identifier_constant(&mut self, identifier_text: &str) -> u8 {
        self.make_constant(Value::create_string(String::from(identifier_text)))
    }

    fn unary(&mut self, unary: &Unary, expression: &Expression) {
        self.expression(expression);
        match unary {
            Unary::Not => self.emit_byte(Byte::Op(OpCode::Not)),
            Unary::Negate => self.emit_byte(Byte::Op(OpCode::Negate)),
        }
    }
}
