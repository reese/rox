// //! # Semantic Analysis
// //! The `semant` module holds all of the semantic analysis
// //! that happens as part of the compilation process.
// //! The most important part of this analysis is
// //! type inference.
// //!
// //! ## Type System
// //!
// //! ### Motivations
// //!
// //! Rox is explicitly typed with type inference for variables.
// //! The reason for choosing explicit typing over total implicit typing (i.e. using type
// //! inference for functions and types) is that while implicit typing does reduce
// //! a lot of the visual noise, there are some instances that are inherently not translatable
// //! to from an explicit system to an implicit system, such as defining polymorphic
// //! functions that take arguments which are themselves polymorphic (such as a function
// //! that takes two polymorphic functions as arguments).
// //!
// //! The more important reason for using explicit types is readability.
// //! Function declarations are often the first thing read by a user, and while good argument
// //! names can be helpful, _good types are even better_. Having type information in the
// //! header of a function or in the field declarations of a type are, more often than not,
// //! far more helpful for readability than the names alone.
// //!
// //! ### Trade-offs
// //!
// //! As with everything, making `Rox` explicitly typed has its trade-offs.
// //! Of course, while explicit types provide more information up-front to the
// //! reader, they can also potentially act as clutter. They also pose more
// //! work to the programmer when refactoring and changing types.

mod tagged_syntax;
mod type_checker;
mod types;

pub(crate) use tagged_syntax::*;
pub(crate) use type_checker::*;
pub(crate) use types::*;
