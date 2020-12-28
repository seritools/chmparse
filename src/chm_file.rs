use pahs::ParseDriver;
use snafu::{ResultExt, Snafu};

use crate::parser::directory_listing::{DirectoryListing, DirectoryListingParseError};
use crate::parser::header::{Header, HeaderParseError};
use crate::parser::header_section_0::{HeaderSection0, HeaderSection0ParseError};
use crate::parser::Pos;

#[derive(Debug)]
pub struct ChmFile {
    header: Header,
    header_section_0: HeaderSection0,
    directory_listing: DirectoryListing,
}

impl ChmFile {
    pub fn load(file: &'_ [u8]) -> Result<ChmFile, ChmFileError> {
        let pd = &mut ParseDriver::new();

        let pos = Pos::new(file);
        let (_, header) = Header::parse(pd, pos).finish();
        let header = header.context(HeaderParse { offset: pos.offset })?;

        let hs0_entry = &header.header_section_table.header_section_0;
        let hs0_offset = hs0_entry.file_offset as usize;
        let hs0_size = hs0_entry.length as usize;

        let hs0_data = &file[hs0_offset
            ..hs0_offset
                .checked_add(hs0_size)
                .ok_or_else(|| HeaderSection0OutOfBounds.build())?];
        let pos = Pos {
            offset: hs0_offset,
            s: hs0_data,
        };

        let (_, header_section_0) = HeaderSection0::parse(file.len() as u64)(pd, pos).finish();
        let header_section_0 =
            header_section_0.context(HeaderSection0Parse { offset: pos.offset })?;

        let dl_entry = &header.header_section_table.directory_listing_entry;
        let dl_offset = dl_entry.file_offset as usize;
        let dl_size = dl_entry.length as usize;

        let dl_data = &file[dl_offset
            ..dl_offset
                .checked_add(dl_size)
                .ok_or_else(|| DirectoryListingOutOfBounds.build())?];

        let pos = Pos {
            offset: dl_offset,
            s: dl_data,
        };

        let (_, directory_listing) = DirectoryListing::parse(pd, pos).finish();
        let directory_listing =
            directory_listing.context(DirectoryListingParse { offset: pos.offset })?;

        Ok(ChmFile {
            header,
            header_section_0,
            directory_listing,
        })
    }
}

#[derive(Debug, Snafu)]
#[snafu(visibility = "pub")]
pub enum ChmFileError {
    #[snafu(display("The header at {:#X} could not be parsed:\n{}", offset, source))]
    HeaderParse {
        offset: usize,
        #[snafu(source(from(HeaderParseError, Box::new)))]
        source: Box<HeaderParseError>,
    },

    #[snafu(display("The location of the header section 0 was out of bounds."))]
    HeaderSection0OutOfBounds,

    #[snafu(display(
        "The header section 0 at offset {:#X} could not be parsed:\n{}",
        offset,
        source
    ))]
    HeaderSection0Parse {
        offset: usize,
        #[snafu(source(from(HeaderSection0ParseError, Box::new)))]
        source: Box<HeaderSection0ParseError>,
    },

    #[snafu(display("The location of the directory listing was out of bounds."))]
    DirectoryListingOutOfBounds,

    #[snafu(display(
        "The directory listing at offset {:#X} could not be parsed:\n{}",
        offset,
        source
    ))]
    DirectoryListingParse {
        offset: usize,
        #[snafu(source(from(DirectoryListingParseError, Box::new)))]
        source: Box<DirectoryListingParseError>,
    },
}
