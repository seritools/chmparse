mod error;

mod chm_file;
mod directory_listing;
mod header;
mod header_section_0;
mod uuid_parse;

use nom::IResult;

pub use chm_file::ChmFile;
pub use error::Error;

type ParseResult<'a, T> = IResult<&'a [u8], T, Error<'a>>;
