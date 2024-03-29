#![deny(missing_docs)]

//! # Rox
//! This module is the executable module for the Rox roxc.
extern crate rox;

use rox::{build_file, run_file};
use std::{path::PathBuf, process::exit};
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
    let exit_status = match args {
        Roxc::Build { file, output, .. } => build_file(file, output),
        Roxc::Run { file } => run_file(file),
    };
    println!("rox: {}", exit_status);
    exit(exit_status.code().unwrap_or(0));
}
