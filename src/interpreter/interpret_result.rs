pub type RoxResult<T> = std::result::Result<T, InterpretError>;

#[derive(Debug, PartialEq)]
pub enum InterpretErrorType {
    InterpretCompileError,
    InterpretRuntimeError,
}

#[derive(Debug, PartialEq)]
pub struct InterpretError {
    pub error_type: InterpretErrorType,
    pub message: Option<String>,
}

impl InterpretError {
    pub fn compile_error<T>() -> RoxResult<T> {
        Err(InterpretError {
            error_type: InterpretErrorType::InterpretCompileError,
            message: None,
        })
    }
}
