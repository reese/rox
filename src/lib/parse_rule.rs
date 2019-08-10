use super::precedence::Precedence;

pub struct ParseRule<'a> {
  pub prefix: &'a str,
  pub infix: &'a str,
  pub precedence: Precedence
}
