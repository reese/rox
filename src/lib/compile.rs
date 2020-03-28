use num_traits::FromPrimitive;
use std::str;

use super::chunk::{Byte, Chunk};
use super::common::OpCode;
use super::parse_rule::{ParseOp, ParseRule};
use super::parser::Parser;
use super::precedence::Precedence;
use super::scanner::Scanner;
use super::token::{Token, TokenType};
use super::traits::PushLine;
use super::value::Value;

const U8_MAX: u8 = 255;
const RULES: [ParseRule; 42] = [
    ParseRule {
        prefix: ParseOp::Grouping,
        infix: ParseOp::Noop,
        precedence: Precedence::PrecedenceNone,
    }, // Left Paren
    ParseRule {
        prefix: ParseOp::Noop,
        infix: ParseOp::Noop,
        precedence: Precedence::PrecedenceNone,
    }, // Right Paren
    ParseRule {
        prefix: ParseOp::Noop,
        infix: ParseOp::Noop,
        precedence: Precedence::PrecedenceNone,
    }, // Left Brace
    ParseRule {
        prefix: ParseOp::Noop,
        infix: ParseOp::Noop,
        precedence: Precedence::PrecedenceNone,
    }, // Right Brace
    ParseRule {
        prefix: ParseOp::Noop,
        infix: ParseOp::Noop,
        precedence: Precedence::PrecedenceNone,
    }, // Comma
    ParseRule {
        prefix: ParseOp::Noop,
        infix: ParseOp::Noop,
        precedence: Precedence::PrecedenceNone,
    }, // Dot
    ParseRule {
        prefix: ParseOp::Unary,
        infix: ParseOp::Binary,
        precedence: Precedence::PrecedenceTerm,
    }, // Minus
    ParseRule {
        prefix: ParseOp::Noop,
        infix: ParseOp::Binary,
        precedence: Precedence::PrecedenceTerm,
    }, // Plus
    ParseRule {
        prefix: ParseOp::Noop,
        infix: ParseOp::Noop,
        precedence: Precedence::PrecedenceNone,
    }, // Semicolon
    ParseRule {
        prefix: ParseOp::Noop,
        infix: ParseOp::Binary,
        precedence: Precedence::PrecedenceFactor,
    }, // Slash
    ParseRule {
        prefix: ParseOp::Noop,
        infix: ParseOp::Binary,
        precedence: Precedence::PrecedenceFactor,
    }, // Star
    ParseRule {
        prefix: ParseOp::Noop,
        infix: ParseOp::Noop,
        precedence: Precedence::PrecedenceNone,
    }, // Bang
    ParseRule {
        prefix: ParseOp::Noop,
        infix: ParseOp::Noop,
        precedence: Precedence::PrecedenceNone,
    }, // BangEqual
    ParseRule {
        prefix: ParseOp::Noop,
        infix: ParseOp::Noop,
        precedence: Precedence::PrecedenceNone,
    }, // Equal
    ParseRule {
        prefix: ParseOp::Noop,
        infix: ParseOp::Noop,
        precedence: Precedence::PrecedenceNone,
    }, // Double Equal
    ParseRule {
        prefix: ParseOp::Noop,
        infix: ParseOp::Noop,
        precedence: Precedence::PrecedenceNone,
    }, // Greater
    ParseRule {
        prefix: ParseOp::Noop,
        infix: ParseOp::Noop,
        precedence: Precedence::PrecedenceNone,
    }, // Greater Equal
    ParseRule {
        prefix: ParseOp::Noop,
        infix: ParseOp::Noop,
        precedence: Precedence::PrecedenceNone,
    }, // Less
    ParseRule {
        prefix: ParseOp::Noop,
        infix: ParseOp::Noop,
        precedence: Precedence::PrecedenceNone,
    }, // Less Equal
    ParseRule {
        prefix: ParseOp::Noop,
        infix: ParseOp::Noop,
        precedence: Precedence::PrecedenceNone,
    }, // Identifier
    ParseRule {
        prefix: ParseOp::Noop,
        infix: ParseOp::Noop,
        precedence: Precedence::PrecedenceNone,
    }, // String
    ParseRule {
        prefix: ParseOp::Number,
        infix: ParseOp::Noop,
        precedence: Precedence::PrecedenceNone,
    }, // Number
    ParseRule {
        prefix: ParseOp::Noop,
        infix: ParseOp::Noop,
        precedence: Precedence::PrecedenceNone,
    }, // And
    ParseRule {
        prefix: ParseOp::Noop,
        infix: ParseOp::Noop,
        precedence: Precedence::PrecedenceNone,
    }, // Class
    ParseRule {
        prefix: ParseOp::Noop,
        infix: ParseOp::Noop,
        precedence: Precedence::PrecedenceNone,
    }, // Else
    ParseRule {
        prefix: ParseOp::Noop,
        infix: ParseOp::Noop,
        precedence: Precedence::PrecedenceNone,
    }, // Class
    ParseRule {
        prefix: ParseOp::Noop,
        infix: ParseOp::Noop,
        precedence: Precedence::PrecedenceNone,
    }, // Else
    ParseRule {
        prefix: ParseOp::Noop,
        infix: ParseOp::Noop,
        precedence: Precedence::PrecedenceNone,
    }, // False
    ParseRule {
        prefix: ParseOp::Noop,
        infix: ParseOp::Noop,
        precedence: Precedence::PrecedenceNone,
    }, // For
    ParseRule {
        prefix: ParseOp::Noop,
        infix: ParseOp::Noop,
        precedence: Precedence::PrecedenceNone,
    }, // Fn
    ParseRule {
        prefix: ParseOp::Noop,
        infix: ParseOp::Noop,
        precedence: Precedence::PrecedenceNone,
    }, // If
    ParseRule {
        prefix: ParseOp::Noop,
        infix: ParseOp::Noop,
        precedence: Precedence::PrecedenceNone,
    }, // Nil
    ParseRule {
        prefix: ParseOp::Noop,
        infix: ParseOp::Noop,
        precedence: Precedence::PrecedenceNone,
    }, // Or
    ParseRule {
        prefix: ParseOp::Noop,
        infix: ParseOp::Noop,
        precedence: Precedence::PrecedenceNone,
    }, // Print
    ParseRule {
        prefix: ParseOp::Noop,
        infix: ParseOp::Noop,
        precedence: Precedence::PrecedenceNone,
    }, // Return
    ParseRule {
        prefix: ParseOp::Noop,
        infix: ParseOp::Noop,
        precedence: Precedence::PrecedenceNone,
    }, // Super
    ParseRule {
        prefix: ParseOp::Noop,
        infix: ParseOp::Noop,
        precedence: Precedence::PrecedenceNone,
    }, // This
    ParseRule {
        prefix: ParseOp::Noop,
        infix: ParseOp::Noop,
        precedence: Precedence::PrecedenceNone,
    }, // True
    ParseRule {
        prefix: ParseOp::Noop,
        infix: ParseOp::Noop,
        precedence: Precedence::PrecedenceNone,
    }, // Var
    ParseRule {
        prefix: ParseOp::Noop,
        infix: ParseOp::Noop,
        precedence: Precedence::PrecedenceNone,
    }, // While
    ParseRule {
        prefix: ParseOp::Noop,
        infix: ParseOp::Noop,
        precedence: Precedence::PrecedenceNone,
    }, // Error
    ParseRule {
        prefix: ParseOp::Noop,
        infix: ParseOp::Noop,
        precedence: Precedence::PrecedenceNone,
    }, // EOF
];

