#![deny(missing_docs)]
#![allow(clippy::needless_doctest_main)]
#![allow(clippy::vec_box)]
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/reese/rox/master/assets/rox-logo-transparent-bg.png"
)]
#![doc(
    html_favicon_url = "https://raw.githubusercontent.com/reese/rox/master/assets/favicon.ico"
)]
//! # Rox

#[macro_use]
extern crate lalrpop_util;

mod roxc;

use crate::roxc::{get_builtin_types, parse_file, Compiler, Stack};
pub use crate::roxc::{Options, Result};
use inkwell::context::Context;
use inkwell::passes::PassManager;
use std::collections::HashMap;

/// `run_file` reads the contents of the given path
/// and runs them through the roxc.
///
/// # Errors
/// Currently, the Rox roxc exits with a
/// `1` error code if it fails to compile. In the future,
/// it would likely be a good idea to refactor these
/// to have POSIX compliant error codes, or at least
/// some consistent error code system.
pub fn build_file(options: Options) -> Result<()> {
    let context = Context::create();
    compile_file(options, &context)
}

fn compile_file(options: Options, context: &Context) -> Result<()> {
    let input_file = options.get_path();
    let module = context.create_module("rox");
    let declarations = parse_file(input_file)?;

    // TODO: Clean this shit up
    let mut environment_stack = Stack::new();
    environment_stack.push(HashMap::new());
    let (_, _, mut function_stack) = get_builtin_types();
    let function_pass_manager = PassManager::create(&module);

    let mut compiler = Compiler::new(
        &context,
        &module,
        &function_pass_manager,
        &mut environment_stack,
        &mut function_stack,
    );

    match options {
        Options::Run { .. } => unsafe {
            compiler.jit_compile()?;
        },
        Options::Build { output, .. } => {
            compiler.compile(declarations)?;
            compiler.finish(output);
        }
    }

    Ok(())
}
