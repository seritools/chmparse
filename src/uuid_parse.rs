use nom::{
    bytes::complete::take, combinator::map_opt, combinator::verify, error::ParseError, IResult,
};
use uuid::Uuid;

pub fn parse_uuid_v1<'a, Error: ParseError<&'a [u8]>>(
    i: &'a [u8],
) -> IResult<&'a [u8], Uuid, Error> {
    map_opt(take(16usize), |bytes| Uuid::from_slice(bytes).ok())(i)
}

pub fn parse_exact_uuid_v1<'a, Error: ParseError<&'a [u8]>>(
    expected: [u8; 16],
) -> impl Fn(&'a [u8]) -> IResult<&'a [u8], Uuid, Error> {
    move |i| verify(parse_uuid_v1, |uuid| uuid.as_bytes() == &expected)(i)
}
