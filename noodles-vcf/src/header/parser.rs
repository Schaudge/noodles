//! VCF header parser.

mod builder;
mod file_format_option;
pub(crate) mod record;

use std::error;

use indexmap::IndexMap;

pub use self::{builder::Builder, file_format_option::FileFormatOption, record::parse_record};
use super::{
    file_format::{self, FileFormat},
    record::Record,
    AlternativeAlleles, Contigs, Filters, Formats, Header, Infos, OtherRecords, SampleNames,
};

#[derive(Debug, Default, Eq, PartialEq)]
enum State {
    #[default]
    Empty,
    Ready,
    Done,
}

/// A VCF header parser.
#[derive(Debug, Default, Eq, PartialEq)]
pub struct Parser {
    file_format_option: FileFormatOption,
    state: State,
    file_format: FileFormat,
    infos: Infos,
    filters: Filters,
    formats: Formats,
    alternative_alleles: AlternativeAlleles,
    contigs: Contigs,
    sample_names: SampleNames,
    other_records: OtherRecords,
}

impl Parser {
    /// Creates a VCF header parser builder.
    pub fn builder() -> Builder {
        Builder::default()
    }

    /// Parses a raw VCF header.
    pub fn parse(&self, s: &str) -> Result<Header, ParseError> {
        let mut parser = Self::default();

        for line in s.lines() {
            parser.parse_partial(line)?;
        }

        parser.finish()
    }

    /// Parses and adds a raw record to the header.
    pub fn parse_partial(&mut self, s: &str) -> Result<(), ParseError> {
        if self.state == State::Done {
            return Err(ParseError::ExpectedEof);
        }

        if self.state == State::Empty {
            let file_format = match parse_file_format(s) {
                Ok(f) => match self.file_format_option {
                    FileFormatOption::Auto => f,
                    FileFormatOption::FileFormat(g) => g,
                },
                Err(e) => return Err(e),
            };

            self.file_format = file_format;
            self.state = State::Ready;

            return Ok(());
        }

        if s.starts_with("#CHROM") {
            parse_header(s, &mut self.sample_names)?;
            self.state = State::Done;
            return Ok(());
        }

        let record = record::parse_record(s.as_bytes(), self.file_format)
            .map_err(ParseError::InvalidRecord)?;

        match record {
            Record::FileFormat(_) => return Err(ParseError::UnexpectedFileFormat),
            Record::Info(id, info) => {
                self.infos.insert(id, info);
            }
            Record::Filter(id, filter) => {
                self.filters.insert(id, filter);
            }
            Record::Format(id, format) => {
                self.formats.insert(id, format);
            }
            Record::AlternativeAllele(id, alternative_allele) => {
                self.alternative_alleles.insert(id, alternative_allele);
            }
            Record::Contig(id, contig) => {
                self.contigs.insert(id, contig);
            }
            Record::Other(key, value) => {
                insert_other_record(&mut self.other_records, key, value)?;
            }
        }

        Ok(())
    }

    /// Builds the VCF header.
    pub fn finish(self) -> Result<Header, ParseError> {
        match self.state {
            State::Empty => Err(ParseError::Empty),
            State::Ready => Err(ParseError::MissingHeader),
            State::Done => Ok(Header {
                file_format: self.file_format,
                infos: self.infos,
                filters: self.filters,
                formats: self.formats,
                alternative_alleles: self.alternative_alleles,
                contigs: self.contigs,
                sample_names: self.sample_names,
                other_records: self.other_records,
            }),
        }
    }
}

/// An error returned when a raw VCF header fails to parse.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ParseError {
    /// The input is empty.
    Empty,
    /// The file format (`fileformat`) is missing.
    MissingFileFormat,
    /// The file format (`fileformat`) appears other than the first line.
    UnexpectedFileFormat,
    /// The file format (`fileformat`) is invalid.
    InvalidFileFormat(file_format::ParseError),
    /// A record is invalid.
    InvalidRecord(record::ParseError),
    /// A record has an invalid value.
    InvalidRecordValue(super::record::value::collection::AddError),
    /// The header is missing.
    MissingHeader,
    /// The header is invalid.
    InvalidHeader(String, String),
    /// A sample name is duplicated.
    ///
    /// § 1.5 Header line syntax (2021-01-13): "Duplicate sample IDs are not allowed."
    DuplicateSampleName(String),
    /// More data unexpectedly appears after the header header (`#CHROM`...).
    ExpectedEof,
    /// The position of the entry in the string match does not match the absolute position defined
    /// by the `IDX` field of a record.
    StringMapPositionMismatch((usize, String), (usize, String)),
}

