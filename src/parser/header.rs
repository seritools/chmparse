use hex_literal::hex;
use nameof::name_of;
use nom::bytes::complete::tag;
use nom::error::context;
use nom::number::complete::le_u32;
use nom::number::complete::le_u64;
use uuid::Uuid;

use super::NomParseResult;

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
    pub fn parse(i: &[u8]) -> NomParseResult<'_, Header> {
        context(name_of!(type Header), |i| {
            let (i, _) = tag(b"ITSF")(i)?;
            let (i, version) = le_u32(i)?;
            let (i, total_header_length) = le_u32(i)?;
            let (i, unknown_dword) = le_u32(i)?;
            let (i, timestamp) = le_u32(i)?;
            let (i, language_id) = le_u32(i)?;

            let (i, _) = context(
                "first const UUID v1 in header",
                crate::parser::uuid_parse::parse_exact_uuid(HEADER_GUID_1),
            )(i)?;

            let (i, _) = context(
                "second const UUID v1 in header",
                crate::parser::uuid_parse::parse_exact_uuid(HEADER_GUID_2),
            )(i)?;

            let (i, header_section_table) = parse_header_section_table(i)?;

            // TODO: find out when to read `offset_first_content_section`
            // "In Version 2 files, this data is not there and the content section
            // starts immediately after the directory."
            // Does that mean it's V3+ or V1?

            Ok((
                i,
                Self {
                    version,
                    total_header_length,
                    unknown_dword,
                    timestamp,
                    language_id,
                    header_section_table,
                    offset_first_content_section: None,
                },
            ))
        })(i)
    }
}

const HEADER_GUID_1: Uuid = Uuid::from_bytes(hex!("10 FD017CAA7BD0119E0C00A0C922E6EC"));
const HEADER_GUID_2: Uuid = Uuid::from_bytes(hex!("11 FD017CAA7BD0119E0C00A0C922E6EC"));

#[derive(Debug)]
pub struct HeaderSectionTableEntry {
    pub file_offset: u64,
    pub length: u64,
}

impl HeaderSectionTableEntry {
    fn parse(i: &[u8]) -> NomParseResult<'_, Self> {
        context(name_of!(type HeaderSectionTableEntry), |i| {
            let (i, file_offset) = le_u64(i)?;
            let (i, length) = le_u64(i)?;

            Ok((
                i,
                Self {
                    file_offset,
                    length,
                },
            ))
        })(i)
    }
}

pub type HeaderSectionTable = [HeaderSectionTableEntry; 2];

fn parse_header_section_table(i: &[u8]) -> NomParseResult<'_, HeaderSectionTable> {
    context(name_of!(type HeaderSectionTable), |i| {
        let (i, e1) = HeaderSectionTableEntry::parse(i)?;
        let (i, e2) = HeaderSectionTableEntry::parse(i)?;

        Ok((i, [e1, e2]))
    })(i)
}
