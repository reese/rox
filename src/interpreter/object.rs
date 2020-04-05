use std::fmt::Formatter;

#[derive(Debug)]
pub enum Object {
    String(String),
}

impl Object {
    pub fn has_equal_content(&self, other: &Object) -> bool {
        match (self, other) {
            (Object::String(first), Object::String(second)) => first.eq(second),
            // Eventually there will be more options here
        }
    }

    pub fn concatenate(&self, other: &Object) -> Object {
        match (self, other) {
            (Object::String(first), Object::String(second)) => {
                Object::String(format!("{}{}", first, second))
            } // TODO: This should probably be a catch all, since we won't be able to
              // concatenate other Objects (e.g. classes)
        }
    }

    pub fn get_string_value(&self) -> &String {
        match self {
            Object::String(val) => val,
        }
    }
}

impl std::fmt::Display for Object {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let display_string = match self {
            Object::String(string) => string,
        };
        write!(f, "{}", display_string)
    }
}
