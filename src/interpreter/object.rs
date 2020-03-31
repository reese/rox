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
}
