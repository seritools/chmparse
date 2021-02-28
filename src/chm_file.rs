use std::collections::HashMap;
use std::convert::TryFrom;

use pahs::try_parse;
use pahs_snafu::ProgressSnafuExt;
use snafu::{ResultExt, Snafu};

use crate::directory_listing::listing_chunk::ListingChunkEntry;
use crate::name_list::{NameList, ParseNameListError};
use crate::{ChmFileHead, Driver, ParseChmFileHeadError, Pos, Progress};

#[derive(Debug)]
pub enum CompressedContentSection<'a> {
    Raw(&'a [u8]),
    Decompressed(Box<[u8]>),
}

#[derive(Debug)]
pub struct ChmFile<'a> {
    file: &'a [u8],
    head: ChmFileHead<'a>,
    uncompressed_content_section: &'a [u8],
    compressed_content_section: Option<CompressedContentSection<'a>>,
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

        let uncompressed_content_section = &file[head.offset_content_section_0..];

        Progress::success(
            pos,
            ChmFile {
                file,
                head,
                file_entries,
                uncompressed_content_section,
                compressed_content_section: None,
            },
        )
    }

    pub fn load(file: &'a [u8]) -> Result<Self, ParseChmFileError> {
        let pd = &mut Driver::with_state(Default::default());
        let pos = Pos::new(file);

        let mut chm_file = Self::parse(pd, pos, file).finish().1?;
        chm_file.populate_extra_content_sections(pd)?;

        Ok(chm_file)
    }

    fn populate_extra_content_sections(
        &mut self,
        pd: &mut Driver,
    ) -> Result<(), ParseChmFileError> {
        // the section name list contains is inside the first section
        // and contains the names of all other sections
        let name_list_pos = self
            .get_pos_for_file("::DataSpace/NameList")
            .map_err(|e| match e {
                GetPosForFileError::FileNotFound => MissingContentSectionNameList.build(),
                GetPosForFileError::FileOutOfBounds => NameListOutOfBounds.build(),
                _ => unreachable!(),
            })?;

        let (_, name_list) = NameList::parse(pd, name_list_pos)
            .snafu(|_| ParseNameList)
            .finish();
        let name_list = name_list?;

        if name_list.has_ms_compressed_section {
            let pos = self
                .get_pos_for_file("::DataSpace/Storage/MSCompressed/Content")
                .context(PopulateContentSections)?;
            self.compressed_content_section = Some(CompressedContentSection::Raw(pos.s));
        }

        Ok(())
    }

    fn get_pos_for_file(&self, file_name: &str) -> Result<Pos<'a>, GetPosForFileError> {
        let name_list_entry = self
            .file_entries
            .get(file_name)
            .ok_or_else(|| FileNotFound.build())?;

        if name_list_entry.content_section != 0 {
            // TODO: implement for compressed section
            todo!();
        }

        usize::try_from(name_list_entry.content_section_offset)
            .ok()
            .and_then(|offset| self.head.offset_content_section_0.checked_add(offset))
            .and_then(|start| Some((start, usize::try_from(name_list_entry.content_length).ok()?)))
            .and_then(|(start, len)| Some((start, start.checked_add(len)?)))
            .and_then(|(start, end)| Some((start, self.file.get(start..end)?)))
            .map(|(start, data)| Pos {
                offset: start,
                s: data,
            })
            .ok_or_else(|| FileOutOfBounds.build())
    }
}

#[derive(Debug, Snafu)]
pub enum ParseChmFileError {
    #[snafu(display("Error parsing the file header:\n{}", source))]
    ParseChmFileHead {
        source: ParseChmFileHeadError,
    },

    #[snafu(display("Missing content section name list"))]
    MissingContentSectionNameList,
    #[snafu(display("Content section name list not in first context section"))]
    ContentSectionNameListNotInContentSection0,

    NameListOutOfBounds,

    ParseNameList {
        source: ParseNameListError,
    },

    PopulateContentSections {
        source: GetPosForFileError,
    },
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

#[derive(Debug, Snafu)]
pub enum GetPosForFileError {
    FileNotFound,
    FileOutOfBounds,
    FileInInvalidContentSection,
}
