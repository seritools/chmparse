use pahs::slice::tag;
use pahs::slice::{num::*, NotEnoughDataError};
use pahs::{sequence, Recoverable};
use pahs_snafu::ProgressSnafuExt;
use snafu::Snafu;

use super::{Driver, Pos, Progress};

#[derive(Debug)]
pub struct HeaderSection0 {
    pub unknown_dword_1: u32,
    pub file_size: u64,
    pub unknown_dword_2: u32,
    pub unknown_dword_3: u32,
}

impl HeaderSection0 {
    pub fn parse<'a>(
        expected_file_size: u64,
    ) -> impl Fn(&mut Driver, Pos<'a>) -> Progress<'a, Self, ParseHeaderSection0Error> {
        const TAG: &[u8; 4] = &[0xFE, 0x01, 0x00, 0x00];

        move |pd, pos| {
            sequence!(
                pd,
                pos,
                {
                    Self::tag(TAG);

                    let unknown_dword_1 = u32_le;
                    let file_size = |pd, p| {
                        u64_le(pd, p)
                            .snafu_leaf(|_| NotEnoughData)
                            .and_then(p, |size| {
                                if size == expected_file_size {
                                    Ok(size)
                                } else {
                                    Err(FileSizeMismatch {
                                        expected: expected_file_size,
                                        parsed: size,
                                    }
                                    .build())
                                }
                            })
                    };

                    let unknown_dword_2 = u32_le;
                    let unknown_dword_3 = u32_le;
                },
                Self {
                    unknown_dword_1,
                    file_size,
                    unknown_dword_2,
                    unknown_dword_3
                }
            )
        }
    }

    fn tag<'a>(
        expected: &'static [u8],
    ) -> impl Fn(&mut Driver, Pos<'a>) -> Progress<'a, &'a [u8], ParseHeaderSection0Error> {
        move |pd, p| {
            tag(expected)(pd, p).snafu_leaf(|pos| InvalidTag {
                offset: pos.offset,
                expected,
            })
        }
    }
}

#[derive(Debug, Snafu)]
pub enum ParseHeaderSection0Error {
    #[snafu(display("Not enough data in the input"))]
    NotEnoughData,

    #[snafu(display("Invalid tag at {:#}, expected: {:X?}", offset, expected))]
    InvalidTag {
        offset: usize,
        expected: &'static [u8],
    },

    #[snafu(display("The file size in the section doesn't match the file size. File size: {:#X}, Parsed: {:#X},", expected, parsed))]
    FileSizeMismatch { expected: u64, parsed: u64 },
}

impl From<NotEnoughDataError> for ParseHeaderSection0Error {
    fn from(_: NotEnoughDataError) -> Self {
        NotEnoughData.build()
    }
}

impl Recoverable for ParseHeaderSection0Error {
    fn recoverable(&self) -> bool {
        match self {
            Self::NotEnoughData => true,
            Self::InvalidTag { .. } => true,
            Self::FileSizeMismatch { .. } => false,
        }
    }
}
