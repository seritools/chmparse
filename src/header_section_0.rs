use nom::{
    bytes::complete::tag, error::context, error::ContextError, error::ParseError,
    number::complete::le_u32, number::complete::le_u64, IResult,
};

#[derive(Debug)]
pub struct HeaderSection0 {
    /// Not all files set this correctly
    pub file_size: u64,
}

impl HeaderSection0 {
    pub fn parse<'a, E: ParseError<&'a [u8]> + ContextError<&'a [u8]>>(
        i: &'a [u8],
    ) -> IResult<&'a [u8], Self, E> {
        context("header section 0", |i| {
            let (i, _) = tag(&[0xFE, 0x01, 0x00, 0x00])(i)?;
            let (i, _) = le_u32(i)?;
            let (i, file_size) = le_u64(i)?;
            let (i, _) = le_u32(i)?;
            let (i, _) = le_u32(i)?;

            Ok((i, Self { file_size }))
        })(i)
    }
}
