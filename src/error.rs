use snafu::Snafu;

#[derive(Debug, Snafu)]
#[snafu(visibility = "pub(crate)")]
pub enum Error<'a> {
    #[snafu(display("Header parse error: {}", inner))]
    HeaderParse { inner: crate::parser::Error<'a> },
}

pub type Result<'a, T> = std::result::Result<T, Error<'a>>;
