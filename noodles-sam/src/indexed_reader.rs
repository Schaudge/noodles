//! Indexed SAM reader.

mod builder;

pub use self::builder::Builder;

use std::io::{self, Read, Seek};

use noodles_bgzf as bgzf;
use noodles_core::Region;
use noodles_csi::BinningIndex;

use super::{alignment::RecordBuf, reader::RecordBufs, Header, Reader, Record};

/// An indexed SAM reader.
pub struct IndexedReader<R> {
    inner: Reader<bgzf::Reader<R>>,
    index: Box<dyn BinningIndex>,
}

impl<R> IndexedReader<R>
where
    R: Read,
{
    /// Creates an indexed SAM reader.
    pub fn new<I>(inner: R, index: I) -> Self
    where
        I: BinningIndex + 'static,
    {
        Self {
            inner: Reader::new(bgzf::Reader::new(inner)),
            index: Box::new(index),
        }
    }

    /// Returns a reference to the underlying reader.
    pub fn get_ref(&self) -> &bgzf::Reader<R> {
        self.inner.get_ref()
    }

    /// Returns a mutable reference to the underlying reader.
    pub fn get_mut(&mut self) -> &mut bgzf::Reader<R> {
        self.inner.get_mut()
    }

    /// Returns the underlying reader.
    pub fn into_inner(self) -> bgzf::Reader<R> {
        self.inner.into_inner()
    }

    /// Reads the SAM header.
    pub fn read_header(&mut self) -> io::Result<Header> {
        self.inner.read_header()
    }

    /// Reads a single SAM record.
    pub fn read_record_buf(
        &mut self,
        header: &Header,
        record: &mut RecordBuf,
    ) -> io::Result<usize> {
        self.inner.read_record_buf(header, record)
    }

    /// Returns an iterator over records starting from the current stream position.
    pub fn record_bufs<'a>(&'a mut self, header: &'a Header) -> RecordBufs<'a, bgzf::Reader<R>> {
        self.inner.record_bufs(header)
    }

    /// Reads a single record without eagerly decoding its fields.
    pub fn read_record(&mut self, record: &mut Record) -> io::Result<usize> {
        self.inner.read_record(record)
    }

    /// Returns the associated index.
    pub fn index(&self) -> &dyn BinningIndex {
        &self.index
    }
}

impl<R> IndexedReader<R>
where
    R: Read + Seek,
{
    /// Returns an iterator over records that intersect the given region.
    ///
    /// To query for unmapped records, use [`Self::query_unmapped`].
    pub fn query<'a>(
        &'a mut self,
        header: &'a Header,
        region: &Region,
    ) -> io::Result<impl Iterator<Item = io::Result<RecordBuf>> + 'a> {
        self.inner.query(header, &self.index, region)
    }

    /// Returns an iterator of unmapped records after querying for the unmapped region.
    pub fn query_unmapped<'a>(
        &'a mut self,
        header: &'a Header,
    ) -> io::Result<impl Iterator<Item = io::Result<RecordBuf>> + 'a> {
        self.inner.query_unmapped(header, &self.index)
    }
}
