use nom::{
    bytes::complete::tag, combinator::map, combinator::map_opt, combinator::verify, error::context,
    error::ContextError, error::ParseError, number::complete::le_u32, IResult,
};

const DIRECTORY_HEADER_GUID: [u8; 16] = [
    0x6a, 0x92, 0x02, 0x5d, 0x2e, 0x21, 0xd0, 0x11, 0x9d, 0xf9, 0x00, 0xa0, 0xc9, 0x22, 0xe6, 0xec,
];

#[derive(Debug)]
pub struct DirectoryHeader {
    version: u32,
    directory_header_length: u32,
    directory_chunk_size: u32,
    quickref_density: u32,
    index_tree_depth: IndexTreeDepth,
    root_index_chunk_number: Option<u32>,
    first_pmgl_chunk_number: u32,
    last_pmgl_chunk_number: u32,
    directory_chunk_count: u32,
    windows_language_id: u32,
}

impl DirectoryHeader {
    pub fn parse<'a, E: ParseError<&'a [u8]> + ContextError<&'a [u8]>>(
        i: &'a [u8],
    ) -> IResult<&'a [u8], Self, E> {
        context("directory listing", |i| {
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
            let (i, _) = le_u32(i)?;

            let (i, directory_chunk_count) = le_u32(i)?;
            let (i, windows_language_id) = le_u32(i)?;

            let (i, _) = context(
                "directory header guid",
                crate::uuid_parse::parse_exact_uuid_v1(DIRECTORY_HEADER_GUID),
            )(i)?;

            let (i, _) = context(
                "verify: both dir header length are the same",
                verify(le_u32, |&l| directory_header_length == l),
            )(i)?;

            // unknown
            let (i, _) = le_u32(i)?;
            let (i, _) = le_u32(i)?;
            let (i, _) = le_u32(i)?;

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
