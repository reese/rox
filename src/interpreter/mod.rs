#![deny(missing_docs)]
pub use compile::*;
pub use function_translator::*;
pub use interpret_result::*;
pub use stack::*;
pub use syntax::*;
pub use traits::*;

mod compile;
mod function_translator;
mod interpret_result;
mod stack;
mod syntax;
mod traits;