#[derive(Debug)]
pub struct Compiler<'a> {
    scanner: Scanner<'a>,
    parser: Parser<'a>,
    compiling_chunk: &'a mut Chunk,
}

impl<'compiler> Compiler<'compiler> {
    pub fn new(
        source: &'compiler Vec<u8>,
        chunk: &'compiler mut Chunk,
    ) -> Compiler<'compiler> {
        return Compiler {
            scanner: Scanner::new(source),
            parser: Parser::default(),
            compiling_chunk: chunk,
        };
    }

    pub fn compile(&'compiler mut self) -> bool {
        self.advance();
        self.expression();
        self.consume(TokenType::TokenEof, "Expected end of expression");

        self.end_compile();

        return !self.parser.had_error;
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
        let operator_type = self.parser.previous.token_type.clone();

        let rule = self.get_rule(operator_type);
        let precedence: Precedence =
            FromPrimitive::from_u8(rule.precedence.clone() as u8 + 1).unwrap();
        self.parse_precedence(precedence.clone());

        match operator_type {
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
        return self.compiling_chunk;
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

    fn get_rule(&self, token_type: TokenType) -> &ParseRule {
        let index = token_type as usize;
        &RULES[index]
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
        let constant = chunk.push_line(value, line);
        if constant > U8_MAX {
            self.error("Too many constants in one chunk.");
            return 0;
        }

        return constant;
    }

    fn number(&mut self) {
        let string = str::from_utf8(self.parser.previous.text).unwrap();
        let value = Value {
            float: string.parse().unwrap(),
        };
        self.emit_constant(value);
    }

    fn parse_precedence(&mut self, precedence: Precedence) {
        self.advance();
        let token_type = self.parser.previous.token_type.clone();
        let prefix_rule = &self.get_rule(token_type).prefix;
        match prefix_rule {
            ParseOp::Grouping => self.grouping(),
            ParseOp::Binary => self.binary(),
            ParseOp::Unary => self.unary(),
            ParseOp::Number => self.number(),
            ParseOp::Noop => self.error("Expected expression."),
        }

        while precedence
            <= self.get_rule(self.parser.current.token_type).precedence
        {
            self.advance();
            let infix_rule =
                &self.get_rule(self.parser.previous.token_type).infix;
            match infix_rule {
                ParseOp::Grouping => self.grouping(),
                ParseOp::Binary => self.binary(),
                ParseOp::Unary => self.unary(),
                ParseOp::Number => self.number(),
                ParseOp::Noop => self.error("Expected expression."),
            }
        }
    }

    fn unary(&mut self) {
        let operator_type = self.parser.previous.token_type.clone();

        self.parse_precedence(Precedence::PrecedenceUnary);

        match operator_type {
            TokenType::TokenMinus => self.emit_byte(Byte::Op(OpCode::OpNegate)),
            _ => unreachable!(),
        }
    }
}
