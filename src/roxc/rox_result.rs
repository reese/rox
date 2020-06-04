use std::error::Error;
use std::fmt::{Display, Formatter};
use std::{fmt, io};

pub type RoxResult<T> = std::result::Result<T, RoxError>;

#[derive(Clone, Debug, PartialEq)]
pub enum RoxErrorType {
    CompileError,
    TypeInferenceError,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RoxError {
    pub error_type: RoxErrorType,
    pub message: Option<String>,
}

impl RoxError {
    pub fn compile_error<T>() -> RoxResult<T> {
        Err(RoxError {
            error_type: RoxErrorType::CompileError,
            message: None,
        })
    }
}

impl Display for RoxError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}: {:#?}", self.error_type, self.message)
    }
}

impl Error for RoxError {}
