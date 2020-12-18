use hex_literal::hex;
use nameof::name_of;
use nom::bytes::complete::tag;
use nom::combinator::map;
use nom::combinator::map_opt;
use nom::combinator::verify;
use nom::error::context;
use nom::number::complete::le_u32;
use uuid::Uuid;

use crate::parser::NomParseResult;

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
    pub fn parse(i: &[u8]) -> NomParseResult<'_, Self> {
        const MINUS_ONE_LE: [u8; 4] = (-1i32).to_le_bytes();

        context(name_of!(type DirectoryHeader), |i| {
            let (i, _) = tag(b"ITSP")(i)?;
            let (i, version) = le_u32(i)?;
            let (i, directory_header_length) = le_u32(i)?;
            let (i, _) = le_u32(i)?;
            let (i, directory_chunk_size) = le_u32(i)?;
            let (i, quickref_density) = le_u32(i)?;
            let (i, index_tree_depth) = map_opt(le_u32, |value| match value {
                1 => Some(IndexTreeDepth::NoIndex),
                2 => Some(IndexTreeDepth::OneLevelOfPMGI),
                _ => None,
            })(i)?;

            let (i, root_index_chunk_number) =
                map(
                    le_u32,
                    |value| if value == u32::MAX { None } else { Some(value) },
                )(i)?;

            let (i, first_pmgl_chunk_number) = le_u32(i)?;
            let (i, last_pmgl_chunk_number) = le_u32(i)?;

            // unknown
            let (i, _) = tag(MINUS_ONE_LE)(i)?;

            let (i, directory_chunk_count) = le_u32(i)?;
            let (i, windows_language_id) = le_u32(i)?;

            let (i, _) = context(
                "directory header guid",
                crate::parser::uuid_parse::parse_exact_uuid(DIRECTORY_HEADER_GUID),
            )(i)?;

            let (i, _) = context(
                "verify: both dir header length are the same",
                verify(le_u32, |&l| directory_header_length == l),
            )(i)?;

            // unknown
            let (i, _) = tag(MINUS_ONE_LE)(i)?;
            let (i, _) = tag(MINUS_ONE_LE)(i)?;
            let (i, _) = tag(MINUS_ONE_LE)(i)?;

            Ok((
                i,
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
                },
            ))
        })(i)
    }
}

#[derive(Debug)]
pub enum IndexTreeDepth {
    NoIndex,
    OneLevelOfPMGI,
}