impl error::Error for ParseError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Self::InvalidFileFormat(e) => Some(e),
            Self::InvalidRecord(e) => Some(e),
            Self::InvalidRecordValue(e) => Some(e),
            _ => None,
        }
    }
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Empty => f.write_str("empty input"),
            Self::MissingFileFormat => f.write_str("missing fileformat"),
            Self::UnexpectedFileFormat => f.write_str("unexpected file format"),
            Self::InvalidFileFormat(_) => f.write_str("invalid file format"),
            Self::InvalidRecord(_) => f.write_str("invalid record"),
            Self::InvalidRecordValue(_) => f.write_str("invalid record value"),
            Self::MissingHeader => f.write_str("missing header"),
            Self::InvalidHeader(actual, expected) => {
                write!(f, "invalid header: expected {expected}, got {actual}")
            }
            Self::DuplicateSampleName(sample_name) => {
                write!(f, "duplicate sample name: {sample_name}")
            }
            Self::ExpectedEof => f.write_str("expected EOF"),
            Self::StringMapPositionMismatch(actual, expected) => write!(
                f,
                "string map position mismatch: expected {} (IDX={}), got {} (IDX={})",
                expected.1, expected.0, actual.1, actual.0,
            ),
        }
    }
}

fn parse_file_format(s: &str) -> Result<FileFormat, ParseError> {
    let record = record::parse_record(s.as_bytes(), FileFormat::default())
        .map_err(ParseError::InvalidRecord)?;

    match record {
        Record::FileFormat(file_format) => Ok(file_format),
        _ => Err(ParseError::MissingFileFormat),
    }
}

fn insert_other_record(
    other_records: &mut OtherRecords,
    key: super::record::key::Other,
    value: super::record::Value,
) -> Result<(), ParseError> {
    let collection = other_records.entry(key).or_insert_with(|| match value {
        super::record::Value::String(_) => {
            super::record::value::Collection::Unstructured(Vec::new())
        }
        super::record::Value::Map(..) => {
            super::record::value::Collection::Structured(IndexMap::new())
        }
    });

    collection
        .add(value)
        .map_err(ParseError::InvalidRecordValue)?;

    Ok(())
}

