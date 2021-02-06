use crate::roxc::vm::function::Function;
use std::{
    borrow::Cow,
    ops::{Add, Div, Mul, Not, Sub},
};

use super::native_function::NativeFuncHolder;

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub(crate) enum Value {
    Unit,
    Bool(bool),
    Number(f64),
    String(Cow<'static, str>),
    Function(Function),
    NativeFunction(NativeFuncHolder),
}

impl Value {
    pub(crate) fn create_string(str: String) -> Self {
        Value::String(Cow::Owned(str))
    }

    pub(crate) fn read_bool(&self) -> bool {
        if let Value::Bool(bool) = self {
            *bool
        } else {
            unreachable!("Encountered unexpected value: {:?}", self);
        }
    }

    pub(crate) fn read_string(&self) -> Cow<'static, str> {
        if let Value::String(string) = self {
            string.clone()
        } else {
            unreachable!("Encountered unexpected value: {:?}", self);
        }
    }

    pub(crate) fn read_number(&self) -> f64 {
        if let Value::Number(number) = self {
            *number
        } else {
            unreachable!("Encountered unexpected value: {:?}", self);
        }
    }
}

impl Mul for Value {
    type Output = Value;

    fn mul(self, rhs: Self) -> Self::Output {
        if let Value::Number(left) = self {
            if let Value::Number(right) = rhs {
                Value::Number(left * right)
            } else {
                unreachable!("Right side of multiplication was not a number")
            }
        } else {
            unreachable!("Left side of multiplication was not a number.")
        }
    }
}

impl Add for Value {
    type Output = Value;

    fn add(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Value::Number(left), Value::Number(right)) => {
                Value::Number(left + right)
            }
            (Value::String(left), Value::String(right)) => {
                Value::String(left + right)
            }
            (left, right) => panic!(
                "Expected string or number values, got {:?} and {:?}",
                left, right
            ),
        }
    }
}

impl Div for Value {
    type Output = Value;

    fn div(self, rhs: Self) -> Self::Output {
        if let Value::Number(left) = self {
            if let Value::Number(right) = rhs {
                Value::Number(left / right)
            } else {
                unreachable!()
            }
        } else {
            unreachable!()
        }
    }
}

impl Sub for Value {
    type Output = Value;

    fn sub(self, rhs: Self) -> Self::Output {
        if let Value::Number(left) = self {
            if let Value::Number(right) = rhs {
                Value::Number(left - right)
            } else {
                unreachable!()
            }
        } else {
            unreachable!()
        }
    }
}

impl Not for Value {
    type Output = Value;

    fn not(self) -> Self::Output {
        if let Value::Bool(val) = self {
            Value::Bool(!val)
        } else {
            unreachable!()
        }
    }
}
