#![feature(or_patterns)]
#![feature(bindings_after_at)]

use chmparse::{ChmFile, ChmFileError};

const TEST_FILES: &[&str] = &[
    "test-files/appverif.chm",
    "test-files/c_readme.chm",
    "test-files/WINBASE.chm",
    "test-files/7-zip.chm",
];

#[test]
fn it_parses_test_files() {
    for file in TEST_FILES {
        println!("File: {}", file);
        let content = std::fs::read(file).unwrap();

        match &ChmFile::load(&content) {
            Err(
                e
                @
                (ChmFileError::HeaderParse { offset, .. }
                | ChmFileError::DirectoryListingParse { offset, .. }
                | ChmFileError::HeaderSection0Parse { offset, .. }),
            ) => {
                println!("File failed at offset {:#X}.", offset);
                println!("File content:");

                let aligned_offset = offset - (offset % 16);

                if aligned_offset >= 16 {
                    let prev_bytes_offset = aligned_offset - 16;
                    print!(
                        "{}",
                        to_hex_from(
                            &content[prev_bytes_offset..prev_bytes_offset + 16],
                            16,
                            prev_bytes_offset
                        )
                    );
                }

                println!("        \t{}↓↓", str::repeat(" ", 3 * (offset % 16)));
                println!(
                    "{}",
                    to_hex_from(
                        &content[aligned_offset..content.len().min(aligned_offset + 192)],
                        16,
                        aligned_offset
                    )
                );

                println!("{}\n\n", e);
                panic!();
            }
            Err(e) => {
                println!("{}", e);
                panic!();
            }
            Ok(c) => {
                println!("{:#X?}", c);
            }
        };
    }
}

use std::io::Write;
fn to_hex_from(slice: &[u8], chunk_size: usize, mut from: usize) -> String {
    if chunk_size == 0 {
        panic!("chunk_size cannot be 0");
    }

    // {min 8 }\t{----------------3 x chunk_size----------------}\t{  chunk size  }\n
    // 00000000\t00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 \t................\n
    let mut v = Vec::with_capacity(chunk_size * 4 + 9 * (slice.len() / chunk_size));

    for chunk in slice.chunks(chunk_size) {
        v.write_fmt(format_args!("{:08X}", from)).unwrap();
        v.push(b'\t');

        from += chunk_size;

        for byte in chunk {
            v.write_fmt(format_args!("{:02X} ", byte)).unwrap();
        }
        if chunk_size > chunk.len() {
            for _ in 0..(chunk_size - chunk.len()) {
                v.write_all(b"   ").unwrap();
            }
        }
        v.push(b'\t');

        for &byte in chunk {
            if (32..=126).contains(&byte) {
                v.push(byte);
            } else if byte < 32 {
                v.push(b'.');
            } else {
                v.push(b'?');
            }
        }
        v.push(b'\n');
    }

    String::from_utf8(v).unwrap()
}
