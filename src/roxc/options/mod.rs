#![allow(missing_docs)]
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(about = "The command line interface to the Rox compiler.")]
pub enum Options {
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

impl Options {
    pub fn get_path(&self) -> &PathBuf {
        match self {
            Options::Run { file, .. } => file,
            Options::Build { file, .. } => file,
        }
    }
}
