use std::collections::HashMap;

use pahs::try_parse;
use pahs_snafu::ProgressSnafuExt;
use smallvec::SmallVec;
use snafu::Snafu;

use crate::directory_listing::listing_chunk::ListingChunkEntry;
use crate::{ChmFileHead, Driver, ParseChmFileHeadError, Pos, Progress};

#[derive(Debug)]
pub enum ContentSection<'a> {
    Uncompressed(ContentSectionPosition, &'a [u8]),
    /// LZX compression
    MsCompressed(ContentSectionPosition, Option<Box<[u8]>>),
}

#[derive(Debug)]
pub struct ContentSectionPosition {
    offset: usize,
    len: usize,
}

#[derive(Debug)]
pub struct ChmFile<'a> {
    file: &'a [u8],
    // content sections except content section 0
    extra_content_sections: SmallVec<[ContentSection<'a>; 2]>,
    file_entries: HashMap<&'a str, FileEntry>,
}

impl<'a> ChmFile<'a> {
    pub fn parse(
        pd: &mut Driver,
        pos: Pos<'a>,
        file: &'a [u8],
    ) -> Progress<'a, Self, ParseChmFileError> {
        let (_, head) = try_parse!(ChmFileHead::parse(pd, pos, file).snafu(|_| ParseChmFileHead));

        let file_entry_count: usize = head
            .directory_listing
            .entries
            .iter()
            .map(|chunk| chunk.entries.len())
            .sum();

        let mut file_entries: HashMap<_, _> = HashMap::with_capacity(file_entry_count);

        file_entries.extend(head.directory_listing.entries.iter().flat_map(|chunk| {
            chunk
                .entries
                .iter()
                .cloned()
                .map(|e| (e.name, FileEntry::from(e)))
        }));

        // the section name list contains is inside the first section
        // and contains the names of all other sections
        let (_, section_name_list) = try_parse!(Progress::from_result(
            pos,
            file_entries
                .get("::DataSpace/NameList")
                .ok_or_else(|| MissingContentSectionNameList.build()),
        ));

        if section_name_list.content_section != 0 {
            return Progress::failure(pos, ContentSectionNameListNotInContentSection0.build());
        }

        Progress::success(
            pos,
            ChmFile {
                file,
                file_entries,
                extra_content_sections: SmallVec::default(),
            },
        )
    }

    pub fn load(file: &'a [u8]) -> Result<Self, ParseChmFileError> {
        let pd = &mut Driver::with_state(Default::default());
        let pos = Pos::new(file);

        Self::parse(pd, pos, file).finish().1
    }
}

#[derive(Debug, Snafu)]
pub enum ParseChmFileError {
    #[snafu(display("Error parsing the file header:\n{}", source))]
    ParseChmFileHead { source: ParseChmFileHeadError },

    #[snafu(display("Missing content section name list"))]
    MissingContentSectionNameList,
    #[snafu(display("Content section name list not in first context section"))]
    ContentSectionNameListNotInContentSection0,
}

#[derive(Debug, Clone, Copy)]
pub struct FileEntry {
    pub content_section: u64,
    pub content_section_offset: u64,
    pub content_length: u64,
}

impl From<ListingChunkEntry<'_>> for FileEntry {
    fn from(entry: ListingChunkEntry<'_>) -> Self {
        let ListingChunkEntry {
            content_section,
            content_section_offset,
            content_length,
            ..
        } = entry;
        Self {
            content_section,
            content_section_offset,
            content_length,
        }
    }
}
