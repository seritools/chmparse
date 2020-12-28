#![forbid(rust_2018_idioms)]
#![deny(nonstandard_style)]

mod chm_file;

pub use chm_file::{ChmFile, ChmFileError};

mod parser;
