extern crate rox;

use rox::run_file;
use std::env::args;
use std::path::Path;
use std::process::exit;

// TODO: Make a more robust CLI with `StructOpt`
fn main() {
    let arguments: Vec<String> = args().collect();
    match arguments.len() {
        2 => run_file(Path::new(&arguments[1])).unwrap(),
        _ => {
            println!("Usage: cargo run path/to/file.rox");
            exit(1);
        }
    }
}
