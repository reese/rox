#![deny(missing_docs)]

//! # Rox
//! This module is the executable module for the Rox roxc.
extern crate rox;

use rox::build_file;
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
        Roxc::Build { file, output, .. } => build_file(file, output),
        _ => todo!("The JIT compiler for this hasn't been built yet. Instead, use the `build` command and run the executable."),
    };
}
