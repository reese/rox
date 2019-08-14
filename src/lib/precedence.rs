use num_derive::FromPrimitive;

#[derive(Clone, Debug, FromPrimitive, PartialEq, PartialOrd)]
pub enum Precedence {
  PrecedenceNone,
  PrecedenceAssignment,
  PrecedenceOr,
  PrecedenceAnd,
  PrecedenceEquality,
  PrecedenceComparison,
  PrecedenceTerm,
  PrecedenceFactor,
  PrecedenceUnary,
  PrecedenceCall,
  PrecedencePrimary,
}
