use nom::{error::context, error::ContextError, error::ParseError, IResult};

pub mod directory_header;

use directory_header::DirectoryHeader;

#[derive(Debug)]
pub struct DirectoryListing {
    header: DirectoryHeader,
}

impl DirectoryListing {
    pub fn parse<'a, E: ParseError<&'a [u8]> + ContextError<&'a [u8]>>(
        i: &'a [u8],
    ) -> IResult<&'a [u8], Self, E> {
        context("directory listing", |i| {
            let (i, header) = DirectoryHeader::parse(i)?;

            Ok((i, Self { header }))
        })(i)
    }
}
