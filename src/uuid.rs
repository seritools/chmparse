use pahs::{try_parse, Recoverable};
use pahs_snafu::ProgressSnafuExt;
use snafu::{ResultExt, Snafu};
use uuid::Uuid;

use super::{Driver, Pos, Progress};

#[derive(Debug, Snafu)]
#[snafu(visibility = "pub(crate)")]
pub enum ParseUuidError {
    NotEnoughData,
    UuidParseFailed { source: uuid::Error },
}

impl Recoverable for ParseUuidError {
    fn recoverable(&self) -> bool {
        true
    }
}

#[derive(Debug, Snafu)]
#[snafu(visibility = "pub(crate)")]
pub enum ParseExactUuidError {
    #[snafu(display("Uuid parse failed:{}\n", source))]
    ParseFailed { source: ParseUuidError },
    #[snafu(display(
        "Uuid successfully parsed, but is different from the expected:\n    Expected: {}\n    Parsed: {}",
        expected,
        parsed
    ))]
    WrongUuid { expected: Uuid, parsed: Uuid },
}

impl Recoverable for ParseExactUuidError {
    fn recoverable(&self) -> bool {
        match self {
            ParseExactUuidError::ParseFailed { source } => source.recoverable(),
            ParseExactUuidError::WrongUuid { .. } => false,
        }
    }
}

pub fn parse_uuid(p: Pos<'_>) -> Progress<'_, Uuid, ParseUuidError> {
    p.take(16)
        .map_err(|_| NotEnoughData.build())
        .and_then(p, |b| Uuid::from_slice(b).context(UuidParseFailed))
}

fn parse_exact_uuid_inner(
    expected: Uuid,
) -> impl Fn(Pos<'_>) -> Progress<'_, Uuid, ParseExactUuidError> {
    move |p| {
        let (np, uuid) = try_parse!(parse_uuid(p).snafu(|_| ParseFailed));

        if uuid == expected {
            Progress::success(np, uuid)
        } else {
            p.failure(
                WrongUuid {
                    expected,
                    parsed: uuid,
                }
                .build(),
            )
        }
    }
}

pub fn parse_exact_uuid<'a, C, F, E2>(
    expected: Uuid,
    context_fn: F,
) -> impl FnOnce(&mut Driver, Pos<'a>) -> Progress<'a, Uuid, E2>
where
    C: snafu::IntoError<E2, Source = ParseExactUuidError>,
    F: FnOnce(Pos<'_>) -> C,
    E2: std::error::Error + snafu::ErrorCompat,
{
    move |_, p| parse_exact_uuid_inner(expected)(p).snafu(context_fn)
}
