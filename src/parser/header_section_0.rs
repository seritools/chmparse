use nom::bytes::complete::tag;
use nom::combinator::verify;
use nom::error::context;
use nom::number::complete::le_u32;
use nom::number::complete::le_u64;

use super::ParseResult;

#[derive(Debug)]
pub struct HeaderSection0 {
    pub unknown_dword_1: u32,
    /// Not all files set this correctly
    pub file_size: u64,
}

impl HeaderSection0 {
    pub fn parse(expected_file_size: u64) -> impl Fn(&[u8]) -> ParseResult<'_, Self> {
        move |i| {
            context("Header Section 0", |i| {
                let (i, _) = context("tag, always 0x0000_01FE", tag(&[0xFE, 0x01, 0x00, 0x00]))(i)?;
                let (i, unknown_dword_1) = context("unknown dword 1", le_u32)(i)?;
                let (i, file_size) = context(
                    "file size equals saved size in header section 0",
                    verify(le_u64, |&size| size == expected_file_size),
                )(i)?;
                let (i, _) = context("tag, always zero", tag(&[0u8; 4]))(i)?;
                let (i, _) = context("tag, always zero", tag(&[0u8; 4]))(i)?;

                Ok((
                    i,
                    Self {
                        unknown_dword_1,
                        file_size,
                    },
                ))
            })(i)
        }
    }
}
