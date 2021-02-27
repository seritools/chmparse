#![forbid(rust_2018_idioms)]
#![deny(nonstandard_style)]

mod chm_file;
mod chm_file_head;
mod name_list;

pub use chm_file::{ChmFile, ParseChmFileError};
pub use chm_file_head::{ChmFileHead, ParseChmFileHeadError};

mod directory_listing;
mod encint;
mod header;
mod header_section_0;
mod uuid;

#[derive(Debug, Default)]
pub struct ParseState {
    pub warnings: Vec<(usize, &'static str)>,
}

pub type Pos<'a> = pahs::slice::BytePos<'a>;
type Progress<'a, T, E> = pahs::Progress<Pos<'a>, T, E>;
pub type Driver = pahs::ParseDriver<ParseState>;
