//! Genomic region interval.

use std::{
    error, fmt,
    ops::{RangeBounds, RangeFrom, RangeFull, RangeInclusive, RangeToInclusive},
    str::FromStr,
};

use crate::{position, Position};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Bound {
    Included(Position),
    Unbounded,
}

/// An interval.
///
/// An interval can be closed ([a, b]), left-closed and right-unbounded ([a, ∞)), left-unbounded
/// and right-closed ((-∞, b]), or unbounded ((-∞, ∞)).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Interval {
    start: Bound,
    end: Bound,
}

impl Interval {
    /// Creates a closed interval.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::ops::{Bound, RangeBounds};
    /// use noodles_core::{region::Interval, Position};
    ///
    /// let start = Position::try_from(8)?;
    /// let end = Position::try_from(13)?;
    ///
    /// let interval = Interval::new(start, end);
    /// assert_eq!(interval.start_bound().cloned(), Bound::Included(start));
    /// assert_eq!(interval.end_bound().cloned(), Bound::Included(end));
    /// # Ok::<_, noodles_core::position::TryFromIntError>(())
    /// ```
    pub fn new(start: Position, end: Position) -> Self {
        Self {
            start: Bound::Included(start),
            end: Bound::Included(end),
        }
    }

    /// Returns the start.
    ///
    /// # Examples
    ///
    /// ```
    /// use noodles_core::{region::Interval, Position};
    ///
    /// let start = Position::try_from(8)?;
    /// let end = Position::try_from(13)?;
    /// let a = Interval::from(start..=end);
    /// assert_eq!(a.start(), Some(start));
    ///
    /// let b = Interval::from(..=end);
    /// assert!(b.start().is_none());
    /// # Ok::<_, noodles_core::position::TryFromIntError>(())
    /// ```
    pub fn start(&self) -> Option<Position> {
        match self.start {
            Bound::Included(start) => Some(start),
            Bound::Unbounded => None,
        }
    }

    /// Returns the end.
    ///
    /// # Examples
    ///
    /// ```
    /// use noodles_core::{region::Interval, Position};
    ///
    /// let start = Position::try_from(8)?;
    /// let end = Position::try_from(13)?;
    /// let a = Interval::from(start..=end);
    /// assert_eq!(a.end(), Some(end));
    ///
    /// let b = Interval::from(start..);
    /// assert!(b.end().is_none());
    /// # Ok::<_, noodles_core::position::TryFromIntError>(())
    /// ```
    pub fn end(&self) -> Option<Position> {
        match self.end {
            Bound::Included(end) => Some(end),
            Bound::Unbounded => None,
        }
    }

    /// Returns whether the given interval intersects this interval.
    ///
    /// # Examples
    ///
    /// ```
    /// use noodles_core::{region::Interval, Position};
    ///
    /// let a = Interval::new(Position::try_from(5)?, Position::try_from(13)?);
    /// let b = Interval::new(Position::try_from(8)?, Position::try_from(21)?);
    /// assert!(a.intersects(b));
    ///
    /// let c = Interval::new(Position::try_from(2)?, Position::try_from(3)?);
    /// assert!(!b.intersects(c));
    /// # Ok::<_, noodles_core::position::TryFromIntError>(())
    /// ```
    pub fn intersects(&self, other: Self) -> bool {
        fn resolve(interval: Interval) -> (Position, Position) {
            (
                interval.start().unwrap_or(Position::MIN),
                interval.end().unwrap_or(Position::MAX),
            )
        }

        let (a_start, a_end) = resolve(*self);
        let (b_start, b_end) = resolve(other);

        a_start <= b_end && b_start <= a_end
    }
}

impl fmt::Display for Interval {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match (self.start, self.end) {
            (Bound::Unbounded, Bound::Unbounded) => Ok(()),
            (Bound::Unbounded, Bound::Included(e)) => write!(f, "{}-{}", Position::MIN, e),
            (Bound::Included(s), Bound::Unbounded) => s.fmt(f),
            (Bound::Included(s), Bound::Included(e)) => write!(f, "{}-{}", s, e),
        }
    }
}

impl RangeBounds<Position> for Interval {
    fn start_bound(&self) -> std::ops::Bound<&Position> {
        bound_to_std_ops_bound(&self.start)
    }

    fn end_bound(&self) -> std::ops::Bound<&Position> {
        bound_to_std_ops_bound(&self.end)
    }
}

fn bound_to_std_ops_bound(bound: &Bound) -> std::ops::Bound<&Position> {
    match bound {
        Bound::Included(ref value) => std::ops::Bound::Included(value),
        Bound::Unbounded => std::ops::Bound::Unbounded,
    }
}

/// An error returned when an interval fails to parse.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ParseError {
    /// The start position is invalid.
    InvalidStartPosition(position::ParseError),
    /// The end position is invalid.
    InvalidEndPosition(position::ParseError),
}

impl error::Error for ParseError {}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidStartPosition(e) => write!(f, "invalid start position: {}", e),
            Self::InvalidEndPosition(e) => write!(f, "invalid end position: {}", e),
        }
    }
}

impl FromStr for Interval {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            return Ok(Self::from(..));
        }

        let mut components = s.splitn(2, '-');

        let start = match components.next() {
            Some(t) => t
                .parse()
                .map(Bound::Included)
                .map_err(ParseError::InvalidStartPosition)?,
            None => Bound::Unbounded,
        };

        let end = match components.next() {
            Some(t) => t
                .parse()
                .map(Bound::Included)
                .map_err(ParseError::InvalidEndPosition)?,
            None => Bound::Unbounded,
        };

        Ok(Self { start, end })
    }
}

