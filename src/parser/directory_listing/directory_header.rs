use hex_literal::hex;
use pahs::slice::num::u32_le;
use pahs::slice::{tag, NotEnoughDataError};
use pahs::{sequence, Recoverable};
use snafu::Snafu;
use uuid::Uuid;

use crate::parser::uuid::{parse_exact_uuid, ExactUuidError};
use crate::parser::{Driver, Pos, Progress};

const DIRECTORY_HEADER_GUID: Uuid = Uuid::from_bytes(hex!("6A92025D2E21D0119DF900A0C922E6EC"));

#[derive(Debug)]
pub struct DirectoryHeader {
    pub version: u32,
    pub directory_header_length: u32,
    pub directory_chunk_size: u32,
    pub quickref_density: u32,
    pub index_tree_depth: IndexTreeDepth,
    pub root_index_chunk_number: Option<u32>,
    pub first_pmgl_chunk_number: u32,
    pub last_pmgl_chunk_number: u32,
    pub directory_chunk_count: u32,
    pub windows_language_id: u32,
}

impl DirectoryHeader {
    pub fn parse<'a>(
        pd: &mut Driver,
        pos: Pos<'a>,
    ) -> Progress<'a, Self, DirectoryHeaderParseError> {
        const MINUS_ONE: &[u8; 4] = &(-1i32).to_le_bytes();

        sequence!(
            pd,
            pos,
            {
                Self::tag(b"ITSP");
                let version = u32_le;
                let directory_header_length = u32_le;

                // unknown
                u32_le;

                let directory_chunk_size = u32_le;
                let quickref_density = u32_le;
                let index_tree_depth = |pd, p| {
                    u32_le(pd, p)
                        .snafu_leaf(|_| NotEnoughData)
                        .and_then(p, |value| match value {
                            1 => Ok(IndexTreeDepth::NoIndex),
                            2 => Ok(IndexTreeDepth::OneLevelOfPMGI),
                            _ => Err(UnknownIndexTreeDepth.build()),
                        })
                };

                let root_index_chunk_number =
                    |pd, p| u32_le(pd, p).map(|val| if val == u32::MAX { None } else { Some(val) });

                let first_pmgl_chunk_number = u32_le;
                let last_pmgl_chunk_number = u32_le;

                Self::tag(MINUS_ONE);

                let directory_chunk_count = u32_le;
                let windows_language_id = u32_le;

                parse_exact_uuid(DIRECTORY_HEADER_GUID, |_| UuidFailed);

                let _ = |pd, p| {
                    u32_le(pd, p)
                        .snafu_leaf(|_| NotEnoughData)
                        .and_then(p, |len2| {
                            if directory_header_length == len2 {
                                Ok(len2)
                            } else {
                                Err(DirectoryHeaderLengthsDoNotMatch {
                                    first: directory_header_length,
                                    second: len2,
                                }
                                .build())
                            }
                        })
                };

                // unknown
                Self::tag(MINUS_ONE);
                Self::tag(MINUS_ONE);
                Self::tag(MINUS_ONE);
            },
            Self {
                version,
                directory_header_length,
                directory_chunk_size,
                quickref_density,
                index_tree_depth,
                root_index_chunk_number,
                first_pmgl_chunk_number,
                last_pmgl_chunk_number,
                directory_chunk_count,
                windows_language_id,
            }
        )
    }

    fn tag<'a>(
        expected: &'static [u8],
    ) -> impl Fn(&mut Driver, Pos<'a>) -> Progress<'a, &'a [u8], DirectoryHeaderParseError> {
        move |pd, p| {
            tag(expected)(pd, p).snafu_leaf(|pos| InvalidTag {
                offset: pos.offset,
                expected,
            })
        }
    }
}

#[derive(Debug, Snafu)]
pub enum DirectoryHeaderParseError {
    #[snafu(display("Not enough data in the input"))]
    NotEnoughData,

    #[snafu(display("Unknown specified index tree depth"))]
    UnknownIndexTreeDepth,

    #[snafu(display("Invalid tag at {:#X}, expected: {:?}", offset, expected))]
    InvalidTag {
        offset: usize,
        expected: &'static [u8],
    },

    #[snafu(display("Failed to parse an exact Uuid:\n{}", source))]
    UuidFailed { source: ExactUuidError },

    #[snafu(display("The two fields specifying the directory header length do not match (first: {:#X}, second: {:#X})", first, second))]
    DirectoryHeaderLengthsDoNotMatch { first: u32, second: u32 },
}

impl From<NotEnoughDataError> for DirectoryHeaderParseError {
    fn from(_: NotEnoughDataError) -> Self {
        NotEnoughData.build()
    }
}

impl Recoverable for DirectoryHeaderParseError {
    fn recoverable(&self) -> bool {
        match self {
            Self::NotEnoughData => true,
            Self::UuidFailed { .. } => true,
            Self::InvalidTag { .. } => true,

            Self::UnknownIndexTreeDepth => false,
            Self::DirectoryHeaderLengthsDoNotMatch { .. } => false,
        }
    }
}

#[derive(Debug)]
pub enum IndexTreeDepth {
    NoIndex,
    OneLevelOfPMGI,
}
