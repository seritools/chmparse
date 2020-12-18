use nameof::name_of;
use nom::error::context;

mod directory_header;
pub use directory_header::DirectoryHeader;

use super::NomParseResult;

#[derive(Debug)]
pub struct DirectoryListing {
    pub header: DirectoryHeader,
}

impl DirectoryListing {
    pub fn parse(i: &[u8]) -> NomParseResult<'_, Self> {
        context(name_of!(type DirectoryListing), |i| {
            let (i, header) = DirectoryHeader::parse(i)?;

            Ok((i, Self { header }))
        })(i)
    }
}
