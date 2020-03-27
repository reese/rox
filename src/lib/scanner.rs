use super::token::{Token, TokenType};

#[derive(Debug)]
pub struct Scanner<'b> {
  pub source: &'b Vec<u8>,
  start: usize,
  pub current: usize,
  pub line: i32,
}

impl<'scanner> Scanner<'scanner> {
  pub fn new(source: &Vec<u8>) -> Scanner {
    return Scanner {
      source: source,
      start: 0,
      current: 0,
      line: 1,
    };
  }

  pub fn advance(&mut self) {
    self.current += 1;
  }

  fn is_at_end(&self) -> bool {
    return self.peek() == String::from("\0").as_bytes()[0];
  }

  pub fn scan_token(&mut self) -> Token<'scanner> {
    self.skip_whitespace();

    self.start = self.current;

    if self.is_at_end() {
      let token = self.make_token(TokenType::TokenEof);
      return token;
    }

    let character = self.peek();
    if Scanner::is_alpha(character) {
      return self.identifier();
    } else if Scanner::is_digit(character) {
      return self.number();
    }

    let token = match character {
      // b'' is a byte literal representation of the char
      b'(' => self.make_token(TokenType::TokenLeftParen),
      b')' => self.make_token(TokenType::TokenRightParen),
      b'{' => self.make_token(TokenType::TokenLeftBrace),
      b'}' => self.make_token(TokenType::TokenRightBrace),
      b';' => self.make_token(TokenType::TokenSemicolon),
      b',' => self.make_token(TokenType::TokenComma),
      b'.' => self.make_token(TokenType::TokenDot),
      b'+' => self.make_token(TokenType::TokenPlus),
      b'-' => self.make_token(TokenType::TokenMinus),
      b'/' => self.make_token(TokenType::TokenSlash),
      b'*' => self.make_token(TokenType::TokenStar),
      b'!' => {
        if self.match_char(b'=') {
          return self.make_token(TokenType::TokenBangEqual);
        } else {
          return self.make_token(TokenType::TokenBang);
        }
      }
      b'=' => {
        if self.match_char(b'=') {
          return self.make_token(TokenType::TokenEqualEqual);
        } else {
          return self.make_token(TokenType::TokenEqual);
        }
      }
      b'>' => {
        if self.match_char(b'=') {
          return self.make_token(TokenType::TokenGreaterEqual);
        } else {
          return self.make_token(TokenType::TokenGreater);
        }
      }
      b'<' => {
        if self.match_char(b'=') {
          return self.make_token(TokenType::TokenLessEqual);
        } else {
          return self.make_token(TokenType::TokenLess);
        }
      }
      _ => self.error_token(),
    };

    self.advance();
    return token;
  }

  fn check_keyword(
    &self,
    start: usize,
    length: usize,
    rest: &str,
    token_type: TokenType,
  ) -> TokenType {
    if (self.current - self.start == start + length)
      && (&self.source[(self.current + start)..(length + 1)] == rest.as_bytes())
    {
      return token_type;
    }

    return TokenType::TokenIdentifier;
  }

  fn identifier(&mut self) -> Token<'scanner> {
    while Scanner::is_alpha(self.peek()) || Scanner::is_digit(self.peek()) {
      self.advance()
    }

    return self.make_token(self.identifier_type());
  }

  fn number(&mut self) -> Token<'scanner> {
    while Scanner::is_digit(self.peek()) {
      self.advance();
    }

    if self.peek() == b'.' && Scanner::is_digit(self.peek_next()) {
      self.advance();

      while Scanner::is_digit(self.peek()) {
        self.advance();
      }
    }

    return self.make_token(TokenType::TokenNumber);
  }

  fn identifier_type(&self) -> TokenType {
    match self.source[self.start] {
      b'a' => return self.check_keyword(1, 2, "nd", TokenType::TokenAnd),
      b'c' => return self.check_keyword(1, 4, "lass", TokenType::TokenClass),
      b'e' => return self.check_keyword(1, 3, "lse", TokenType::TokenElse),
      b'i' => return self.check_keyword(1, 1, "f", TokenType::TokenIf),
      b'n' => return self.check_keyword(1, 2, "il", TokenType::TokenNil),
      b'o' => return self.check_keyword(1, 1, "r", TokenType::TokenOr),
      b'p' => return self.check_keyword(1, 4, "rint", TokenType::TokenPrint),
      b'r' => return self.check_keyword(1, 5, "eturn", TokenType::TokenReturn),
      b's' => return self.check_keyword(1, 4, "uper", TokenType::TokenSuper),
      b'v' => return self.check_keyword(1, 2, "ar", TokenType::TokenVar),
      b'w' => return self.check_keyword(1, 4, "hile", TokenType::TokenWhile),
      b'f' => {
        if (self.current - self.start) > 1 {
          match self.source[self.current + 1] {
            b'a' => return self.check_keyword(2, 3, "lse", TokenType::TokenFalse),
            b'o' => return self.check_keyword(2, 1, "r", TokenType::TokenFalse),
            b'n' => return self.check_keyword(2, 0, "n", TokenType::TokenFalse),
            _ => {}
          }
        }
      }
      b't' => {
        if (self.current - self.start) > 1 {
          match self.source[self.current + 1] {
            b'h' => return self.check_keyword(2, 2, "is", TokenType::TokenThis),
            b'r' => return self.check_keyword(2, 12, "ue", TokenType::TokenTrue),
            _ => {}
          }
        }
      }
      _ => {}
    }

    return TokenType::TokenIdentifier;
  }

  fn make_token(&self, token_type: TokenType) -> Token<'scanner> {
    let token = Token {
      token_type: token_type,
      text: &self.source[self.start..self.current],
      line: self.line,
    };

    return token;
  }

  fn match_char(&mut self, expected: u8) -> bool {
    if self.is_at_end() {
      return false;
    }
    if self.peek() != expected {
      return false;
    }
    self.advance();
    return true;
  }

  fn error_token(&self) -> Token<'scanner> {
    return Token {
      token_type: TokenType::TokenError,
      text: &self.source[self.start..self.current],
      line: self.line,
    };
  }

  fn skip_whitespace(&mut self) {
    loop {
      let current_char = self.peek();
      match current_char {
        b' ' | b'\r' | b'\t' => self.advance(),
        b'\n' => {
          self.line += 1;
          self.advance();
        }
        b'/' => {
          if self.peek_next() == b'/' {
            while self.peek() != b'\n' && !self.is_at_end() {
              self.advance();
            }
          } else {
            return;
          }
        }
        _ => return,
      }
    }
  }

  fn peek(&self) -> u8 {
    // This is a check for the repl, since there's no EOF
    // on the command line
    if self.current == self.source.len() {
      return b'\0';
    }
    return self.source[self.current];
  }

  fn peek_next(&self) -> u8 {
    if self.is_at_end() {
      return b'\0';
    } else {
      return self.source[self.current + 1];
    }
  }

  fn is_alpha(character: u8) -> bool {
    return (character >= b'a' && character <= b'z')
      || (character >= b'A' && character <= b'Z')
      || character == b'_';
  }

  fn is_digit(character: u8) -> bool {
    return character >= b'0' && character <= b'9';
  }
}