//! Creates a new SAM file.
//!
//! This writes a SAM header and three unmapped records to stdout.
//!
//! Verify the output by piping to `samtools view --no-PG --with-header`.

use noodles_sam::{
    self as sam,
    alignment::Record,
    header::{
        record::value::{map::Program, Map},
        ReferenceSequence,
    },
};
use tokio::io;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut writer = sam::AsyncWriter::new(io::stdout());

    let header = sam::Header::builder()
        .set_header(Default::default())
        .add_reference_sequence(ReferenceSequence::new("sq0".parse()?, 8)?)
        .add_reference_sequence(ReferenceSequence::new("sq1".parse()?, 13)?)
        .add_reference_sequence(ReferenceSequence::new("sq2".parse()?, 21)?)
        .add_program(Map::<Program>::new("noodles-sam"))
        .add_comment("an example SAM written by noodles-sam")
        .build();

    writer.write_header(&header).await?;

    for _ in 0..3 {
        let record = Record::default();
        writer.write_record(&header, &record).await?;
    }

    Ok(())
}
