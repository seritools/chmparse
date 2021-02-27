use listing_chunk::{ListingChunk, ParseListingChunkError};
use pahs::combinators::count_push_into;
use pahs::{try_parse, Push, Recoverable};
use pahs_snafu::ProgressSnafuExt;
use snafu::Snafu;

use super::{Driver, Pos, Progress};
use directory_header::{DirectoryHeader, ParseDirectoryHeaderError};
use index_chunk::{parse_index_chunk, ParseIndexChunkError};

mod directory_header;
mod index_chunk;
pub mod listing_chunk;

enum Chunk<'a> {
    Listing(ListingChunk<'a>),
    Index,
}

#[derive(Default)]
struct OnlyListingChunks<'a>(Vec<ListingChunk<'a>>);

impl<'a> Push<Chunk<'a>> for OnlyListingChunks<'a> {
    fn push(&mut self, value: Chunk<'a>) {
        if let Chunk::Listing(chunk) = value {
            self.0.push(chunk)
        }
    }
}

#[derive(Debug)]
pub struct DirectoryListing<'a> {
    pub header: DirectoryHeader,
    pub entries: Vec<ListingChunk<'a>>,
}

impl<'a> DirectoryListing<'a> {
    pub fn parse(pd: &mut Driver, pos: Pos<'a>) -> Progress<'a, Self, ParseDirectoryListingError> {
        let (pos, header) =
            try_parse!(DirectoryHeader::parse(pd, pos).snafu(|_| DirectoryHeaderParse));

        let (pos, OnlyListingChunks(entries)) = try_parse!(count_push_into(
            header.total_directory_chunk_count as usize,
            OnlyListingChunks::default,
            |pd: &mut Driver, pos| pd
                .alternate(pos)
                .one(
                    |pd, pos| ListingChunk::parse(header.directory_chunk_size as usize)(pd, pos)
                        .snafu(|pos| ListingChunkParse { offset: pos.offset })
                        .map(Chunk::Listing)
                )
                .one(
                    |pd, pos| parse_index_chunk(header.directory_chunk_size as usize)(pd, pos)
                        .snafu(|pos| IndexChunkParse { offset: pos.offset })
                        .map(|_| Chunk::Index)
                )
                .finish()
        )(pd, pos));

        pos.success(Self { header, entries })
    }
}

#[derive(Debug, Snafu)]
pub enum ParseDirectoryListingError {
    #[snafu(display("Failed to parse the directory header:\n{}", source))]
    DirectoryHeaderParse { source: ParseDirectoryHeaderError },

    #[snafu(display("Failed to parse a listing chunk at {:#X}:\n{}", offset, source))]
    ListingChunkParse {
        offset: usize,
        source: ParseListingChunkError,
    },

    #[snafu(display("Failed to parse an index chunk at {:#X}:\n{}", offset, source))]
    IndexChunkParse {
        offset: usize,
        source: ParseIndexChunkError,
    },
}

impl Recoverable for ParseDirectoryListingError {
    fn recoverable(&self) -> bool {
        match self {
            Self::DirectoryHeaderParse { source, .. } => source.recoverable(),
            Self::ListingChunkParse { source, .. } => source.recoverable(),
            Self::IndexChunkParse { source, .. } => source.recoverable(),
        }
    }
}
