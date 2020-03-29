use super::traits::Push;
use crate::interpreter::RoxResult;
use std::ops::Neg;

#[derive(Clone, Debug)]
pub enum Value {
    Bool(bool),
    Float(f64),
    Nil,
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
