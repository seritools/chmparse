use pahs::try_parse;
use snafu::Snafu;

use directory_header::{DirectoryHeader, DirectoryHeaderParseError};

use super::{Driver, Pos, Progress};

mod directory_header;

#[derive(Debug)]
pub struct DirectoryListing {
    pub header: DirectoryHeader,
}

impl DirectoryListing {
    pub fn parse<'a>(
        pd: &mut Driver,
        pos: Pos<'a>,
    ) -> Progress<'a, DirectoryListing, DirectoryListingParseError> {
        let (pos, header) =
            try_parse!(DirectoryHeader::parse(pd, pos).snafu(|_| DirectoryHeaderParse));

        pos.success(DirectoryListing { header })
    }
}

#[derive(Debug, Snafu)]
pub enum DirectoryListingParseError {
    #[snafu(display("Failed to parse the directory header:\n{}", source))]
    DirectoryHeaderParse { source: DirectoryHeaderParseError },
}
