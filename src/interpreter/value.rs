use std::ops::{Add, Div, Mul, Neg, Sub};

use super::traits::Push;

#[derive(Clone, Debug)]
pub struct Value {
    pub float: f64,
}

impl Add for Value {
    type Output = Self;
    fn add(self, other: Self) -> Value {
        Value {
            float: self.float + other.float,
        }
    }
}

impl Div for Value {
    type Output = Self;
    fn div(self, other: Self) -> Value {
        Value {
            float: self.float / other.float,
        }
    }
}

impl Mul for Value {
    type Output = Self;
    fn mul(self, other: Self) -> Value {
        Value {
            float: self.float * other.float,
        }
    }
}

impl Neg for Value {
    type Output = Value;
    fn neg(self) -> Value {
        Value { float: -self.float }
    }
}

impl Sub for Value {
    type Output = Self;
    fn sub(self, other: Self) -> Value {
        Value {
            float: self.float - other.float,
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
