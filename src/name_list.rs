use std::convert::TryFrom;

use pahs::combinators::count;
use pahs::slice::num::u16_le;
use pahs::slice::{tag, TagError};
use pahs::{sequence, try_parse, Recoverable};
use pahs_snafu::ProgressSnafuExt;
use snafu::Snafu;

use crate::{Driver, Pos, Progress};

const SECTION_NAME_UNCOMPRESSED: &[u8; 24] = b"U\0n\0c\0o\0m\0p\0r\0e\0s\0s\0e\0d\0"; // "Uncompressed"
const SECTION_NAME_MSCOMPRESSED: &[u8; 24] = b"M\0S\0C\0o\0m\0p\0r\0e\0s\0s\0e\0d\0"; // "MSCompressed"
const MAX_NAME_LIST_ENTRIES: u16 = 2;

enum NameListEntry {
    Uncompressed,
    MsCompressed,
}

impl TryFrom<&'_ [u8]> for NameListEntry {
    type Error = ParseNameListError;

    fn try_from(value: &'_ [u8]) -> Result<Self, Self::Error> {
        if value == SECTION_NAME_UNCOMPRESSED {
            Ok(Self::Uncompressed)
        } else if value == SECTION_NAME_MSCOMPRESSED {
            Ok(Self::MsCompressed)
        } else {
            Err(UnknownSectionName.build())
        }
    }
}

impl NameListEntry {
    fn parse<'a>(pd: &mut Driver, pos: Pos<'a>) -> Progress<'a, Self, ParseNameListError> {
        sequence!(
            pd,
            pos,
            {
                let length_in_words_without_null =
                    |pd, pos| u16_le(pd, pos).snafu_leaf(|_| NotEnoughData);
                let name = |_, pos: Pos<'a>| {
                    pos.take(2 * usize::from(length_in_words_without_null))
                        .snafu_leaf(|_| NotEnoughData)
                };
                // utf16 null terminated
                let _ = |pd, pos| {
                    tag(b"\0\0")(pd, pos).map_err(|e| match e {
                        TagError::NotEnoughData => NotEnoughData.build(),
                        TagError::TagMismatch => NameListEntryNotNullTerminated.build(),
                    })
                };
                let entry = |_, pos| Progress::from_result(pos, NameListEntry::try_from(name));
            },
            entry
        )
    }
}

#[derive(Debug, Default)]
pub(crate) struct NameList {
    pub(crate) has_ms_compressed_section: bool,
}

impl NameList {
    pub fn parse<'a>(pd: &mut Driver, pos: Pos<'a>) -> Progress<'a, Self, ParseNameListError> {
        let (pos, file_len_words) = try_parse!(u16_le(pd, pos).snafu_leaf(|_| NotEnoughData));

        if file_len_words < 1 {
            return Progress::failure(pos, NotEnoughData.build());
        }

        let (end, file_content) = try_parse!(pos
            .take(usize::from((file_len_words - 1) * 2))
            .snafu_leaf(|_| NotEnoughData));

        // limit rest of the parsing to the content the file
        let pos = Pos {
            offset: pos.offset,
            s: file_content,
        };

        let (pos, num_entries) = try_parse!(u16_le(pd, pos).snafu_leaf(|_| NotEnoughData));

        if num_entries > MAX_NAME_LIST_ENTRIES {
            return Progress::failure(
                pos,
                TooManyNameListEntries {
                    actual: num_entries,
                }
                .build(),
            );
        }

        // todo: vec → smallvec
        let (_, entries) = try_parse!(count(num_entries as usize, NameListEntry::parse)(pd, pos));

        let mut name_list = NameList::default();
        let mut has_uncompressed = false;

        for entry in entries {
            match entry {
                NameListEntry::Uncompressed => has_uncompressed = true,
                NameListEntry::MsCompressed => name_list.has_ms_compressed_section = true,
            }
        }

        if !has_uncompressed {
            return Progress::failure(pos, MissingUncompressedSectionEntry.build());
        }

        Progress::success(end, name_list)
    }
}

#[derive(Debug, Snafu)]
pub enum ParseNameListError {
    NotEnoughData,
    #[snafu(display(
        "Too many name list entries (max: {}, actual: {})",
        MAX_NAME_LIST_ENTRIES,
        actual
    ))]
    TooManyNameListEntries {
        actual: u16,
    },
    NameListEntryNotNullTerminated,
    UnknownSectionName,
    MissingUncompressedSectionEntry,
}

impl Recoverable for ParseNameListError {
    fn recoverable(&self) -> bool {
        use ParseNameListError::*;
        match self {
            NotEnoughData => true,
            MissingUncompressedSectionEntry => true,
            TooManyNameListEntries { .. } => false,
            NameListEntryNotNullTerminated => false,
            UnknownSectionName => false,
        }
    }
}
