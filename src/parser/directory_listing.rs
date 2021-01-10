use listing_chunk::{ListingChunk, ListingChunkParseError};
use pahs::combinators::{count, count_push_into};
use pahs::try_parse;
use snafu::Snafu;

use directory_header::{DirectoryHeader, DirectoryHeaderParseError};

use super::{Driver, Pos, Progress};

mod directory_header;
mod index_chunk;
mod listing_chunk;

#[derive(Debug)]
pub struct DirectoryListing {
    pub header: DirectoryHeader,
    pub entries: Vec<ListingChunk>,
}

impl DirectoryListing {
    pub fn parse<'a>(
        pd: &mut Driver,
        pos: Pos<'a>,
    ) -> Progress<'a, Self, DirectoryListingParseError> {
        let (pos, header) =
            try_parse!(DirectoryHeader::parse(pd, pos).snafu(|_| DirectoryHeaderParse));

        let (pos, entries) = try_parse!(count_push_into(
            header.total_directory_chunk_count as usize,
            Vec::new,
            |pd, pos| ListingChunk::parse(header.directory_chunk_size as usize)(pd, pos)
                .snafu(|pos| ListingChunkParse { offset: pos.offset })
        )(pd, pos));

        pos.success(Self { header, entries })
    }
}

#[derive(Debug, Snafu)]
pub enum DirectoryListingParseError {
    #[snafu(display("Failed to parse the directory header:\n{}", source))]
    DirectoryHeaderParse { source: DirectoryHeaderParseError },

    #[snafu(display("Failed to parse a listing chunk at {:#X}:\n{}", offset, source))]
    ListingChunkParse {
        offset: usize,
        source: ListingChunkParseError,
    },
}
