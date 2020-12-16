use nom::bytes::complete::tag;
use nom::error::context;
use nom::number::complete::le_u32;
use nom::number::complete::le_u64;

use super::ParseResult;

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
    pub fn parse(i: &[u8]) -> ParseResult<'_, Header> {
        context("CHM file format header", |i| {
            let (i, _) = tag(b"ITSF")(i)?;
            let (i, version) = le_u32(i)?;
            let (i, total_header_length) = le_u32(i)?;
            let (i, unknown_dword) = le_u32(i)?;
            let (i, timestamp) = le_u32(i)?;
            let (i, language_id) = le_u32(i)?;

            let (i, _) = context(
                "first const uuid v1 in header",
                crate::parser::uuid_parse::parse_exact_uuid_v1(HEADER_GUID_1),
            )(i)?;

            let (i, _) = context(
                "second const uuid v1 in header",
                crate::parser::uuid_parse::parse_exact_uuid_v1(HEADER_GUID_2),
            )(i)?;

            let (i, header_section_table) = parse_header_section_table(i)?;

            let (i, offset_first_content_section) = if version >= 3 {
                let output = le_u64(i)?;
                (output.0, Some(output.1))
            } else {
                (i, None)
            };

            Ok((
                i,
                Self {
                    version,
                    total_header_length,
                    unknown_dword,
                    timestamp,
                    language_id,
                    header_section_table,
                    offset_first_content_section,
                },
            ))
        })(i)
    }
}

const HEADER_GUID_1: [u8; 16] = [
    0x10, 0xfd, 0x1, 0x7c, 0xaa, 0x7b, 0xd0, 0x11, 0x9e, 0xc, 0x0, 0xa0, 0xc9, 0x22, 0xe6, 0xec,
];

const HEADER_GUID_2: [u8; 16] = [
    0x11, 0xfd, 0x1, 0x7c, 0xaa, 0x7b, 0xd0, 0x11, 0x9e, 0xc, 0x0, 0xa0, 0xc9, 0x22, 0xe6, 0xec,
];

#[derive(Debug)]
pub struct HeaderSectionTableEntry {
    pub file_offset: u64,
    pub length: u64,
}

impl HeaderSectionTableEntry {
    fn parse(i: &[u8]) -> ParseResult<'_, Self> {
        let (i, file_offset) = le_u64(i)?;
        let (i, length) = le_u64(i)?;

        Ok((
            i,
            Self {
                file_offset,
                length,
            },
        ))
    }
}

pub type HeaderSectionTable = [HeaderSectionTableEntry; 2];

fn parse_header_section_table(i: &[u8]) -> ParseResult<'_, HeaderSectionTable> {
    let (i, e1) = HeaderSectionTableEntry::parse(i)?;
    let (i, e2) = HeaderSectionTableEntry::parse(i)?;

    Ok((i, [e1, e2]))
}
