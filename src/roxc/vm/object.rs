use crate::roxc::vm::function::Function;

#[derive(Debug, Clone, PartialOrd, PartialEq)]
pub(crate) enum Object {
    Function(Function),
    String(String),
}
