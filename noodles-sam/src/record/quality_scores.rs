//! Raw SAM record quality scores.

/// Raw SAM record quality scores.
#[derive(Debug, Eq, PartialEq)]
pub struct QualityScores<'a>(&'a [u8]);

impl<'a> QualityScores<'a> {
    /// Creates raw SAM record quality scores.
    ///
    /// # Examples
    ///
    /// ```
    /// use noodles_sam::record::QualityScores;
    /// let quality_scores = QualityScores::new(b"NDLS");
    /// ```
    pub fn new(buf: &'a [u8]) -> Self {
        Self(buf)
    }

    /// Returns whether there are any scores.
    ///
    /// # Examples
    ///
    /// ```
    /// use noodles_sam::record::QualityScores;
    ///
    /// let quality_scores = QualityScores::new(b"");
    /// assert!(quality_scores.is_empty());
    ///
    /// let quality_scores = QualityScores::new(b"NDLS");
    /// assert!(!quality_scores.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns the number of scores.
    ///
    /// # Examples
    ///
    /// ```
    /// use noodles_sam::record::QualityScores;
    /// let quality_scores = QualityScores::new(b"NDLS");
    /// assert_eq!(quality_scores.len(), 4);
    /// ```
    pub fn len(&self) -> usize {
        self.0.len()
    }
}

impl<'a> crate::alignment::record::QualityScores for QualityScores<'a> {
    fn is_empty(&self) -> bool {
        self.is_empty()
    }

    fn len(&self) -> usize {
        self.len()
    }

    fn iter(&self) -> Box<dyn Iterator<Item = u8> + '_> {
        const OFFSET: u8 = b'!';
        Box::new(self.as_ref().iter().map(|&b| b - OFFSET))
    }
}

impl<'a> AsRef<[u8]> for QualityScores<'a> {
    fn as_ref(&self) -> &[u8] {
        self.0
    }
}