impl From<RangeFrom<Position>> for Interval {
    fn from(range: RangeFrom<Position>) -> Self {
        Self {
            start: Bound::Included(range.start),
            end: Bound::Unbounded,
        }
    }
}

impl From<RangeFull> for Interval {
    fn from(_: RangeFull) -> Self {
        Self {
            start: Bound::Unbounded,
            end: Bound::Unbounded,
        }
    }
}

impl From<RangeInclusive<Position>> for Interval {
    fn from(range: RangeInclusive<Position>) -> Self {
        let (start, end) = range.into_inner();

        Self {
            start: Bound::Included(start),
            end: Bound::Included(end),
        }
    }
}

impl From<RangeToInclusive<Position>> for Interval {
    fn from(range: RangeToInclusive<Position>) -> Self {
        Self {
            start: Bound::Unbounded,
            end: Bound::Included(range.end),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_intersects() -> Result<(), crate::position::TryFromIntError> {
        //   1 2 3 4 5 6 7 8 9 0
        // a         [-----]     [5, 8]
        //   1 2 3 4 5 6 7 8 9 0
        // b       [---]         [4, 6]
        // c       [-]           [4, 5]
        // d             [---]   [7, 9]
        // e               [-]   [8, 9]
        // f       [---------]   [4, 9]
        // g           [-]       [6, 7]
        // h [-----------------> [1, ∞)
        // i             [-----> [7, ∞)
        // j <---------------]   (-∞, 9]
        // k <---------]         (-∞, 6]
        //   1 2 3 4 5 6 7 8 9 0
        // l [-]                 [1, 2]
        // m                 [-] [9, 10]
        // n                 [-> [9, ∞)
        // o <-]                 (-∞, 2]

        let a = Interval::new(Position::try_from(5)?, Position::try_from(8)?);
        assert!(a.intersects(a));

        let b = (Position::try_from(4)?..=Position::try_from(6)?).into();
        assert!(a.intersects(b));
        assert!(b.intersects(a));
        let c = (Position::try_from(4)?..=Position::try_from(5)?).into();
        assert!(a.intersects(c));
        assert!(c.intersects(a));
        let d = (Position::try_from(7)?..=Position::try_from(9)?).into();
        assert!(a.intersects(d));
        assert!(d.intersects(a));
        let e = (Position::try_from(8)?..=Position::try_from(9)?).into();
        assert!(a.intersects(e));
        assert!(e.intersects(a));
        let f = (Position::try_from(4)?..=Position::try_from(9)?).into();
        assert!(a.intersects(f));
        assert!(f.intersects(a));
        let g = (Position::try_from(6)?..=Position::try_from(7)?).into();
        assert!(a.intersects(g));
        assert!(g.intersects(a));
        let h = (Position::try_from(1)?..).into();
        assert!(a.intersects(h));
        assert!(h.intersects(a));
        let i = (Position::try_from(7)?..).into();
        assert!(a.intersects(i));
        assert!(i.intersects(a));
        let j = (..=Position::try_from(9)?).into();
        assert!(a.intersects(j));
        assert!(j.intersects(a));
        let k = (..=Position::try_from(6)?).into();
        assert!(a.intersects(k));
        assert!(k.intersects(a));

        let l = (Position::try_from(1)?..=Position::try_from(2)?).into();
        assert!(!a.intersects(l));
        assert!(!l.intersects(a));
        let m = (Position::try_from(9)?..=Position::try_from(10)?).into();
        assert!(!a.intersects(m));
        assert!(!m.intersects(a));
        let n = (Position::try_from(9)?..).into();
        assert!(!a.intersects(n));
        assert!(!n.intersects(a));
        let o = (..=Position::try_from(2)?).into();
        assert!(!a.intersects(o));
        assert!(!o.intersects(a));

        Ok(())
    }

    #[test]
    fn test_fmt() -> Result<(), crate::position::TryFromIntError> {
        let start = Position::try_from(8)?;
        let end = Position::try_from(13)?;

        let interval = Interval {
            start: Bound::Unbounded,
            end: Bound::Unbounded,
        };
        assert_eq!(interval.to_string(), "");

        let interval = Interval {
            start: Bound::Unbounded,
            end: Bound::Included(end),
        };
        assert_eq!(interval.to_string(), "1-13");

        let interval = Interval {
            start: Bound::Included(start),
            end: Bound::Unbounded,
        };
        assert_eq!(interval.to_string(), "8");

        assert_eq!(Interval::new(start, end).to_string(), "8-13");

        Ok(())
    }

    #[test]
    fn test_parse() -> Result<(), crate::position::TryFromIntError> {
        let start = Position::try_from(8)?;
        let end = Position::try_from(13)?;

        assert_eq!(
            "".parse(),
            Ok(Interval {
                start: Bound::Unbounded,
                end: Bound::Unbounded
            })
        );

        assert_eq!(
            "8".parse(),
            Ok(Interval {
                start: Bound::Included(start),
                end: Bound::Unbounded
            })
        );

        assert_eq!(
            "8-13".parse(),
            Ok(Interval {
                start: Bound::Included(start),
                end: Bound::Included(end),
            })
        );

        assert!(matches!(
            "x".parse::<Interval>(),
            Err(ParseError::InvalidStartPosition(_))
        ));

        assert!(matches!(
            "1-x".parse::<Interval>(),
            Err(ParseError::InvalidEndPosition(_))
        ));

        Ok(())
    }
}
