use super::traits::Push;
use crate::interpreter::{Object, RoxResult};
use std::fmt::Formatter;
use std::ops::Neg;
use std::rc::Rc;

/// # Values
/// The `Value` enum represents the core value types in Rox.
/// With the exception of numbers (which are all floats in Rox) and booleans,
/// all other values in Rox are heap-allocated `Object`s. All `Object`s are
/// reference-counted and thus are garbage collected when all references go out of scope.
#[derive(Clone, Debug, PartialOrd, PartialEq)]
pub enum Value {
    /// `Bool` represents standard boolean values (`true` and `false`).
    Bool(bool),
    /// All numbers in Rox are stored as Rust's `f64` type. This is likely to change
    /// in the future.
    Float(f64),
    /// All values that are not `Bool` or `Float` values are reference-counted `Object`s that
    /// are allocated on the heap. This includes strings, records, functions, etc.
    Object(Rc<Object>),
}

impl Value {
    /// Creates a `Value::String` from a Rust `String` value
    pub fn create_string(string: String) -> Self {
        let object = Rc::new(Object::String(string));
        Value::Object(object)
    }

    /// Gets the Rust `String` value from the object.
    /// This panics if the `Value` is not a `Object::String` variant.
    pub fn get_string_value(&self) -> &String {
        match self {
            Value::Object(obj) => obj.get_string_value(),
            _ => panic!(
                "Attempted to retrieve string value of non-string Value type."
            ),
        }
    }

    /// Checks if Value::Bool is `true`. This panics if the value is
    /// not a `Bool` variant.
    pub fn is_true(&self) -> &bool {
        match self {
            Value::Bool(val) => val,
            _ => {
                panic!("Attempted to retrieve boolean value from non-bool type")
            }
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

impl std::ops::Add for Value {
    type Output = Self;
    fn add(self, other: Self) -> Value {
        match (self, other) {
            (Value::Float(first), Value::Float(second)) => {
                Value::Float(first + second)
            }
            (Value::Object(first), Value::Object(second)) => {
                Value::Object(Rc::from(first.concatenate(&second)))
            }
            _ => panic!("Cannot add two non-float types."),
        }
    }
}

impl std::ops::Div for Value {
    type Output = Self;
    fn div(self, other: Self) -> Self {
        match (self, other) {
            (Value::Float(first), Value::Float(second)) => {
                Value::Float(first / second)
            }
            _ => panic!("Cannot divide two non-float types."),
        }
    }
}

impl std::ops::Mul for Value {
    type Output = Self;
    fn mul(self, other: Self) -> Self {
        match (self, other) {
            (Value::Float(first), Value::Float(second)) => {
                Value::Float(first * second)
            }
            _ => panic!("Cannot multiply two non-float types."),
        }
    }
}

impl std::ops::Sub for Value {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        match (self, other) {
            (Value::Float(first), Value::Float(second)) => {
                Value::Float(first - second)
            }
            _ => panic!("Cannot subtract two non-float types."),
        }
    }
}

impl std::ops::Rem for Value {
    type Output = Self;

    fn rem(self, other: Self) -> Self {
        match (self, other) {
            (Value::Float(first), Value::Float(second)) => {
                Value::Float(first % second)
            }
            _ => panic!("Cannot compare non-float types"),
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

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Object(object) => object.as_ref().fmt(f),
            Value::Float(num) => write!(f, "{}", num),
            Value::Bool(val) => write!(f, "{}", val),
        }
    }
}

// TODO: Can we remove this?
#[derive(Debug, PartialOrd, PartialEq)]
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
    fn push(&mut self, value: Value) -> u8 {
        self.values.push(value);
        (self.values.len() - 1) as u8
    }
}
