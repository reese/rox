use crate::interpreter::function::Function;
use std::fmt::Formatter;

#[derive(Debug, PartialOrd, PartialEq)]
pub enum Object {
    Function(Function),
    String(String),
}

impl Object {
    pub fn concatenate(&self, other: &Object) -> Object {
        match (self, other) {
            (Object::String(first), Object::String(second)) => {
                Object::String(format!("{}{}", first, second))
            }
            _ => panic!("Cannot concatenate non-string objects"),
        }
    }

    pub fn get_string_value(&self) -> &String {
        match self {
            Object::String(val) => val,
            _ => panic!("Object is not a string"),
        }
    }
}

impl std::fmt::Display for Object {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let display_string = match self {
            Object::String(string) => string,
            Object::Function(func) => "<func>", // TODO: Make this something more useful
        };
        write!(f, "{}", display_string)
    }
}
