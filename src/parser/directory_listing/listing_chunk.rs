use pahs::slice::num::u32_le;
use pahs::slice::{tag, NotEnoughDataError};
use pahs::try_parse;
use snafu::Snafu;

use crate::parser::{Driver, Pos, Progress};

#[derive(Debug)]
pub struct ListingChunk {
    entries: Vec<ListingEntry>,
}

impl ListingChunk {
    pub fn parse<'a>(
        chunk_size: usize,
    ) -> impl Fn(&mut Driver, Pos<'a>) -> Progress<'a, Self, ListingChunkParseError> {
        move |pd, pos| {
            let (end, chunk_data) =
                try_parse!(pos.take(chunk_size).snafu_leaf(|pos| ChunkOutOfBounds {
                    offset: pos.offset,
                    chunk_size
                }));

            {
                let pos = Pos {
                    offset: pos.offset,
                    s: chunk_data,
                };

                let (pos, _) = try_parse!(Self::tag(b"PMGL")(pd, pos));
                let (pos, quickref_len) = try_parse!(u32_le(pd, pos));
                let (pos, _) = try_parse!(Self::tag(&[0, 0, 0, 0])(pd, pos));

                let num_except_minus_one = |pd: &mut _, pos| {
                    u32_le(pd, pos).map(|i| if i == 0xFFFF_FFFF { None } else { Some(i) })
                };

                let (pos, chunk_index_before) = try_parse!(num_except_minus_one(pd, pos));
                let (pos, chunk_index_after) = try_parse!(num_except_minus_one(pd, pos));

                // TODO: parse dir entries

                // TODO: actually parse/validate quickref area
                let (pos, _) = try_parse!(pos.take(quickref_len as usize));
            }

            Progress::success(end, Self { entries: vec![] })
        }
    }

    fn tag<'a>(
        expected: &'static [u8],
    ) -> impl Fn(&mut Driver, Pos<'a>) -> Progress<'a, &'a [u8], ListingChunkParseError> {
        move |pd, p| {
            tag(expected)(pd, p).snafu_leaf(|pos| InvalidTag {
                offset: pos.offset,
                expected,
            })
        }
    }
}

#[derive(Debug, Snafu)]
pub enum ListingChunkParseError {
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
}

impl From<NotEnoughDataError> for ListingChunkParseError {
    fn from(_: NotEnoughDataError) -> Self {
        NotEnoughDataInChunk.build()
    }
}

#[derive(Debug)]
pub struct ListingEntry {
    name: String,
    content_section_index: usize,
    content_section_offset: usize,
    length: usize,
}
