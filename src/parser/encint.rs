use pahs::try_parse;
use snafu::Snafu;

use super::{Driver, Pos, Progress};

/// Parses a CHM `ENCINT` variable-length integer.
///
/// An ENCINT is a variable-length integer.
/// The high bit of each byte indicates "continued to the next byte".
/// Bytes are stored most significant to least significant.
/// So, for example, $EA $15 is (((0xEA&0x7F)<<7)|0x15) = 0x3515.
pub fn parse_encint_be<'a>(
    _: &mut Driver,
    mut pos: Pos<'a>,
) -> Progress<'a, u64, EncIntParseError> {
    const CONTINUE: u8 = 0b1000_0000;

    let initial_pos = pos;
    let mut val = 0u64;
    let mut n = 0;
    loop {
        let (p, &b) = try_parse!(pos
            .take1()
            .rewind_on_err(initial_pos)
            .snafu_leaf(|_| NotEnoughData));
        pos = p;
        n += 1;

        let continue_next_byte = b & CONTINUE != 0;
        let byte_data = b & !CONTINUE;

        if n == 10 {
            // 9 ENCINT bytes * 7 bits = 63 bits
            if continue_next_byte || (val >> (64 - 7)) != 0 {
                // so at the 10th byte we have to check if shifting `val` would shift out `1`s,
                // or the encoding says an 11th byte will follow
                return Progress::failure(initial_pos, EncodedIntegerTooLong.build());
            }
        }

        val <<= 7;
        val |= byte_data as u64;

        if !continue_next_byte {
            return Progress::success(pos, val);
        }
    }
}

#[derive(Debug, Snafu)]
pub enum EncIntParseError {
    #[snafu(display("Not enough data in the input"))]
    NotEnoughData,
    #[snafu(display("Encoded integer longer than u64::MAX"))]
    EncodedIntegerTooLong,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn it_fails_on_empty() {
        let input = &[];
        let pos = Pos {
            offset: 0,
            s: input,
        };
        let pd = &mut Driver::new();

        let (pos_out, err) = parse_encint_be(pd, pos).unwrap_err();
        assert_eq!(pos, pos_out);
        assert!(matches!(err, EncIntParseError::NotEnoughData));
    }

    #[test]
    fn it_fails_on_not_enough_data() {
        let inputs: &[&[u8]] = &[
            &[0xF0],
            &[0xFF, 0xFF, 0xFF],
            &[0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF],
        ];
        for &input in inputs.iter() {
            let pos = Pos {
                offset: 0,
                s: input,
            };
            let pd = &mut Driver::new();

            let (pos_out, err) = parse_encint_be(pd, pos).unwrap_err();
            assert_eq!(pos, pos_out);
            assert!(matches!(err, EncIntParseError::NotEnoughData));
        }
    }

    #[test]
    fn it_fails_on_too_long_for_u64() {
        let inputs: &[&[u8]] = &[
            &[
                // 11 bytes (definitely > 64bit of data)
                0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x00,
            ],
            &[
                // only 10 bytes, but >64 ones
                0b1000_0011,
                0b1111_1111,
                0b1111_1111,
                0b1111_1111,
                0b1111_1111,
                0b1111_1111,
                0b1111_1111,
                0b1111_1111,
                0b1111_1111,
                0b0111_1111,
            ],
        ];

        for &input in inputs.iter() {
            let pos = Pos {
                offset: 0,
                s: input,
            };
            let pd = &mut Driver::new();

            let (pos_out, err) = parse_encint_be(pd, pos).unwrap_err();
            assert_eq!(pos, pos_out);
            assert!(matches!(err, EncIntParseError::EncodedIntegerTooLong));
        }
    }

    #[test]
    fn it_works() {
        let in_outs: &[(&[u8], u64)] = &[
            (&[0], 0),
            (&[0b10], 2),
            (&[0b0101_1010], 0b0101_1010),
            (&[0b1111_1111, 0b0111_1111], 0b0011_1111_1111_1111),
            (
                &[
                    // exactly 64 ones
                    0b1000_0001,
                    0b1111_1111,
                    0b1111_1111,
                    0b1111_1111,
                    0b1111_1111,
                    0b1111_1111,
                    0b1111_1111,
                    0b1111_1111,
                    0b1111_1111,
                    0b0111_1111,
                ],
                u64::MAX,
            ),
        ];

        for &(input, output) in in_outs.iter() {
            let pos = Pos {
                offset: 0,
                s: input,
            };
            let pd = &mut Driver::new();

            let (Pos { offset, .. }, val) = parse_encint_be(pd, pos).unwrap();
            assert_eq!(offset, input.len());
            assert_eq!(val, output);
        }
    }
}
