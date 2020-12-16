#![feature(or_patterns)]

use chmparse::parser::ChmFile;

const TEST_FILES: &[&str] = &[
    "test-files/appverif.chm",
    "test-files/c_readme.chm",
    "test-files/WINBASE.chm",
    "test-files/7-zip.chm",
];

#[test]
fn it_parses_test_files() {
    for file in TEST_FILES {
        println!("file: {}", file);
        let content = std::fs::read(file).unwrap();
        match ChmFile::parse(&content) {
            Err(nom::Err::Error(e) | nom::Err::Failure(e)) => {
                println!("{}", e);
                panic!();
            }
            Err(e) => {
                println!("{}", e);
                panic!();
            }
            _ => {}
        };
    }
}
