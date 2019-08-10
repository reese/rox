#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TokenType {
  TokenLeftParen,
  TokenRightParen,
  TokenLeftBrace,
  TokenRightBrace,
  TokenComma,
  TokenDot,
  TokenMinus,
  TokenPlus,
  TokenSemicolon,
  TokenSlash,
  TokenStar,

  TokenBang,
  TokenBangEqual,
  TokenEqual,
  TokenEqualEqual,
  TokenGreater,
  TokenGreaterEqual,
  TokenLess,
  TokenLessEqual,

  TokenIdentifier,
  TokenString,
  TokenNumber,

  TokenAnd,
  TokenClass,
  TokenElse,
  TokenFalse,
  TokenFor,
  TokenFn,
  TokenIf,
  TokenNil,
  TokenOr,
  TokenPrint,
  TokenReturn,
  TokenSuper,
  TokenThis,
  TokenTrue,
  TokenVar,
  TokenWhile,

  TokenError,
  TokenEof,
  TokenDefault,
}

impl Default for TokenType {
  fn default() -> TokenType { TokenType::TokenDefault }
}

#[derive(Clone, Debug, Default)]
pub struct Token<'scanner> {
  pub token_type: TokenType,
  pub text: &'scanner [u8],
  pub line: i32,
}
