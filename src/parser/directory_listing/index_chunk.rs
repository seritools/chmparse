use std::convert::TryInto;
use std::str::Utf8Error;

use pahs::combinators::zero_or_more;
use pahs::slice::num::u32_le;
use pahs::slice::{tag, NotEnoughDataError};
use pahs::{sequence, try_parse, Recoverable};
use snafu::{IntoError, ResultExt, Snafu};

use crate::parser::encint::{parse_encint_be, EncIntParseError};
use crate::parser::{Driver, Pos, Progress};

#[derive(Debug)]
pub struct IndexChunk {
    entries: Vec<IndexChunkEntry>,
}

impl IndexChunk {
    pub fn parse<'a>(
        chunk_size: usize,
    ) -> impl Fn(&mut Driver, Pos<'a>) -> Progress<'a, Self, IndexChunkParseError> {
        move |pd, pos| {
            let (end_of_chunk, chunk_data) =
                try_parse!(pos.take(chunk_size).snafu_leaf(|pos| ChunkOutOfBounds {
                    offset: pos.offset,
                    chunk_size
                }));

            let pos = Pos {
                s: chunk_data,
                ..pos
            };

            let (pos, _) = try_parse!(Self::tag(b"PMGI")(pd, pos));
            let (pos, quickref_len) = try_parse!(u32_le(pd, pos));

            let len_of_rest_of_non_quickref_area = match pos
                .s
                .len()
                .checked_sub(quickref_len as usize)
                .ok_or_else(|| NotEnoughDataInChunk.build())
            {
                Ok(len) => len,
                Err(e) => return Progress::failure(pos, e),
            };

            let (_quickref_pos, rest_of_non_quickref_area) =
                pos.take(len_of_rest_of_non_quickref_area).unwrap();

            // TODO: actually parse/validate quickref area
            // let (_, _) = try_parse!(quickref_pos.take(quickref_len as usize));

            let pos = Pos {
                s: rest_of_non_quickref_area,
                ..pos
            };

            let (_, entries) = try_parse!(zero_or_more(IndexChunkEntry::parse)(pd, pos)
                .snafu(|pos| InvalidChunkEntry { offset: pos.offset }));

            Progress::success(end_of_chunk, Self { entries })
        }
    }

    fn tag<'a>(
        expected: &'static [u8],
    ) -> impl Fn(&mut Driver, Pos<'a>) -> Progress<'a, &'a [u8], IndexChunkParseError> {
        move |pd, p| {
            tag(expected)(pd, p).snafu_leaf(|pos| InvalidTag {
                offset: pos.offset,
                expected,
            })
        }
    }
}

#[derive(Debug, Snafu)]
pub enum IndexChunkParseError {
    #[snafu(display("Not enough data in the chunk"))]
    NotEnoughDataInChunk,

    #[snafu(display(
        "Not enough data for chunk (chunk size: {}) at {:#X}",
        chunk_size,
        offset
    ))]
    ChunkOutOfBounds { offset: usize, chunk_size: usize },

    #[snafu(display("Invalid tag at {:#X}, expected: {:?}", offset, expected))]
    InvalidTag {
        offset: usize,
        expected: &'static [u8],
    },

    #[snafu(display("Invalid listing chunk entry at {:#X}:\n{}", offset, source))]
    InvalidChunkEntry {
        offset: usize,
        source: IndexChunkEntryParseError,
    },
}

impl From<NotEnoughDataError> for IndexChunkParseError {
    fn from(_: NotEnoughDataError) -> Self {
        NotEnoughDataInChunk.build()
    }
}

impl Recoverable for IndexChunkParseError {
    fn recoverable(&self) -> bool {
        match self {
            Self::InvalidChunkEntry { source, .. } => source.recoverable(),
            _ => true,
        }
    }
}

#[derive(Debug)]
pub struct IndexChunkEntry {
    name: String,
    listing_chunk_starting_with_name: u64,
}

impl IndexChunkEntry {
    fn parse<'a>(pd: &mut Driver, pos: Pos<'a>) -> Progress<'a, Self, IndexChunkEntryParseError> {
        let (pos, name_len) = try_parse!(parse_encint_be(pd, pos).map_err(|e| {
            if let EncIntParseError::NotEnoughData = e {
                NotEnoughData.build()
            } else {
                LengthOfNameInvalid.into_error(e)
            }
        }));

        let name_len = match name_len.try_into() {
            Ok(l) => l,
            Err(_) => return Progress::failure(pos, NameTooLong.build()),
        };

        sequence!(
            pd,
            pos,
            {
                let name = |_, pos: Pos<'a>| {
                    pos.take(name_len)
                        .snafu_leaf(|_| NameStringOutOfBounds)
                        .and_then(pos, |s| std::str::from_utf8(s).context(NameInvalidUtf8))
                        .map(|s| s.to_owned())
                };
                let listing_chunk_starting_with_name =
                    |pd, pos| parse_encint_be(pd, pos).snafu(|_| ChunkNumberInvalid);
            },
            Self {
                name,
                listing_chunk_starting_with_name,
            }
        )
    }
}

#[derive(Debug, Snafu)]
pub enum IndexChunkEntryParseError {
    NotEnoughData,
    LengthOfNameInvalid { source: EncIntParseError },
    NameTooLong,
    NameStringOutOfBounds,
    NameInvalidUtf8 { source: Utf8Error },
    ChunkNumberInvalid { source: EncIntParseError },
}

impl Recoverable for IndexChunkEntryParseError {
    fn recoverable(&self) -> bool {
        matches!(self, Self::NotEnoughData)
    }
}