fn parse_header(line: &str, sample_names: &mut SampleNames) -> Result<(), ParseError> {
    static HEADERS: &[&str] = &[
        "#CHROM", "POS", "ID", "REF", "ALT", "QUAL", "FILTER", "INFO",
    ];
    static FORMAT_HEADER: &str = "FORMAT";

    let mut fields = line.split(crate::record::FIELD_DELIMITER);

    for &expected in HEADERS.iter() {
        if let Some(actual) = fields.next() {
            if actual != expected {
                return Err(ParseError::InvalidHeader(actual.into(), expected.into()));
            }
        } else {
            return Err(ParseError::InvalidHeader(String::from(""), expected.into()));
        }
    }

    if let Some(field) = fields.next() {
        if field != FORMAT_HEADER {
            return Err(ParseError::InvalidHeader(
                field.into(),
                FORMAT_HEADER.into(),
            ));
        }

        for sample_name in fields {
            if !sample_names.insert(sample_name.into()) {
                return Err(ParseError::DuplicateSampleName(sample_name.into()));
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_str() -> Result<(), Box<dyn std::error::Error>> {
        use crate::{
            header::record::{
                value::{
                    map::{AlternativeAllele, Contig, Filter, Format, Info, Other},
                    Map,
                },
                Value,
            },
            record::{genotypes, info},
        };

        let s = r#"##fileformat=VCFv4.3
##fileDate=20200506
##source=noodles-vcf
##contig=<ID=sq0,length=8>
##contig=<ID=sq1,length=13>
##contig=<ID=sq2,length=21>
##INFO=<ID=NS,Number=1,Type=Integer,Description="Number of samples with data">
##FILTER=<ID=q10,Description="Quality below 10">
##FORMAT=<ID=GT,Number=1,Type=String,Description="Genotype">
##ALT=<ID=DEL,Description="Deletion">
##META=<ID=Assay,Type=String,Number=.,Values=[WholeGenome, Exome]>
##SAMPLE=<ID=sample0,Assay=WholeGenome>
##PEDIGREE=<ID=cid,Father=fid,Mother=mid>
#CHROM	POS	ID	REF	ALT	QUAL	FILTER	INFO	FORMAT	sample0
"#;

        let actual = Parser::default().parse(s)?;

        let expected = Header::builder()
            .set_file_format(FileFormat::new(4, 3))
            .insert("fileDate".parse()?, Value::String(String::from("20200506")))?
            .insert(
                "source".parse()?,
                Value::String(String::from("noodles-vcf")),
            )?
            .add_contig(
                "sq0".parse()?,
                Map::<Contig>::builder().set_length(8).build()?,
            )
            .add_contig(
                "sq1".parse()?,
                Map::<Contig>::builder().set_length(13).build()?,
            )
            .add_contig(
                "sq2".parse()?,
                Map::<Contig>::builder().set_length(21).build()?,
            )
            .add_info(
                info::field::key::SAMPLES_WITH_DATA_COUNT,
                Map::<Info>::from(&info::field::key::SAMPLES_WITH_DATA_COUNT),
            )
            .add_filter("q10", Map::<Filter>::new("Quality below 10"))
            .add_format(
                genotypes::keys::key::GENOTYPE,
                Map::<Format>::from(&genotypes::keys::key::GENOTYPE),
            )
            .add_alternative_allele("DEL".parse()?, Map::<AlternativeAllele>::new("Deletion"))
            .insert(
                "META".parse()?,
                Value::Map(
                    String::from("Assay"),
                    Map::<Other>::builder()
                        .insert("Type".parse()?, "String")
                        .insert("Number".parse()?, ".")
                        .insert("Values".parse()?, "[WholeGenome, Exome]")
                        .build()?,
                ),
            )?
            .insert(
                "SAMPLE".parse()?,
                Value::Map(
                    String::from("sample0"),
                    Map::<Other>::builder()
                        .insert("Assay".parse()?, "WholeGenome")
                        .build()?,
                ),
            )?
            .insert(
                "PEDIGREE".parse()?,
                Value::Map(
                    String::from("cid"),
                    Map::<Other>::builder()
                        .insert("Father".parse()?, "fid")
                        .insert("Mother".parse()?, "mid")
                        .build()?,
                ),
            )?
            .add_sample_name("sample0")
            .build();

        assert_eq!(actual, expected);

        Ok(())
    }

    #[test]
    fn test_from_str_without_file_format() {
        let s = r#"##ALT=<ID=DEL,Description="Deletion">
"#;

        assert_eq!(
            Parser::default().parse(s),
            Err(ParseError::MissingFileFormat)
        );
    }

    #[test]
    fn test_from_str_with_data_after_header() {
        let s = r#"##fileformat=VCFv4.3
#CHROM	POS	ID	REF	ALT	QUAL	FILTER	INFO
##contig=<ID=sq0,length=8>
"#;

        assert_eq!(Parser::default().parse(s), Err(ParseError::ExpectedEof));
    }

    #[test]
    fn test_from_str_with_multiple_fileformats() {
        let s = "\
##fileformat=VCFv4.3
##fileformat=VCFv4.3
";

        assert_eq!(
            Parser::default().parse(s),
            Err(ParseError::UnexpectedFileFormat)
        );
    }

    #[test]
    fn test_from_str_with_missing_headers() {
        let s = "##fileformat=VCFv4.3
";
        assert_eq!(Parser::default().parse(s), Err(ParseError::MissingHeader));
    }

    #[test]
    fn test_from_str_with_invalid_headers() {
        let s = "##fileformat=VCFv4.3
#CHROM	POS	ID	REF	ALT	QUALITY	FILTER	INFO
";

        assert_eq!(
            Parser::default().parse(s),
            Err(ParseError::InvalidHeader(
                String::from("QUALITY"),
                String::from("QUAL")
            ))
        );

        let s = "##fileformat=VCFv4.3
#CHROM	POS	ID
";

        assert_eq!(
            Parser::default().parse(s),
            Err(ParseError::InvalidHeader(
                String::from(""),
                String::from("REF")
            ))
        );

        let s = "##fileformat=VCFv4.3
#CHROM	POS	ID	REF	ALT	QUAL	FILTER	INFO	sample0
";

        assert_eq!(
            Parser::default().parse(s),
            Err(ParseError::InvalidHeader(
                String::from("sample0"),
                String::from("FORMAT")
            ))
        );
    }

    #[test]
    fn test_from_str_with_duplicate_sample_names() {
        let s = "##fileformat=VCFv4.3
#CHROM	POS	ID	REF	ALT	QUAL	FILTER	INFO	FORMAT	sample0	sample0
";

        assert_eq!(
            Parser::default().parse(s),
            Err(ParseError::DuplicateSampleName(String::from("sample0")))
        );
    }
}
