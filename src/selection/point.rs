//! Selection point type representing a point in time.

use chrono::{NaiveDate, NaiveTime};

/// A point in time that may or may not include a specific time.
/// - `time = None` means "all day" or "date only" (month view)
/// - `time = Some(t)` means a specific time (week/day views)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SelectionPoint {
    pub date: NaiveDate,
    pub time: Option<NaiveTime>,
}

impl SelectionPoint {
    /// Create a date-only selection point (for month view)
    pub fn date_only(date: NaiveDate) -> Self {
        Self { date, time: None }
    }

    /// Create a date+time selection point (for week/day views)
    #[allow(dead_code)] // Reserved for week/day view time selection
    pub fn with_time(date: NaiveDate, time: NaiveTime) -> Self {
        Self { date, time: Some(time) }
    }

    /// Get the date component
    #[allow(dead_code)] // Part of selection API
    pub fn date(&self) -> NaiveDate {
        self.date
    }

    /// Get the time component (if any)
    #[allow(dead_code)] // Part of selection API
    pub fn time(&self) -> Option<NaiveTime> {
        self.time
    }

    /// Check if this is a date-only point
    #[allow(dead_code)] // Part of selection API
    pub fn is_date_only(&self) -> bool {
        self.time.is_none()
    }
}
