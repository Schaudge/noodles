mod attributes;
mod bounds;
mod fields;
mod position;
mod strand;

use core::fmt;

use self::{attributes::Attributes, position::Position, strand::Strand};
pub(crate) use self::{bounds::Bounds, fields::Fields};

/// An immutable, lazily-evalulated GFF record.
#[derive(Clone, Default, Eq, PartialEq)]
pub struct Record(pub(crate) Fields);

impl Record {
    /// Returns the reference sequence name.
    pub fn reference_sequence_name(&self) -> &str {
        self.0.reference_sequence_name()
    }

    /// Returns the source.
    pub fn source(&self) -> &str {
        self.0.source()
    }

    /// Returns the feature type.
    pub fn ty(&self) -> &str {
        self.0.ty()
    }

    /// Returns the start position.
    pub fn start(&self) -> Position<'_> {
        self.0.start()
    }

    /// Returns the end position.
    pub fn end(&self) -> Position<'_> {
        self.0.end()
    }

    /// Returns the score.
    pub fn score(&self) -> &str {
        self.0.score()
    }

    /// Returns the strand.
    pub fn strand(&self) -> Strand<'_> {
        self.0.strand()
    }

    /// Returns the phase.
    pub fn phase(&self) -> &str {
        self.0.phase()
    }

    /// Returns the attributes.
    pub fn attributes(&self) -> Attributes<'_> {
        self.0.attributes()
    }
}

impl fmt::Debug for Record {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Record")
            .field("reference_sequence_name", &self.reference_sequence_name())
            .field("source", &self.source())
            .field("ty", &self.ty())
            .field("start", &self.start())
            .field("end", &self.end())
            .field("score", &self.score())
            .field("strand", &self.strand())
            .field("phase", &self.phase())
            .field("attributes", &self.attributes())
            .finish()
    }
}

impl From<Record> for String {
    fn from(record: Record) -> Self {
        record.0.buf
    }
}
