use std::convert::TryInto;
use std::str::Utf8Error;

use pahs::combinators::zero_or_more;
use pahs::slice::num::u32_le;
use pahs::slice::{tag, NotEnoughDataError};
use pahs::{sequence, try_parse, Recoverable};
use pahs_snafu::ProgressSnafuExt;
use snafu::{IntoError, ResultExt, Snafu};

use crate::encint::{parse_encint_be, ParseEncIntError};
use crate::{Driver, Pos, Progress};

#[derive(Debug)]
pub struct ListingChunk<'a> {
    chunk_index_before: Option<u32>,
    chunk_index_after: Option<u32>,
    pub(crate) entries: Vec<ListingChunkEntry<'a>>,
}

impl<'a> ListingChunk<'a> {
    pub fn parse(
        chunk_size: usize,
    ) -> impl Fn(&mut Driver, Pos<'a>) -> Progress<'a, ListingChunk<'a>, ParseListingChunkError>
    {
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

            let (pos, _) = try_parse!(Self::tag(b"PMGL")(pd, pos));
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

            // always 0 according to russotto's chm format spec, 7-zip.chm has 0D 00 00 00 here
            // the value is unused, so just skip over these 4 bytes
            let (pos, _) = try_parse!(pos.take(4));

            let num_except_minus_one = |pd: &mut _, pos| {
                u32_le(pd, pos).map(|i| if i == 0xFFFF_FFFF { None } else { Some(i) })
            };

            let (pos, chunk_index_before) = try_parse!(num_except_minus_one(pd, pos));
            let (pos, chunk_index_after) = try_parse!(num_except_minus_one(pd, pos));

            let (_, entries) = try_parse!(zero_or_more(ListingChunkEntry::parse)(pd, pos)
                .snafu(|pos| InvalidChunkEntry { offset: pos.offset }));

            Progress::success(
                end_of_chunk,
                Self {
                    chunk_index_before,
                    chunk_index_after,
                    entries,
                },
            )
        }
    }

    fn tag(
        expected: &'static [u8],
    ) -> impl Fn(&mut Driver, Pos<'a>) -> Progress<'a, &'a [u8], ParseListingChunkError> {
        move |pd, p| {
            tag(expected)(pd, p).snafu_leaf(|pos| InvalidTag {
                offset: pos.offset,
                expected,
            })
        }
    }
}

#[derive(Debug, Snafu)]
pub enum ParseListingChunkError {
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
        source: ParseListingChunkEntryError,
    },
}

impl From<NotEnoughDataError> for ParseListingChunkError {
    fn from(_: NotEnoughDataError) -> Self {
        NotEnoughDataInChunk.build()
    }
}

impl Recoverable for ParseListingChunkError {
    fn recoverable(&self) -> bool {
        match self {
            Self::InvalidChunkEntry { source, .. } => source.recoverable(),
            _ => true,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ListingChunkEntry<'a> {
    pub(crate) name: &'a str,
    pub(crate) content_section: u64,
    pub(crate) content_section_offset: u64,
    pub(crate) content_length: u64,
}

impl<'a> ListingChunkEntry<'a> {
    fn parse(pd: &mut Driver, pos: Pos<'a>) -> Progress<'a, Self, ParseListingChunkEntryError> {
        let (pos, name_len) = try_parse!(parse_encint_be(pd, pos).map_err(|e| {
            if let ParseEncIntError::NotEnoughData = e {
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
                };
                let content_section_index =
                    |pd, pos| parse_encint_be(pd, pos).snafu(|_| ContentSectionNumberInvalid);
                let content_section_offset =
                    |pd, pos| parse_encint_be(pd, pos).snafu(|_| ContentSectionOffsetInvalid);
                let content_length =
                    |pd, pos| parse_encint_be(pd, pos).snafu(|_| ContentLengthInvalid);
            },
            ListingChunkEntry {
                name,
                content_section: content_section_index,
                content_section_offset,
                content_length
            }
        )
    }
}

#[derive(Debug, Snafu)]
pub enum ParseListingChunkEntryError {
    NotEnoughData,
    LengthOfNameInvalid { source: ParseEncIntError },
    NameTooLong,
    NameStringOutOfBounds,
    NameInvalidUtf8 { source: Utf8Error },
    ContentSectionNumberInvalid { source: ParseEncIntError },
    ContentSectionOffsetInvalid { source: ParseEncIntError },
    ContentLengthInvalid { source: ParseEncIntError },
}

impl Recoverable for ParseListingChunkEntryError {
    fn recoverable(&self) -> bool {
        matches!(self, Self::NotEnoughData)
    }
}
