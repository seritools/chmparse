mod error;

pub mod directory_listing;
pub mod header;
pub mod header_section_0;
pub mod uuid_parse;

use nom::IResult;

pub use error::Error;

pub type NomParseResult<'a, T> = IResult<&'a [u8], T, Error<'a>>;
