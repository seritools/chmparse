#![forbid(rust_2018_idioms)]
#![deny(nonstandard_style)]

mod chm_file;
mod error;
mod parser;

pub use chm_file::ChmFile;
pub use error::{Error, Result};
