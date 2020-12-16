mod directory_header;
pub use directory_header::DirectoryHeader;

use super::ParseResult;

use nom::error::context;

#[derive(Debug)]
pub struct DirectoryListing {
    pub header: DirectoryHeader,
}

impl DirectoryListing {
    pub fn parse(i: &[u8]) -> ParseResult<'_, Self> {
        context("directory listing", |i| {
            let (i, header) = DirectoryHeader::parse(i)?;

            Ok((i, Self { header }))
        })(i)
    }
}
