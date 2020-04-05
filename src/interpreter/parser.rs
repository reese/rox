use super::token::Token;
use crate::interpreter::TokenType;

#[derive(Debug, Default)]
pub struct Parser<'a> {
    pub current: Token<'a>,
    pub previous: Token<'a>,
    pub had_error: bool,
    pub panic_mode: bool,
}

impl Parser<'_> {
    pub fn check(&self, token_type: TokenType) -> bool {
        self.current_token_type() == token_type
    }

    pub fn current_token_type(&self) -> TokenType {
        self.current.token_type
    }

    pub fn is_end_of_file(&self) -> bool {
        self.check(TokenType::TokenEof)
    }
}
