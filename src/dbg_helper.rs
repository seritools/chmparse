use nom::HexDisplay;
use nom::{
    error::{VerboseError, VerboseErrorKind},
    Offset,
};
use nom::{
    Err::{Error, Failure},
    IResult,
};

pub fn print_err<T>(orig_input: &[u8], result: IResult<&[u8], T, VerboseError<&[u8]>>) {
    if let Err(Error(e) | Failure(e)) = result {
        for (pos, error_kind) in &e.errors {
            match error_kind {
                VerboseErrorKind::Context(c) => {
                    println!("context at 0x{:X}: {}", orig_input.offset(pos), c)
                }
                VerboseErrorKind::Char(c) => {
                    println!("char expected at 0x{:X}: {}", orig_input.offset(pos), c)
                }
                VerboseErrorKind::Nom(n) => {
                    println!("nom error at 0x{:X}: {:?}", orig_input.offset(pos), n)
                }
            }
        }
        if let Some((pos, _)) = e.errors.first() {
            println!(
                "data at error:\n{}",
                pos[0..std::cmp::min(128, pos.len())].to_hex(16)
            );
        }
        panic!();
    }
}
