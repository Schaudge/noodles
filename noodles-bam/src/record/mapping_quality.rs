use std::io;

use noodles_sam as sam;

/// A raw BAM record mapping quality.
#[derive(Debug, Eq, PartialEq)]
pub struct MappingQuality(u8);

impl MappingQuality {
    pub(super) fn new(n: u8) -> Self {
        Self(n)
    }
}

impl sam::alignment::record::MappingQuality for MappingQuality {
    fn try_to_u8(&self) -> io::Result<u8> {
        Ok(self.0)
    }
}

impl From<MappingQuality> for u8 {
    fn from(mapping_quality: MappingQuality) -> Self {
        mapping_quality.0
    }
}