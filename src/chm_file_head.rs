use std::convert::TryFrom;

use pahs::try_parse;
use pahs_snafu::ProgressSnafuExt;
use snafu::Snafu;

use crate::directory_listing::{DirectoryListing, ParseDirectoryListingError};
use crate::header::{Header, HeaderSectionTableEntry, ParseHeaderError};
use crate::header_section_0::{HeaderSection0, ParseHeaderSection0Error};
use crate::{Driver, Pos, Progress};

#[derive(Debug)]
pub struct ChmFileHead<'a> {
    file: &'a [u8],
    header: Header,
    header_section_0: HeaderSection0,
    pub(crate) directory_listing: DirectoryListing<'a>,
    pub(crate) actual_offset_content_section_0: usize,
}

impl<'a> ChmFileHead<'a> {
    pub fn parse(
        pd: &mut Driver,
        pos: Pos<'a>,
        file: &'a [u8],
    ) -> Progress<'a, Self, ParseChmFileHeadError> {
        let (pos_after_header, header) =
            try_parse!(Header::parse(pd, pos).snafu(|pos| ParseHeader { offset: pos.offset }));

        // The content section 0 usually follows the header (sections).
        // In v2 files this is always the case. In v3 files the offset is specified in
        // the header itself, after the header section table. However, not all files fill this
        // field properly (setting it to 0 instead). In this case we assume the usual position,
        // as this is what the Windows CHM viewer does as well.
        let offset_content_section_0 = header
            .offset_content_section_0
            .map(|o| o as usize)
            .unwrap_or(pos_after_header.offset);

        let (_, (hs0_offset, hs0_data)) = try_parse!(Progress::from_result(
            pos_after_header,
            get_header_section_data(
                "Header section 0",
                file,
                &header.header_section_table.header_section_0,
            )
        ));

        let pos = Pos {
            offset: hs0_offset,
            s: hs0_data,
        };

        let (_, header_section_0) = try_parse!(HeaderSection0::parse(file.len() as u64)(pd, pos)
            .snafu(|_| ParseHeaderSection0 { offset: pos.offset }));

        let (_, (dl_offset, dl_data)) = try_parse!(Progress::from_result(
            pos_after_header,
            get_header_section_data(
                "Directory listing header section",
                file,
                &header.header_section_table.directory_listing_entry,
            )
        ));

        let pos = Pos {
            offset: dl_offset,
            s: dl_data,
        };

        let (pos, directory_listing) = try_parse!(DirectoryListing::parse(pd, pos)
            .snafu(|_| ParseDirectoryListing { offset: pos.offset }));

        Progress::success(
            pos,
            ChmFileHead {
                file,
                header,
                header_section_0,
                directory_listing,
                actual_offset_content_section_0: offset_content_section_0,
            },
        )
    }
}

fn get_header_section_data<'a>(
    what: &'static str,
    file: &'a [u8],
    header_section_table_entry: &HeaderSectionTableEntry,
) -> Result<(usize, &'a [u8]), ParseChmFileHeadError> {
    let offset = usize::try_from(header_section_table_entry.file_offset)
        .map_err(|_| OutOfBounds { what }.build())?;
    let length = usize::try_from(header_section_table_entry.length)
        .map_err(|_| OutOfBounds { what }.build())?;

    Ok((
        offset,
        offset
            .checked_add(length)
            .and_then(|end| file.get(offset..end))
            .ok_or_else(|| OutOfBounds { what }.build())?,
    ))
}

#[derive(Debug, Snafu)]
#[snafu(visibility = "pub")]
pub enum ParseChmFileHeadError {
    #[snafu(display("The header at {:#X} could not be parsed:\n{}", offset, source))]
    ParseHeader {
        offset: usize,
        #[snafu(source(from(ParseHeaderError, Box::new)))]
        source: Box<ParseHeaderError>,
    },

    #[snafu(display("A value for `{}` was out of bounds.", what))]
    OutOfBounds { what: &'static str },

    #[snafu(display(
        "The header section 0 at offset {:#X} could not be parsed:\n{}",
        offset,
        source
    ))]
    ParseHeaderSection0 {
        offset: usize,
        #[snafu(source(from(ParseHeaderSection0Error, Box::new)))]
        source: Box<ParseHeaderSection0Error>,
    },

    #[snafu(display(
        "The directory listing at offset {:#X} could not be parsed:\n{}",
        offset,
        source
    ))]
    ParseDirectoryListing {
        offset: usize,
        #[snafu(source(from(ParseDirectoryListingError, Box::new)))]
        source: Box<ParseDirectoryListingError>,
    },
}
