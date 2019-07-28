use std::ops::{Add, Div, Mul, Neg, Sub};

use super::traits::Push;

#[derive(Clone, Debug)]
pub struct Value {
  pub f: f64
}

impl Add for Value {
  type Output = Self;
  fn add(self, other: Self) -> Value {
    return Value { f: self.f + other.f }
  }
}

impl Div for Value {
  type Output = Self;
  fn div(self, other: Self) -> Value {
    return Value { f: self.f / other.f }
  }
}

impl Mul for Value {
  type Output = Self;
  fn mul(self, other: Self) -> Value {
    return Value { f: self.f * other.f }
  }
}

impl Neg for Value {
  type Output = Value;
  fn neg(self) -> Value {
    return Value { f: -self.f }
  }
}

impl Sub for Value {
  type Output = Self;
  fn sub(self, other: Self) -> Value {
    return Value { f: self.f - other.f }
  }
}


#[derive(Debug)]
pub struct ValueArray {
  pub values: Vec<Value>
}

impl ValueArray {
  pub fn new() -> ValueArray {
    return ValueArray {
      values: vec![]
    }
  }
}

impl Clone for ValueArray{
  fn clone(&self) -> ValueArray {
    return ValueArray {
      values: self.values.clone()
    }
  }
}

impl Push<Value> for ValueArray {
  fn push(&mut self, value: Value) {
    self.values.push(value)
  }
}
