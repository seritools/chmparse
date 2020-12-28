use pahs::{try_parse, ParseDriver, Recoverable};
use snafu::{ResultExt, Snafu};
use uuid::Uuid;

use super::{Pos, Progress};

#[derive(Debug, Snafu)]
#[snafu(visibility = "pub(crate)")]
pub enum UuidError {
    NotEnoughData,
    UuidParseFailed { source: uuid::Error },
}

impl Recoverable for UuidError {
    fn recoverable(&self) -> bool {
        true
    }
}

#[derive(Debug, Snafu)]
#[snafu(visibility = "pub(crate)")]
pub enum ExactUuidError {
    ParseFailed { source: UuidError },
    WrongUuid { expected: Uuid, parsed: Uuid },
}

impl Recoverable for ExactUuidError {
    fn recoverable(&self) -> bool {
        match self {
            ExactUuidError::ParseFailed { source } => source.recoverable(),
            ExactUuidError::WrongUuid { .. } => false,
        }
    }
}

pub fn parse_uuid(p: Pos<'_>) -> Progress<'_, Uuid, UuidError> {
    p.take(16)
        .map_err(|_| NotEnoughData.build())
        .and_then(p, |b| Uuid::from_slice(b).context(UuidParseFailed))
}

fn parse_exact_uuid_inner(
    expected: Uuid,
) -> impl Fn(Pos<'_>) -> Progress<'_, Uuid, ExactUuidError> {
    move |p| {
        let (np, uuid) = try_parse!(parse_uuid(p).snafu(|_| ParseFailed));

        if uuid == expected {
            Progress::success(np, uuid)
        } else {
            p.fail().map_err(|_| {
                WrongUuid {
                    expected,
                    parsed: uuid,
                }
                .build()
            })
        }
    }
}

pub fn parse_exact_uuid<'a, C, F, E2>(
    expected: Uuid,
    context_fn: F,
) -> impl FnOnce(&mut ParseDriver, Pos<'a>) -> Progress<'a, Uuid, E2>
where
    C: snafu::IntoError<E2, Source = ExactUuidError>,
    F: FnOnce(Pos<'_>) -> C,
    E2: std::error::Error + snafu::ErrorCompat,
{
    move |_, p| parse_exact_uuid_inner(expected)(p).snafu(context_fn)
}
