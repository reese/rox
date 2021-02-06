#[derive(Clone, Debug)]
pub(crate) struct Local {
    pub(crate) name: String,
    pub(crate) depth: i32,
}

impl Local {
    pub(crate) fn new(name: String, depth: i32) -> Self {
        Local { name, depth }
    }
}
