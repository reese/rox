extern crate lalrpop;

use std::env;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use walkdir::WalkDir;

fn main() {
    // Generate parser
    lalrpop::process_root().unwrap();

    // Generate test cases from fixtures
    let out_dir = env::var("OUT_DIR").unwrap();
    let destination = Path::new(&out_dir).join("tests.rs");
    let mut test_file = File::create(&destination).unwrap();
    let test_data_directories = WalkDir::new("./examples/")
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file());
    for entry in test_data_directories {
        write_test(&mut test_file, &entry);
    }
}

fn write_test(test_file: &mut File, directory: &walkdir::DirEntry) {
    let directory = directory.path().canonicalize().unwrap();
    let path = directory.display();
    let test_name = format!(
        "rox_compile_test_{}",
        directory.file_stem().unwrap().to_string_lossy()
    );

    write!(
        test_file,
        include_str!("./tests/test_template"),
        name = test_name,
        path = path
    )
    .unwrap();
}
