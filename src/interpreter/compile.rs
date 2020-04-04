use num_traits::FromPrimitive;
use std::str;

use super::chunk::{Byte, Chunk};
use super::common::OpCode;
use super::parse_rule::{ParseOp, ParseRule, RULES};
use super::parser::Parser;
use super::precedence::Precedence;
use super::scanner::Scanner;
use super::token::{Token, TokenType};
use super::traits::PushLine;
use super::value::Value;
use crate::interpreter::TokenType::TokenSemicolon;

#[derive(Debug)]
pub struct Compiler<'a> {
    scanner: Scanner<'a>,
    parser: Parser<'a>,
    compiling_chunk: &'a mut Chunk,
}

impl<'compiler> Compiler<'compiler> {
    pub fn new(
        source: &'compiler [u8],
        chunk: &'compiler mut Chunk,
    ) -> Compiler<'compiler> {
        Compiler {
            scanner: Scanner::new(source),
            parser: Parser::default(),
            compiling_chunk: chunk,
        }
    }

    pub fn compile(&'compiler mut self) -> bool {
        self.advance();
        while !self.match_token(TokenType::TokenEof) {
            self.declaration();
        }
        self.end_compile();

        !self.parser.had_error
    }

    pub fn advance(&mut self) {
        self.parser.previous = self.parser.current.clone();
        loop {
            self.parser.current = self.scanner.scan_token();
            if self.parser.current.token_type != TokenType::TokenError {
                break;
            }

            self.error_at_current("This is a terrible error message.");
        }
    }

    pub fn consume(&mut self, token_type: TokenType, message: &str) {
        let is_same_token = self.parser.current.token_type == token_type;
        if is_same_token {
            self.advance();
            return;
        }

        self.error_at_current(message);
    }

    fn binary(&mut self) {
        let operator_type = self.parser.previous.token_type;

        let rule = self.get_rule(operator_type);
        let precedence: Precedence =
            FromPrimitive::from_u8(rule.precedence as u8 + 1).unwrap();
        self.parse_precedence(precedence.clone());

        match operator_type {
            TokenType::TokenBangEqual => self
                .emit_bytes(Byte::Op(OpCode::OpEqual), Byte::Op(OpCode::OpNot)),
            TokenType::TokenEqualEqual => {
                self.emit_byte(Byte::Op(OpCode::OpEqual))
            }
            TokenType::TokenLess => {
                self.emit_byte(Byte::Op(OpCode::OpLessThan))
            }
            TokenType::TokenLessEqual => self.emit_bytes(
                Byte::Op(OpCode::OpGreaterThan),
                Byte::Op(OpCode::OpNot),
            ),
            TokenType::TokenGreater => {
                self.emit_byte(Byte::Op(OpCode::OpGreaterThan))
            }
            TokenType::TokenGreaterEqual => self.emit_bytes(
                Byte::Op(OpCode::OpLessThan),
                Byte::Op(OpCode::OpNot),
            ),
            TokenType::TokenPlus => self.emit_byte(Byte::Op(OpCode::OpAdd)),
            TokenType::TokenMinus => {
                self.emit_byte(Byte::Op(OpCode::OpSubtract))
            }
            TokenType::TokenStar => {
                self.emit_byte(Byte::Op(OpCode::OpMultiply))
            }
            TokenType::TokenSlash => self.emit_byte(Byte::Op(OpCode::OpDivide)),
            other => unreachable!(
                "Attempted to parse a binary operator, got {:?}",
                other
            ),
        }
    }

    fn current_chunk(&mut self) -> &mut Chunk {
        self.compiling_chunk
    }

    fn declaration(&mut self) {
        self.statement();
    }

    fn emit_byte(&mut self, byte: Byte) {
        let line = self.parser.previous.line;
        let chunk = self.current_chunk();
        chunk.push_line(byte, line);
    }

    fn emit_bytes(&mut self, byte_one: Byte, byte_two: Byte) {
        self.emit_byte(byte_one);
        self.emit_byte(byte_two);
    }

    fn emit_constant(&mut self, value: Value) {
        let constant_index = self.make_constant(value);
        self.emit_bytes(
            Byte::Op(OpCode::OpConstant),
            Byte::Constant(constant_index),
        );
    }

    fn emit_return(&mut self) {
        self.emit_byte(Byte::Op(OpCode::OpReturn))
    }

    fn end_compile(&mut self) {
        self.emit_return();
    }

    fn error_at_current(&mut self, message: &str) {
        self.error_at(self.parser.current.clone(), message);
    }

    fn error(&mut self, message: &str) {
        self.error_at(self.parser.previous.clone(), message);
    }

    fn error_at(&mut self, token: Token, message: &str) {
        // We currently don't have statement boundaries, so this will change later.
        // Otherwise, basically the whole thing will error out since we don't know
        // statements actually end.
        if self.parser.panic_mode {
            return;
        }
        self.parser.panic_mode = true;
        print!("{} Error", token.line);

        if token.token_type == TokenType::TokenEof {
            print!(" at end");
        } else if token.token_type == TokenType::TokenError {
            // pass
        } else {
            print!(
                " at {:?} on line {}",
                str::from_utf8(token.text).unwrap(),
                token.line
            );
        }

        println!("\n{}", message);
        self.parser.had_error = true;
    }

    fn expression(&mut self) {
        self.parse_precedence(Precedence::PrecedenceAssignment);
    }

    fn statement(&mut self) {
        match self.parser.current_token_type() {
            TokenType::TokenPrint => self.print_statement(),
            _ => self.expression(),
        }
    }

    fn get_rule(&self, token_type: TokenType) -> ParseRule {
        let index = token_type as usize;
        RULES[index]
    }

    fn grouping(&mut self) {
        self.expression();
        self.consume(
            TokenType::TokenRightParen,
            "Expected ')' after expression.",
        )
    }

    fn make_constant(&mut self, value: Value) -> u8 {
        let line = self.scanner.line;
        let chunk = self.current_chunk();
        chunk.push_line(value, line)
    }

    fn number(&mut self) {
        let string = str::from_utf8(self.parser.previous.text).unwrap();
        let value = Value::Float(string.parse().unwrap());
        self.emit_constant(value);
    }

    fn parse_precedence(&mut self, precedence: Precedence) {
        self.advance();
        let prefix_rule = self.get_rule(self.parser.previous.token_type);
        self.match_rule(prefix_rule.prefix);

        while precedence
            <= self.get_rule(self.parser.current_token_type()).precedence
        {
            self.advance();
            let infix_rule =
                self.get_rule(self.parser.previous.token_type).infix;
            self.match_rule(infix_rule);
        }
    }

    fn match_rule(&mut self, prefix_rule: ParseOp) {
        match prefix_rule {
            ParseOp::Binary => self.binary(),
            ParseOp::Grouping => self.grouping(),
            ParseOp::Literal => self.literal(),
            ParseOp::Number => self.number(),
            ParseOp::Noop => {}
            ParseOp::String => self.string(),
            ParseOp::Unary => self.unary(),
        }
    }

    fn match_token(&mut self, token: TokenType) -> bool {
        if !self.parser.check(token) {
            false
        } else {
            self.advance();
            true
        }
    }

    fn unary(&mut self) {
        let operator_type = self.parser.previous.token_type;

        self.parse_precedence(Precedence::PrecedenceUnary);

        match operator_type {
            TokenType::TokenMinus => self.emit_byte(Byte::Op(OpCode::OpNegate)),
            TokenType::TokenBang => self.emit_byte(Byte::Op(OpCode::OpNot)),
            _ => unreachable!(),
        }
    }

    fn literal(&mut self) {
        match self.parser.previous.token_type {
            TokenType::TokenFalse => self.emit_byte(Byte::Op(OpCode::OpFalse)),
            TokenType::TokenTrue => self.emit_byte(Byte::Op(OpCode::OpTrue)),
            _ => unreachable!(),
        }
    }

    fn string(&mut self) {
        let val = Value::create_string(&self.parser.previous.text[1..]);
        self.emit_constant(val)
    }

    fn print_statement(&mut self) {
        self.advance();
        self.expression();
        self.consume(TokenType::TokenSemicolon, "Expected ';' after value");
        self.emit_byte(Byte::Op(OpCode::OpPrint))
    }
}
