use crate::roxc::vm::object::Object;
use std::ops::{Add, Div, Mul, Not, Sub};
use std::rc::Rc;

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub(crate) enum Value {
    Bool(bool),
    Number(f64),
    Obj(Rc<Object>),
}

impl Value {
    pub(crate) fn create_string(str: String) -> Self {
        Value::Obj(Rc::new(Object::String(str)))
    }
    /// Read an index constant. Used
    /// for offsets in jumps and indices
    pub(crate) fn read_number(&self) -> usize {
        if let Value::Number(num) = self {
            *num as usize
        } else {
            unreachable!("Encountered unexpected value: {:?}", self);
        }
    }

    pub(crate) fn read_bool(&self) -> bool {
        if let Value::Bool(bool) = self {
            *bool
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
                unreachable!()
            }
        } else {
            unreachable!()
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
            (Value::Obj(left), Value::Obj(right)) => {
                match (left.as_ref(), right.as_ref()) {
                    (Object::String(l), Object::String(r)) => Value::Obj(
                        Rc::new(Object::String(l.clone() + r.as_str())),
                    ),
                    _ => unreachable!(),
                }
            }
            _ => unreachable!(),
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
