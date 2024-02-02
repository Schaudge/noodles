mod bounds;

use std::io;

use self::bounds::Bounds;
use super::{Genotypes, Ids, ReferenceBases};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Fields {
    site_buf: Vec<u8>,
    samples_buf: Vec<u8>,
    bounds: Bounds,
}

impl Fields {
    pub(crate) fn site_buf_mut(&mut self) -> &mut Vec<u8> {
        &mut self.site_buf
    }

    pub(crate) fn samples_buf_mut(&mut self) -> &mut Vec<u8> {
        &mut self.samples_buf
    }

    pub(super) fn reference_sequence_id(&self) -> i32 {
        let src = &self.site_buf[bounds::REFERENCE_SEQUENCE_ID_RANGE];
        // SAFETY: `src` is 4 bytes.
        i32::from_le_bytes(src.try_into().unwrap())
    }

    // N.B. this is 0-based.
    pub(super) fn position(&self) -> i32 {
        let src = &self.site_buf[bounds::POSITION_RANGE];
        // SAFETY: `src` is 4 bytes.
        i32::from_le_bytes(src.try_into().unwrap())
    }

    pub(super) fn span(&self) -> i32 {
        let src = &self.site_buf[bounds::SPAN_RANGE];
        // SAFETY: `src` is 4 bytes.
        i32::from_le_bytes(src.try_into().unwrap())
    }

    pub(super) fn quality_score(&self) -> io::Result<Option<f32>> {
        use crate::record::codec::value::Float;

        let src = &self.site_buf[bounds::QUALITY_SCORE_RANGE];
        // SAFETY: `src` is 4 bytes.
        let n = f32::from_le_bytes(src.try_into().unwrap());

        match Float::from(n) {
            Float::Value(n) => Ok(Some(n)),
            Float::Missing => Ok(None),
            _ => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "invalid quality score",
            )),
        }
    }

    fn sample_count(&self) -> io::Result<usize> {
        let src = &self.site_buf[bounds::SAMPLE_COUNT_RANGE];
        let n = u32::from_le_bytes([src[0], src[1], src[2], 0x00]);
        usize::try_from(n).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }

    fn format_key_count(&self) -> usize {
        let n = self.site_buf[bounds::FORMAT_KEY_COUNT_INDEX];
        usize::from(n)
    }

    pub(super) fn ids(&self) -> Ids<'_> {
        let src = &self.site_buf[self.bounds.ids_range()];
        Ids::new(src)
    }

    pub(super) fn reference_bases(&self) -> ReferenceBases<'_> {
        let src = &self.site_buf[self.bounds.reference_bases_range()];
        ReferenceBases::new(src)
    }

    pub(super) fn genotypes(&self) -> io::Result<Genotypes<'_>> {
        self.sample_count().map(|sample_count| {
            Genotypes::new(&self.samples_buf, sample_count, self.format_key_count())
        })
    }

    pub(crate) fn index(&mut self) -> io::Result<()> {
        index(&self.site_buf, &mut self.bounds)
    }
}

fn index(buf: &[u8], bounds: &mut Bounds) -> io::Result<()> {
    use super::value::{read_type, Type};

    const IDS_START_INDEX: usize = bounds::FORMAT_KEY_COUNT_INDEX + 1;

    // [start, end)
    fn consume_string(buf: &mut &[u8], offset: usize) -> io::Result<(usize, usize)> {
        let prev_buf_len = buf.len();

        let Some(Type::String(len)) = read_type(buf)? else {
            return Err(io::Error::from(io::ErrorKind::InvalidData));
        };

        let start = offset + (prev_buf_len - buf.len());
        let end = start + len;

        *buf = &buf[len..];

        Ok((start, end))
    }

    let mut i = IDS_START_INDEX;

    let Some(mut buf) = buf.get(i..) else {
        return Err(io::Error::from(io::ErrorKind::UnexpectedEof));
    };

    let (start, end) = consume_string(&mut buf, i)?;
    bounds.ids_range = start..end;
    i = end;

    let (start, end) = consume_string(&mut buf, i)?;
    bounds.reference_bases_range = start..end;

    Ok(())
}

impl Default for Fields {
    fn default() -> Self {
        let site_buf = vec![
            0x00, 0x00, 0x00, 0x00, // chrom = 0
            0x00, 0x00, 0x00, 0x00, // pos = 0 (0-based)
            0x01, 0x00, 0x00, 0x00, // rlen = 1
            0x01, 0x00, 0x80, 0x7f, // qual = None
            0x00, 0x00, // n_info = 0
            0x01, 0x00, // n_allele = 1
            0x00, 0x00, 0x00, // n_sample = 0
            0x00, // n_fmt = 0
            0x07, // ids = []
            0x17, b'N', // ref = N
            0x00, // filters = []
        ];

        let bounds = Bounds {
            ids_range: 24..24,
            reference_bases_range: 26..27,
        };

        Self {
            site_buf,
            samples_buf: Vec::new(),
            bounds,
        }
    }
}
