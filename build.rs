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

fn write_test(test_file: &mut File, entry: &walkdir::DirEntry) {
    let entry = entry.path().canonicalize().unwrap();
    let path = entry.display();
    let test_name = format!(
        "rox_compile_test{}",
        entry
            .to_str()
            .unwrap()
            // Scrub to a relative path
            .replace(std::env::current_dir().unwrap().to_str().unwrap(), "")
            .replace('/', "_")
            // Remove file ending (assuming this is always .rox)
            .replace(".rox", "")
    );

    write!(
        test_file,
        include_str!("./tests/test_template"),
        name = test_name,
        path = path
    )
    .unwrap();
}
