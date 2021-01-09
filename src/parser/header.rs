use hex_literal::hex;
use pahs::slice::num::{u32_le, u64_le};
use pahs::slice::{tag, NotEnoughDataError};
use pahs::{sequence, ParseDriver, Recoverable};
use snafu::Snafu;
use uuid::Uuid;

use super::uuid::parse_exact_uuid;
use super::{Driver, Pos, Progress};

const HEADER_GUID_1: Uuid = Uuid::from_bytes(hex!("10 FD017CAA7BD0119E0C00A0C922E6EC"));
const HEADER_GUID_2: Uuid = Uuid::from_bytes(hex!("11 FD017CAA7BD0119E0C00A0C922E6EC"));

#[derive(Debug)]
pub struct Header {
    pub version: u32,
    pub total_header_length: u32,
    pub unknown_dword: u32,
    pub timestamp: u32,
    pub language_id: u32,
    pub header_section_table: HeaderSectionTable,
    pub offset_first_content_section: Option<u64>,
}

impl Header {
    pub fn parse<'a>(pd: &mut Driver, pos: Pos<'a>) -> Progress<'a, Self, HeaderParseError> {
        const TAG: &[u8; 4] = b"ITSF";

        sequence!(
            pd,
            pos,
            {
                Self::tag(TAG);
                let version = u32_le;
                let total_header_length = u32_le;
                let unknown_dword = u32_le;
                let timestamp = u32_le;
                let language_id = u32_le;

                parse_exact_uuid(HEADER_GUID_1, |_| UuidFailed);
                parse_exact_uuid(HEADER_GUID_2, |_| UuidFailed);

                let header_section_table =
                    |pd, p| HeaderSectionTable::parse(pd, p).snafu(|_| HeaderSectionTableFailed);

                let offset_first_content_section = |pd, p| {
                    if version > 2 {
                        u64_le(pd, p).map(Some)
                    } else {
                        // v2 does not have this field
                        p.success(None)
                    }
                };
            },
            Header {
                version,
                total_header_length,
                unknown_dword,
                timestamp,
                language_id,
                header_section_table,
                offset_first_content_section
            }
        )
    }

    fn tag<'a>(
        expected: &'static [u8],
    ) -> impl Fn(&mut Driver, Pos<'a>) -> Progress<'a, &'a [u8], HeaderParseError> {
        move |pd, p| {
            tag(expected)(pd, p).snafu_leaf(|pos| InvalidTag {
                offset: pos.offset,
                expected,
            })
        }
    }
}

#[derive(Debug, Snafu)]
pub enum HeaderParseError {
    #[snafu(display("Not enough data in the input"))]
    NotEnoughData,

    #[snafu(display("Invalid tag at {:#X}, expected: {:?}", offset, expected))]
    InvalidTag {
        offset: usize,
        expected: &'static [u8],
    },

    #[snafu(display("Failed to parse an exact Uuid:\n{}", source))]
    UuidFailed { source: super::uuid::ExactUuidError },

    #[snafu(display("Failed to parse the header section table:\n{}", source))]
    HeaderSectionTableFailed { source: HeaderSectionTableError },
}

impl Recoverable for HeaderParseError {
    fn recoverable(&self) -> bool {
        match self {
            Self::NotEnoughData => true,
            Self::InvalidTag { .. } => true,
            Self::UuidFailed { source } => source.recoverable(),
            Self::HeaderSectionTableFailed { source } => source.recoverable(),
        }
    }
}

impl From<NotEnoughDataError> for HeaderParseError {
    fn from(_: NotEnoughDataError) -> Self {
        NotEnoughData.build()
    }
}

#[derive(Debug)]
pub struct HeaderSectionTableEntry {
    pub file_offset: u64,
    pub length: u64,
}

impl HeaderSectionTableEntry {
    fn parse<'a>(
        pd: &mut ParseDriver,
        pos: Pos<'a>,
    ) -> Progress<'a, Self, HeaderSectionTableError> {
        sequence!(
            pd,
            pos,
            {
                let file_offset = u64_le;
                let length = u64_le;
            },
            Self {
                file_offset,
                length
            }
        )
        .into_snafu_leaf(|_: NotEnoughDataError, pos| HeaderSectionTableContext {
            offset: pos.offset,
        })
    }
}

#[derive(Debug, Snafu)]
#[snafu(display(
    "Not enough data to parse header section table entry at offset {:#X}",
    offset
))]
pub struct HeaderSectionTableError {
    offset: usize,
}

impl Recoverable for HeaderSectionTableError {
    fn recoverable(&self) -> bool {
        true
    }
}

#[derive(Debug)]
pub struct HeaderSectionTable {
    pub header_section_0: HeaderSectionTableEntry,
    pub directory_listing_entry: HeaderSectionTableEntry,
}

impl HeaderSectionTable {
    fn parse<'a>(pd: &mut Driver, pos: Pos<'a>) -> Progress<'a, Self, HeaderSectionTableError> {
        sequence!(
            pd,
            pos,
            {
                let header_section_0 = HeaderSectionTableEntry::parse;
                let directory_listing_entry = HeaderSectionTableEntry::parse;
            },
            Self {
                header_section_0,
                directory_listing_entry
            }
        )
    }
}
