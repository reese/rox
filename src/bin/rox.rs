#![deny(missing_docs)]

//! # Rox
//! This module is the executable module for the Rox roxc.
extern crate rox;

use rox::{build_file, Options};
use structopt::StructOpt;

/// # Rox
/// This is the executable for running the Rox roxc.
fn main() -> std::io::Result<()> {
    let args = Options::from_args();
    let result = build_file(args);
    if let Err(error) = result {
        error.emit_error()
    } else {
        Ok(())
    }
}
