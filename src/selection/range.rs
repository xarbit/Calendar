//! Selection range representing a normalized date/time range.

use chrono::NaiveDate;
use chrono::NaiveTime;

use super::point::SelectionPoint;

/// A normalized selection range with start <= end.
/// Supports both date-only and date+time ranges.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SelectionRange {
    /// The start point of the selection
    pub start: SelectionPoint,
    /// The end point of the selection
    pub end: SelectionPoint,
}

impl SelectionRange {
    /// Create a new range, normalizing so start <= end
    pub fn new(point1: SelectionPoint, point2: SelectionPoint) -> Self {
        // Compare by date first, then by time
        let p1_key = (point1.date, point1.time);
        let p2_key = (point2.date, point2.time);

        if p1_key <= p2_key {
            Self { start: point1, end: point2 }
        } else {
            Self { start: point2, end: point1 }
        }
    }

    /// Create a date-only range (for month view compatibility)
    #[allow(dead_code)] // Part of selection API
    pub fn from_dates(date1: NaiveDate, date2: NaiveDate) -> Self {
        Self::new(SelectionPoint::date_only(date1), SelectionPoint::date_only(date2))
    }

    /// Get the start date
    #[allow(dead_code)] // Part of selection API
    pub fn start_date(&self) -> NaiveDate {
        self.start.date
    }

    /// Get the end date
    #[allow(dead_code)] // Part of selection API
    pub fn end_date(&self) -> NaiveDate {
        self.end.date
    }

    /// Get the start time (if any)
    pub fn start_time(&self) -> Option<NaiveTime> {
        self.start.time
    }

    /// Get the end time (if any)
    pub fn end_time(&self) -> Option<NaiveTime> {
        self.end.time
    }

    /// Check if this is a date-only range (no times specified)
    #[allow(dead_code)] // Part of selection API
    pub fn is_date_only(&self) -> bool {
        self.start.is_date_only() && self.end.is_date_only()
    }

    /// Check if a date is within this range (inclusive, ignoring time)
    pub fn contains_date(&self, date: NaiveDate) -> bool {
        date >= self.start.date && date <= self.end.date
    }

    /// Check if this range spans multiple days
    #[allow(dead_code)] // Part of selection API
    pub fn is_multi_day(&self) -> bool {
        self.start.date != self.end.date
    }

    /// Get the number of days in this range
    #[allow(dead_code)] // Part of selection API
    pub fn day_count(&self) -> i64 {
        (self.end.date - self.start.date).num_days() + 1
    }

    /// Iterate over all dates in this range
    #[allow(dead_code)] // Part of selection API
    pub fn dates(&self) -> impl Iterator<Item = NaiveDate> {
        let start = self.start.date;
        let end = self.end.date;
        (0..=((end - start).num_days())).map(move |i| start + chrono::Duration::days(i))
    }
}
