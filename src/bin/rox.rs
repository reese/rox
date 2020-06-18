#![deny(missing_docs)]

//! # Rox
//! This module is the executable module for the Rox roxc.
extern crate rox;

use rox::{build_file, run_file};
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(about = "The command line interface to the Rox compiler.")]
enum Roxc {
    /// Compiles and links the program
    Build {
        /// The file to compile
        #[structopt(parse(from_os_str))]
        file: PathBuf,
        /// The name of the output executable
        #[structopt(short, long)]
        output: PathBuf,
        /// Option to not link the native object file
        #[structopt(short, long)]
        no_link: bool,
    },
    /// Executes the program with Rox's JIT compiler
    Run {
        /// The file to run
        #[structopt(parse(from_os_str))]
        file: PathBuf,
    },
}

/// # Rox
/// This is the executable for running the Rox roxc.
fn main() {
    let args = Roxc::from_args();
    match args {
        Roxc::Build {
            file,
            output,
            no_link,
        } => build_file(file, output, no_link).unwrap(),
        Roxc::Run { file } => run_file(file).unwrap(),
    }
}
