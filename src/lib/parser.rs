use super::token::{Token};

#[derive(Debug, Default)]
pub struct Parser<'a> {
  pub current: Token<'a>,
  pub previous: Token<'a>,
  pub hadError: bool,
  pub panic_mode: bool
}