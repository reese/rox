extern crate rox;

use rox::{repl, run_file};
use std::env::args;
use std::path::Path;
use std::process::exit;

fn main() {
    let arguments: Vec<String> = args().collect();
    if arguments.len() == 1 {
        repl();
    } else if arguments.len() == 2 {
        run_file(Path::new(&arguments[1])).unwrap();
    } else {
        panic!("Usage: rox [path_to_run]")
    }
    exit(0);
}
