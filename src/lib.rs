#![feature(or_patterns)]

// use nom::{
//     bytes::complete::tag,
//     bytes::complete::take,
//     combinator::map,
//     combinator::map_opt,
//     combinator::map_res,
//     error::{self, ParseError},
//     multi::count,
//     number::complete::le_u32,
//     number::complete::le_u64,
//     sequence::tuple,
//     IResult, Parser,
// };

pub mod chm_file;
pub mod directory_listing;
pub mod header;
pub mod header_section_0;
mod uuid_parse;

#[cfg(test)]
mod dbg_helper;

// #[cfg(test)]
// mod tests {
//     #[test]
//     fn it_works() {
//         let test_chm = include_bytes!("../test-files/7-zip.chm");

//         match super::parse_header(test_chm) {
//             Ok((_, header)) => {
//                 dbg!(&header);
//             }
//             Err(e) => {
//                 dbg!(e);
//             }
//         }
//     }
// }
