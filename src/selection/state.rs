//! Selection state for tracking drag selection across cells.

use chrono::{NaiveDate, NaiveTime};
use log::debug;

use super::point::SelectionPoint;
use super::range::SelectionRange;

/// State for tracking drag selection across day/time cells.
///
/// This is a transient UI state, not a dialog, so it lives directly
/// in CosmicCalendar rather than in ActiveDialog.
///
/// Supports both:
/// - Date-only selection (month view): Uses SelectionPoint with time = None
/// - Time-based selection (week/day views): Uses SelectionPoint with time = Some(t)
#[derive(Debug, Clone, Default)]
pub struct SelectionState {
    /// The point where the selection started (mouse press)
    start: Option<SelectionPoint>,
    /// The current end point of the selection (follows mouse)
    end: Option<SelectionPoint>,
    /// Whether a drag selection is currently active
    pub is_active: bool,
}

impl SelectionState {
    /// Create a new empty selection state
    pub fn new() -> Self {
        Self::default()
    }

    /// Start a new date-only selection (for month view)
    pub fn start(&mut self, date: NaiveDate) {
        self.start_at(SelectionPoint::date_only(date));
    }

    /// Start a new time-based selection (for week/day views)
    #[allow(dead_code)] // Reserved for week/day view selection
    pub fn start_with_time(&mut self, date: NaiveDate, time: NaiveTime) {
        self.start_at(SelectionPoint::with_time(date, time));
    }

    /// Start a new selection at the given point
    fn start_at(&mut self, point: SelectionPoint) {
        debug!("SelectionState: Starting selection at {:?}", point);
        self.start = Some(point);
        self.end = Some(point);
        self.is_active = true;
    }

    /// Update the selection end point with date only (for month view)
    pub fn update(&mut self, date: NaiveDate) {
        self.update_to(SelectionPoint::date_only(date));
    }

    /// Update the selection end point with date and time (for week/day views)
    #[allow(dead_code)] // Reserved for week/day view selection
    pub fn update_with_time(&mut self, date: NaiveDate, time: NaiveTime) {
        self.update_to(SelectionPoint::with_time(date, time));
    }

    /// Update the selection end point
    fn update_to(&mut self, point: SelectionPoint) {
        if self.is_active {
            debug!("SelectionState: Updating selection end to {:?}", point);
            self.end = Some(point);
        }
    }

    /// End the selection and return the selected range if valid
    pub fn end(&mut self) -> Option<SelectionRange> {
        if !self.is_active {
            return None;
        }

        let range = self.get_range();
        debug!("SelectionState: Ending selection with range {:?}", range);
        self.reset();
        range
    }

    /// Cancel the current selection
    pub fn cancel(&mut self) {
        debug!("SelectionState: Cancelling selection");
        self.reset();
    }

    /// Reset the selection state
    pub fn reset(&mut self) {
        self.start = None;
        self.end = None;
        self.is_active = false;
    }

    /// Get the current selection range (normalized so start <= end)
    pub fn get_range(&self) -> Option<SelectionRange> {
        match (self.start, self.end) {
            (Some(start), Some(end)) => Some(SelectionRange::new(start, end)),
            _ => None,
        }
    }

    /// Check if a date is within the current selection (ignoring time)
    pub fn contains(&self, date: NaiveDate) -> bool {
        self.get_range()
            .map(|r| r.contains_date(date))
            .unwrap_or(false)
    }

    /// Check if the current selection spans multiple days
    #[allow(dead_code)] // Part of selection API
    pub fn is_multi_day(&self) -> bool {
        self.get_range()
            .map(|r| r.is_multi_day())
            .unwrap_or(false)
    }

    /// Check if a date+hour cell is within the current time-based selection
    /// Used for highlighting hour cells in week/day views
    pub fn contains_time(&self, date: NaiveDate, hour: u32) -> bool {
        let Some(range) = self.get_range() else {
            return false;
        };

        // Create time points for the start and end of the hour
        let cell_start = NaiveTime::from_hms_opt(hour, 0, 0).unwrap();
        let cell_end = NaiveTime::from_hms_opt(hour, 59, 59).unwrap();

        // Get selection times (default to full day if not set)
        let sel_start_time = range.start.time.unwrap_or_else(|| NaiveTime::from_hms_opt(0, 0, 0).unwrap());
        let sel_end_time = range.end.time.unwrap_or_else(|| NaiveTime::from_hms_opt(23, 59, 59).unwrap());

        let sel_start_date = range.start.date;
        let sel_end_date = range.end.date;

        // Check if this cell overlaps with the selection
        // The cell is selected if:
        // - date is within the date range AND
        // - time overlaps with the selected time range
        if date < sel_start_date || date > sel_end_date {
            return false;
        }

        // Same day selection
        if sel_start_date == sel_end_date && date == sel_start_date {
            // Cell overlaps if: cell_end >= sel_start AND cell_start <= sel_end
            return cell_end >= sel_start_time && cell_start <= sel_end_time;
        }

        // Multi-day selection
        if date == sel_start_date {
            // First day: include cells from start time onwards
            return cell_end >= sel_start_time;
        } else if date == sel_end_date {
            // Last day: include cells up to end time
            return cell_start <= sel_end_time;
        } else {
            // Middle days: all cells are selected
            return true;
        }
    }

    /// Get the start date (for backwards compatibility)
    #[allow(dead_code)] // Part of selection API
    pub fn start_date(&self) -> Option<NaiveDate> {
        self.start.map(|p| p.date)
    }

    /// Get the end date (for backwards compatibility)
    #[allow(dead_code)] // Part of selection API
    pub fn end_date(&self) -> Option<NaiveDate> {
        self.end.map(|p| p.date)
    }
}
