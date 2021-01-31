use index_chunk::IndexChunk;
use listing_chunk::{ListingChunk, ListingChunkParseError};
use pahs::combinators::count_push_into;
use pahs::{try_parse, Recoverable};
use snafu::Snafu;

use directory_header::{DirectoryHeader, DirectoryHeaderParseError};

use self::index_chunk::IndexChunkParseError;

use super::{Driver, Pos, Progress};

mod directory_header;
mod index_chunk;
mod listing_chunk;

#[derive(Debug)]
pub enum DirectoryListingChunk {
    ListingChunk(ListingChunk),
    IndexChunk(IndexChunk),
}

#[derive(Debug)]
pub struct DirectoryListing {
    pub header: DirectoryHeader,
    pub entries: Vec<DirectoryListingChunk>,
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
            |pd: &mut Driver, pos| pd
                .alternate(pos)
                .one(
                    |pd, pos| ListingChunk::parse(header.directory_chunk_size as usize)(pd, pos)
                        .snafu(|pos| ListingChunkParse { offset: pos.offset })
                        .map(DirectoryListingChunk::ListingChunk)
                )
                .one(
                    |pd, pos| IndexChunk::parse(header.directory_chunk_size as usize)(pd, pos)
                        .snafu(|pos| IndexChunkParse { offset: pos.offset })
                        .map(DirectoryListingChunk::IndexChunk)
                )
                .finish()
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

    #[snafu(display("Failed to parse an index chunk at {:#X}:\n{}", offset, source))]
    IndexChunkParse {
        offset: usize,
        source: IndexChunkParseError,
    },
}

impl Recoverable for DirectoryListingParseError {
    fn recoverable(&self) -> bool {
        match self {
            Self::DirectoryHeaderParse { source, .. } => source.recoverable(),
            Self::ListingChunkParse { source, .. } => source.recoverable(),
            Self::IndexChunkParse { source, .. } => source.recoverable(),
        }
    }
}
