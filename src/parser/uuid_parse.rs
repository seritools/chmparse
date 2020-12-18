use nom::bytes::complete::take;
use nom::combinator::map_res;
use nom::combinator::verify;
use nom::error::context;
use snafu::ResultExt;
use uuid::Uuid;

use super::NomParseResult;

pub fn parse_uuid(i: &[u8]) -> NomParseResult<'_, Uuid> {
    context(
        "UUID",
        map_res(take(16usize), |bytes| {
            Uuid::from_slice(bytes).context(super::error::UuidFail)
        }),
    )(i)
}

pub fn parse_exact_uuid(expected: Uuid) -> impl Fn(&[u8]) -> NomParseResult<'_, Uuid> {
    move |i| {
        context(
            "Check matching UUID",
            verify(parse_uuid, |uuid| uuid.as_bytes() == expected.as_bytes()),
        )(i)
    }
}
