use nom::{combinator::verify, error::context, error::ContextError, error::ParseError, IResult};

use crate::directory_listing::DirectoryListing;
use crate::header::Header;
use crate::header_section_0::HeaderSection0;

#[derive(Debug)]
pub struct ChmFile {
    header: Header,
    header_section_0: HeaderSection0,
    directory_listing: DirectoryListing,
}

impl ChmFile {
    pub fn parse<'a, E: ParseError<&'a [u8]> + ContextError<&'a [u8]>>(
        file: &'a [u8],
    ) -> IResult<&'a [u8], Self, E> {
        let i = file;
        let (i, header) = Header::parse(i)?;

        let hs0_entry = &header.header_section_table[0];
        let hs0_offset = hs0_entry.file_offset as usize;
        let hs0_size = hs0_entry.length as usize;

        let hs0_data = &file[hs0_offset
            ..hs0_offset
                .checked_add(hs0_size)
                .expect("overflow calculating end position of header section 0")];

        let (_, header_section_0) = context(
            "verify: file size equals saved size in header section",
            verify(HeaderSection0::parse, |hs0| {
                file.len() as u64 == hs0.file_size
            }),
        )(hs0_data)?;

        // TODO: give section table entries correct names
        // TODO: also add helper fns for the calcs we're doing here
        let dl_entry = &header.header_section_table[1];
        let dl_offset = dl_entry.file_offset as usize;
        let dl_size = dl_entry.length as usize;

        let dl_data = &file[dl_offset
            ..dl_offset
                .checked_add(dl_size)
                .expect("overflow calculating end position of directory listing")];

        let (_, directory_listing) = DirectoryListing::parse(dl_data)?;

        Ok((
            i,
            Self {
                header,
                header_section_0,
                directory_listing,
            },
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_FILES: &[&str] = &[
        "test-files/appverif.chm",
        "test-files/c_readme.chm",
        "test-files/WINBASE.chm",
        "test-files/7-zip.chm",
    ];

    #[test]
    fn it_works() {
        for file in TEST_FILES {
            println!("file: {}", file);
            let content = std::fs::read(file).unwrap();
            let result = ChmFile::parse(&content);

            if let Ok((_, header)) = result {
                println!("{:#X?}", &header);
            } else {
                crate::dbg_helper::print_err(&content, result)
            }
        }
    }
}
