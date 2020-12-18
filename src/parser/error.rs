use std::fmt::Debug;
use std::fmt::Display;

use nom::error::ContextError;
use nom::error::FromExternalError;
use nom::error::ParseError;
use snafu::IntoError;
use snafu::Snafu;

#[derive(Debug, Snafu)]
#[snafu(visibility = "pub(crate)")]
pub enum ExternalParseError {
    #[snafu(display("Uuid v1 parse error: {}", source))]
    UuidFail { source: uuid::Error },
}

#[derive(Debug, Snafu)]
pub enum Error<'a> {
    #[snafu(display(
        "Parse error '{}' ({})\n    at {}{}{}",
        format!("{:?}", kind),
        kind.description(),
        input,
        context,
        prev
    ))]
    NomError {
        input: Input<'a>,
        kind: nom::error::ErrorKind,
        prev: PrevError<'a>,
        context: Context,
    },
    #[snafu(display(
        "Parse error '{}' ({})\n    at {}\n    External error:\n        {}{}{}",
        format!("{:?}", kind),
        kind.description(),
        input,
        source,
        context,
        prev
    ))]
    ExternalError {
        input: Input<'a>,
        kind: nom::error::ErrorKind,
        prev: PrevError<'a>,
        context: Context,
        #[snafu(source(from(ExternalParseError, Box::new)))]
        source: Box<ExternalParseError>,
    },
}

impl Error<'_> {
    pub fn input(&self) -> &[u8] {
        match self {
            Error::NomError { input, .. } => input.0,
            Error::ExternalError { input, .. } => input.0,
        }
    }
}

impl<'a> ParseError<&'a [u8]> for Error<'a> {
    fn from_error_kind(input: &'a [u8], kind: nom::error::ErrorKind) -> Self {
        NomError {
            input,
            kind,
            prev: PrevError::default(),
            context: Context::default(),
        }
        .build()
    }

    fn append(input: &'a [u8], kind: nom::error::ErrorKind, other: Self) -> Self {
        NomError {
            input,
            kind,
            prev: PrevError(Some(other.into())),
            context: Context::default(),
        }
        .build()
    }
}

impl ContextError<&[u8]> for Error<'_> {
    fn add_context(_: &[u8], ctx: &'static str, mut other: Self) -> Self {
        match &mut other {
            Self::NomError { context, .. } => context.add_context(ctx),
            Self::ExternalError { context, .. } => context.add_context(ctx),
        }
        other
    }
}

impl<'a> FromExternalError<&'a [u8], ExternalParseError> for Error<'a> {
    fn from_external_error(
        input: &'a [u8],
        kind: nom::error::ErrorKind,
        source: ExternalParseError,
    ) -> Self {
        ExternalError {
            input,
            kind,
            prev: PrevError::default(),
            context: Context::default(),
        }
        .into_error(source)
    }
}

#[derive(Debug, Default)]
pub struct Context(Vec<&'static str>);

impl Context {
    fn add_context(&mut self, ctx: &'static str) {
        self.0.push(ctx)
    }
}

impl Display for Context {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.0.len() > 0 {
            f.write_str("\nContext:")?;
            for context in &self.0 {
                f.write_fmt(format_args!("\n    {}", context))?;
            }
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct PrevError<'a>(Option<Box<Error<'a>>>);

impl Default for PrevError<'_> {
    fn default() -> Self {
        Self(None)
    }
}

impl Display for PrevError<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(err) = &self.0 {
            f.write_fmt(format_args!("\n{}", err))?;
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct Input<'a>(&'a [u8]);

impl<'a> From<&'a [u8]> for Input<'a> {
    fn from(i: &'a [u8]) -> Self {
        Self(i)
    }
}

impl Display for Input<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("[")?;

        let len = self.0.len();

        if len == 0 {
            f.write_str("]")?;
            return Ok(());
        } else {
            f.write_str(" ")?
        }

        for byte in &self.0[0..len.min(8)] {
            f.write_fmt(format_args!("{:02X} ", byte))?
        }

        if len > 8 {
            f.write_str("... ")?;
        }

        f.write_str("]")?;

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn formats_empty_slice() {
        assert_eq!(Input(&[]).to_string(), "[]");
    }

    #[test]
    fn formats_up_to_len8_slice() {
        assert_eq!(
            Input(&[0x11, 0xfd, 0x1, 0x7c]).to_string(),
            "[ 11 FD 01 7C ]"
        );
        assert_eq!(
            Input(&[0x11, 0xfd, 0x1, 0x7c, 0xaa, 0x7b, 0xd0, 0x11]).to_string(),
            "[ 11 FD 01 7C AA 7B D0 11 ]"
        );
    }
    #[test]
    fn formats_over_len8_slice() {
        assert_eq!(
            Input(&[
                0x11, 0xfd, 0x1, 0x7c, 0xaa, 0x7b, 0xd0, 0x11, 0x9e, 0xc, 0x0, 0xa0, 0xc9, 0x22,
                0xe6, 0xec,
            ])
            .to_string(),
            "[ 11 FD 01 7C AA 7B D0 11 ... ]"
        );
    }
}
