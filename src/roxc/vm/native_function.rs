use super::Value;
use std::rc::Rc;
use std::{borrow::Cow, fmt};

pub(crate) trait NativeType: Sized {
    fn to_value(self) -> Value;
    fn from_value(v: Value) -> Option<Self>;
}

pub(crate) trait NativeFunction {
    fn call(&self, args: Vec<Value>) -> Option<Value>;
}

macro_rules! helper {
    ($x:ident $y:ident) => {
        $x
    };
}

macro_rules! gen_impls {
    ($($($T:ident),* -> $R:ident;)*) => {
        $(
            impl<$($T,)* $R> NativeFunction for fn($($T),*) -> $R
            where
                $($T: NativeType,)*
                $R: NativeType,
            {
                fn call(&self, args: Vec<Value>) -> Option<Value> {
                    let mut args = args.into_iter();
                    Some(NativeType::to_value(
                        self($(NativeType::from_value(helper!(args $T).next()?)?),*)
                    ))
                }
            }
        )*
    }
}

gen_impls! {
    A -> R;
    A, B -> R;
    A, B, C -> R;
    A, B, C, D -> R;
    A, B, C, D, E -> R;
    A, B, C, D, E, F -> R;
    A, B, C, D, E, F, G -> R;
    // etc.
}

impl NativeType for f64 {
    fn to_value(self) -> Value {
        Value::Number(self)
    }
    fn from_value(value: Value) -> Option<Self> {
        Some(value.read_number())
    }
}

impl NativeType for bool {
    fn to_value(self) -> Value {
        Value::Bool(self)
    }
    fn from_value(value: Value) -> Option<Self> {
        Some(value.read_bool())
    }
}

impl NativeType for Cow<'static, str> {
    fn to_value(self) -> Value {
        Value::String(self)
    }
    fn from_value(value: Value) -> Option<Self> {
        Some(value.read_string())
    }
}

impl NativeType for () {
    fn to_value(self) -> Value {
        Value::Unit
    }
    fn from_value(_value: Value) -> Option<Self> {
        Some(())
    }
}

#[derive(Clone)]
pub struct NativeFuncHolder {
    pub(crate) inner: Rc<dyn NativeFunction>,
}

impl NativeFuncHolder {
    pub(crate) fn call(&self, args: Vec<Value>) -> Option<Value> {
        self.inner.call(args)
    }
}

impl PartialEq for NativeFuncHolder {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.inner, &other.inner)
    }
}

impl fmt::Debug for NativeFuncHolder {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "NativeFuncHolder")
    }
}
impl fmt::Display for NativeFuncHolder {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "NativeFuncHolder")
    }
}

// TODO: make this not terrible
impl PartialOrd for NativeFuncHolder {
    fn partial_cmp(&self, _other: &Self) -> Option<std::cmp::Ordering> {
        unimplemented!("Cannot compare native functions")
    }
}
