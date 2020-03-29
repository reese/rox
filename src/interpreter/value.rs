use super::traits::Push;
use crate::interpreter::RoxResult;
use std::ops::Neg;

#[derive(Clone, Debug)]
pub enum Value {
    Bool(bool),
    Float(f64),
}

impl Value {
    pub fn add(self, other: Self) -> RoxResult<Value> {
        match (self, other) {
            (Value::Float(first), Value::Float(second)) => {
                Ok(Value::Float(first + second))
            }
            _ => panic!("Cannot add two non-float types."),
        }
    }

    pub fn subtract(self, other: Self) -> RoxResult<Value> {
        match (self, other) {
            (Value::Float(first), Value::Float(second)) => {
                Ok(Value::Float(first - second))
            }
            _ => panic!("Cannot subtract two non-float types."),
        }
    }

    pub fn divide(self, other: Self) -> RoxResult<Value> {
        match (self, other) {
            (Value::Float(first), Value::Float(second)) => {
                Ok(Value::Float(first / second))
            }
            _ => panic!("Cannot divide two non-float types."),
        }
    }

    pub fn multiply(self, other: Self) -> RoxResult<Value> {
        match (self, other) {
            (Value::Float(first), Value::Float(second)) => {
                Ok(Value::Float(first * second))
            }
            _ => panic!("Cannot multiply two non-float types."),
        }
    }

    pub fn equals(self, other: Self) -> RoxResult<Value> {
        match (self, other) {
            (Value::Float(first), Value::Float(second)) => {
                Ok(Value::Bool(first == second))
            }
            (Value::Bool(first), Value::Bool(second)) => {
                Ok(Value::Bool(first == second))
            }
            _ => panic!("Cannot compare equality of mismatched types"),
        }
    }

    pub fn less_than(self, other: Self) -> RoxResult<Value> {
        match (self, other) {
            (Value::Float(first), Value::Float(second)) => {
                Ok(Value::Bool(first < second))
            }
            _ => panic!("Cannot compare non-float types"),
        }
    }

    pub fn greater_than(self, other: Self) -> RoxResult<Value> {
        match (self, other) {
            (Value::Float(first), Value::Float(second)) => {
                Ok(Value::Bool(first > second))
            }
            _ => panic!("Cannot compare non-float types"),
        }
    }

    pub fn is_number(&self) -> bool {
        match self {
            Value::Float(_) => true,
            _ => false,
        }
    }

    pub fn is_bool(&self) -> bool {
        match self {
            Value::Bool(_) => true,
            _ => false,
        }
    }
}

impl Neg for Value {
    type Output = Self;
    fn neg(self) -> Self {
        match self {
            Value::Float(num) => Value::Float(-num),
            _ => panic!("Cannot negate non-numeric type."),
        }
    }
}

impl std::ops::Not for Value {
    type Output = Self;
    fn not(self) -> Self {
        match self {
            Value::Bool(val) => Value::Bool(!val),
            _ => panic!("Cannot apply unary operator `!` to non-bool Value."),
        }
    }
}

#[derive(Debug)]
pub struct ValueArray {
    pub values: Vec<Value>,
}

impl ValueArray {
    pub fn new() -> ValueArray {
        ValueArray { values: vec![] }
    }
}

impl Clone for ValueArray {
    fn clone(&self) -> ValueArray {
        ValueArray {
            values: self.values.clone(),
        }
    }
}

impl Push<Value> for ValueArray {
    fn push(&mut self, value: Value) {
        self.values.push(value)
    }
}
