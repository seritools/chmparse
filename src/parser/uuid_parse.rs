use nom::bytes::complete::take;
use nom::combinator::map_res;
use nom::combinator::verify;
use nom::error::context;
use snafu::ResultExt;
use uuid::Uuid;

use super::ParseResult;

pub fn parse_uuid_v1(i: &[u8]) -> ParseResult<'_, Uuid> {
    context(
        "parse uuid v1",
        map_res(take(16usize), |bytes| {
            Uuid::from_slice(bytes).context(super::error::UuidFail)
        }),
    )(i)
}

pub fn parse_exact_uuid_v1(expected: [u8; 16]) -> impl Fn(&[u8]) -> ParseResult<'_, Uuid> {
    move |i| {
        context(
            "check matching uuid v1",
            verify(parse_uuid_v1, |uuid| *uuid.as_bytes() == expected),
        )(i)
    }
}
