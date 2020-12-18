use nom::Finish;
use snafu::ResultExt;

use crate::error::*;
use crate::parser::directory_listing::DirectoryListing;
use crate::parser::header::Header;
use crate::parser::header_section_0::HeaderSection0;

#[derive(Debug)]
pub struct ChmFile {
    header: Header,
    header_section_0: HeaderSection0,
    directory_listing: DirectoryListing,
}

impl ChmFile {
    pub fn load(file: &[u8]) -> crate::Result<'_, Self> {
        let i = file;
        let (i, header) = Header::parse(i)
            .finish()
            .map_err(|inner| HeaderParse { inner }.build())?;

        let hs0_entry = &header.header_section_table[0];
        let hs0_offset = hs0_entry.file_offset as usize;
        let hs0_size = hs0_entry.length as usize;

        let hs0_data = &file[hs0_offset..hs0_offset.checked_add(hs0_size).expect("oops")];

        let (_, header_section_0) = HeaderSection0::parse(file.len() as u64)(hs0_data)?;

        // TODO: give section table entries correct names
        // TODO: also add helper fns for the calcs we're doing here
        // directory listing
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
