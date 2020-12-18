#![feature(or_patterns)]

use chmparse::ChmFile;

use nom::HexDisplay;
use nom::Offset;

const TEST_FILES: &[&str] = &[
    "test-files/appverif.chm",
    "test-files/c_readme.chm",
    "test-files/WINBASE.chm",
    "test-files/7-zip.chm",
];

#[test]
fn it_parses_test_files() {
    for file in TEST_FILES {
        let content = std::fs::read(file).unwrap();

        match ChmFile::load(&content) {
            Err(nom::Err::Error(e) | nom::Err::Failure(e)) => {
                let offset = content.offset(e.input());
                println!("File '{}' failed at offset {:#X}.", file, offset);
                println!("File content:");

                let aligned_offset = offset - (offset % 16);

                if aligned_offset >= 16 {
                    let prev_bytes_offset = aligned_offset - 16;
                    print!(
                        "{}",
                        content[prev_bytes_offset..prev_bytes_offset + 16]
                            .to_hex_from(16, prev_bytes_offset)
                    );
                }

                println!("        \t{}↓↓", str::repeat(" ", 3 * (offset % 16)));
                println!(
                    "{}",
                    content[aligned_offset..content.len().min(aligned_offset + 128)]
                        .to_hex_from(16, aligned_offset)
                );

                println!("{}", e);
                panic!();
            }
            Err(nom::Err::Incomplete(_)) => unreachable!(),
            Ok(_) => {}
        };
    }
}
