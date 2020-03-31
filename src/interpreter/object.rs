#[derive(Debug)]
pub enum Object {
    String(String),
}

impl Object {
    pub fn has_equal_content(&self, other: &Object) -> bool {
        match (self, other) {
            (Object::String(first), Object::String(second)) => first.eq(second),
            _ => false,
        }
    }
}
