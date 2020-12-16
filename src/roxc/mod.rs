#![deny(missing_docs)]
pub(crate) use builtins::*;
pub use compile::*;
pub use errors::*;
pub(crate) use parser::*;
pub(crate) use semant::*;

mod builtins;
mod compile;
mod errors;
mod function_translator;
mod local;
mod parser;
mod semant;
mod stack;
mod vm;
