use pahs::slice::tag;
use snafu::Snafu;

use crate::parser::{Driver, Pos, Progress};

#[derive(Debug)]
pub struct IndexChunk {}

impl IndexChunk {
    fn tag<'a>(
        expected: &'static [u8],
    ) -> impl Fn(&mut Driver, Pos<'a>) -> Progress<'a, &'a [u8], IndexChunkParseError> {
        move |pd, p| {
            tag(expected)(pd, p).snafu_leaf(|pos| InvalidTag {
                offset: pos.offset,
                expected,
            })
        }
    }
}

#[derive(Debug, Snafu)]
pub enum IndexChunkParseError {
    #[snafu(display("Not enough data in the input"))]
    NotEnoughData,

    #[snafu(display("Invalid tag at {:#X}, expected: {:?}", offset, expected))]
    InvalidTag {
        offset: usize,
        expected: &'static [u8],
    },
}

impl From<()> for IndexChunkParseError {
    fn from(_: ()) -> Self {
        NotEnoughData.build()
    }
}
