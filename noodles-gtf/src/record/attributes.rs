//! GTF record attributes.

pub mod entry;

pub use self::entry::Entry;

use std::{
    error,
    fmt::{self, Write},
    ops::Deref,
    str::FromStr,
};

const DELIMITER: char = ' ';

/// GTF record attributes.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Attributes(Vec<Entry>);

impl Deref for Attributes {
    type Target = [Entry];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<Vec<Entry>> for Attributes {
    fn from(entries: Vec<Entry>) -> Self {
        Self(entries)
    }
}

impl fmt::Display for Attributes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, entry) in self.0.iter().enumerate() {
            write!(f, "{entry}")?;

            f.write_char(entry::DELIMITER)?;

            if i < self.0.len() - 1 {
                f.write_char(DELIMITER)?;
            }
        }

        Ok(())
    }
}

/// An error returned when raw GTF attributes fail to parse.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ParseError {
    /// The input is empty.
    Empty,
    /// The input is invalid.
    Invalid,
    /// The input has an invalid entry.
    InvalidEntry(entry::ParseError),
}

impl error::Error for ParseError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Self::InvalidEntry(e) => Some(e),
            _ => None,
        }
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => write!(f, "empty input"),
            Self::Invalid => write!(f, "invalid input"),
            Self::InvalidEntry(_) => write!(f, "invalid entry"),
        }
    }
}

impl FromStr for Attributes {
    type Err = ParseError;

    fn from_str(mut s: &str) -> Result<Self, Self::Err> {
        use self::entry::parse_entry;

        if s.is_empty() {
            return Err(ParseError::Empty);
        }

        let mut entries = Vec::new();

        while !s.is_empty() {
            let entry = parse_entry(&mut s).map_err(ParseError::InvalidEntry)?;
            entries.push(entry);
        }

        Ok(Self(entries))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fmt() {
        let attributes = Attributes::from(vec![Entry::new("gene_id", "g0")]);
        assert_eq!(attributes.to_string(), r#"gene_id "g0";"#);

        let attributes = Attributes::from(vec![
            Entry::new("gene_id", "g0"),
            Entry::new("transcript_id", "t0"),
        ]);
        assert_eq!(
            attributes.to_string(),
            r#"gene_id "g0"; transcript_id "t0";"#
        );
    }

    #[test]
    fn test_from_str() {
        assert_eq!(
            r#"gene_id "g0";"#.parse(),
            Ok(Attributes::from(vec![Entry::new("gene_id", "g0")]))
        );

        assert_eq!(
            r#"gene_id "g0""#.parse::<Attributes>(),
            Ok(Attributes::from(vec![Entry::new("gene_id", "g0")]))
        );

        assert_eq!(
            r#"gene_id "g0"; transcript_id "t0";"#.parse(),
            Ok(Attributes::from(vec![
                Entry::new("gene_id", "g0"),
                Entry::new("transcript_id", "t0")
            ]))
        );

        assert_eq!(
            r#"gene_id "g0";transcript_id "t0";"#.parse::<Attributes>(),
            Ok(Attributes::from(vec![
                Entry::new("gene_id", "g0"),
                Entry::new("transcript_id", "t0")
            ]))
        );

        assert_eq!(
            r#"gene_id "g0";  transcript_id "t0";"#.parse::<Attributes>(),
            Ok(Attributes::from(vec![
                Entry::new("gene_id", "g0"),
                Entry::new("transcript_id", "t0")
            ]))
        );

        assert_eq!("".parse::<Attributes>(), Err(ParseError::Empty));
        assert!(matches!(
            r#";"#.parse::<Attributes>(),
            Err(ParseError::InvalidEntry(_))
        ));
    }
}
