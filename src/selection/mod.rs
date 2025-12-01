//! Selection state management for multi-day/time event creation and event dragging.
//!
//! This module provides state tracking for:
//! - Drag selection across day/time cells for creating new events
//! - Event drag-and-drop for moving existing events
//!
//! The architecture supports both date-only (month view) and date+time (week/day views) operations.
//!
//! ## Module Structure
//!
//! - [`point`] - Selection point representing a date with optional time
//! - [`range`] - Normalized selection range (start <= end)
//! - [`state`] - Selection state for tracking drag selection
//! - [`drag`] - Event drag state for moving events
//!
//! # Usage Flow for Selection
//!
//! ```text
//! User presses on day/time cell
//!         │
//!         ▼
//! Message::SelectionStart(date, time)
//!         │
//!         ▼
//! SelectionState { start: (date, time), end: (date, time), active: true }
//!         │
//!         ▼
//! User drags over other cells
//!         │
//!         ▼
//! Message::SelectionUpdate(date, time)  [for each cell entered]
//!         │
//!         ▼
//! SelectionState { start: original, end: current, active: true }
//!         │
//!         ▼
//! User releases mouse
//!         │
//!         ▼
//! Message::SelectionEnd
//!         │
//!         ├── Month view: Date range → Quick event or event dialog
//!         └── Week/Day view: Time range → Event dialog with times pre-filled
//! ```

mod drag;
mod point;
mod range;
mod state;

// Re-export public types
#[allow(unused_imports)] // Part of selection API
pub use drag::{DragPreviewInfo, DragTarget, EventDragState};
#[allow(unused_imports)] // Part of selection API, used by tests
pub use point::SelectionPoint;
#[allow(unused_imports)] // Part of selection API, used by tests
pub use range::SelectionRange;
pub use state::SelectionState;

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{NaiveDate, NaiveTime};

    #[test]
    fn test_selection_point_date_only() {
        let date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
        let point = SelectionPoint::date_only(date);

        assert_eq!(point.date(), date);
        assert!(point.time().is_none());
        assert!(point.is_date_only());
    }

    #[test]
    fn test_selection_point_with_time() {
        let date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
        let time = NaiveTime::from_hms_opt(9, 30, 0).unwrap();
        let point = SelectionPoint::with_time(date, time);

        assert_eq!(point.date(), date);
        assert_eq!(point.time(), Some(time));
        assert!(!point.is_date_only());
    }

    #[test]
    fn test_selection_state_start() {
        let mut state = SelectionState::new();
        let date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();

        state.start(date);

        assert!(state.is_active);
        assert_eq!(state.start_date(), Some(date));
        assert_eq!(state.end_date(), Some(date));
    }

    #[test]
    fn test_selection_state_start_with_time() {
        let mut state = SelectionState::new();
        let date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
        let time = NaiveTime::from_hms_opt(9, 0, 0).unwrap();

        state.start_with_time(date, time);

        assert!(state.is_active);
        let range = state.get_range().unwrap();
        assert_eq!(range.start_time(), Some(time));
    }

    #[test]
    fn test_selection_state_update() {
        let mut state = SelectionState::new();
        let start = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
        let end = NaiveDate::from_ymd_opt(2024, 1, 18).unwrap();

        state.start(start);
        state.update(end);

        assert!(state.is_active);
        assert_eq!(state.start_date(), Some(start));
        assert_eq!(state.end_date(), Some(end));
    }

    #[test]
    fn test_selection_state_end() {
        let mut state = SelectionState::new();
        let start = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
        let end = NaiveDate::from_ymd_opt(2024, 1, 18).unwrap();

        state.start(start);
        state.update(end);
        let range = state.end();

        assert!(!state.is_active);
        assert!(range.is_some());
        let range = range.unwrap();
        assert_eq!(range.start_date(), start);
        assert_eq!(range.end_date(), end);
    }

    #[test]
    fn test_selection_range_normalized() {
        let early = NaiveDate::from_ymd_opt(2024, 1, 10).unwrap();
        let late = NaiveDate::from_ymd_opt(2024, 1, 20).unwrap();

        // Forward selection
        let range1 = SelectionRange::from_dates(early, late);
        assert_eq!(range1.start_date(), early);
        assert_eq!(range1.end_date(), late);

        // Backward selection (dragging up/left)
        let range2 = SelectionRange::from_dates(late, early);
        assert_eq!(range2.start_date(), early);
        assert_eq!(range2.end_date(), late);
    }

    #[test]
    fn test_selection_range_contains() {
        let start = NaiveDate::from_ymd_opt(2024, 1, 10).unwrap();
        let end = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
        let range = SelectionRange::from_dates(start, end);

        assert!(range.contains_date(start));
        assert!(range.contains_date(end));
        assert!(range.contains_date(NaiveDate::from_ymd_opt(2024, 1, 12).unwrap()));
        assert!(!range.contains_date(NaiveDate::from_ymd_opt(2024, 1, 9).unwrap()));
        assert!(!range.contains_date(NaiveDate::from_ymd_opt(2024, 1, 16).unwrap()));
    }

    #[test]
    fn test_selection_range_is_multi_day() {
        let date1 = NaiveDate::from_ymd_opt(2024, 1, 10).unwrap();
        let date2 = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();

        let single = SelectionRange::from_dates(date1, date1);
        assert!(!single.is_multi_day());

        let multi = SelectionRange::from_dates(date1, date2);
        assert!(multi.is_multi_day());
    }

    #[test]
    fn test_selection_range_day_count() {
        let start = NaiveDate::from_ymd_opt(2024, 1, 10).unwrap();
        let end = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
        let range = SelectionRange::from_dates(start, end);

        assert_eq!(range.day_count(), 6); // 10, 11, 12, 13, 14, 15
    }

    #[test]
    fn test_selection_range_dates_iterator() {
        let start = NaiveDate::from_ymd_opt(2024, 1, 10).unwrap();
        let end = NaiveDate::from_ymd_opt(2024, 1, 12).unwrap();
        let range = SelectionRange::from_dates(start, end);

        let dates: Vec<_> = range.dates().collect();
        assert_eq!(dates.len(), 3);
        assert_eq!(dates[0], NaiveDate::from_ymd_opt(2024, 1, 10).unwrap());
        assert_eq!(dates[1], NaiveDate::from_ymd_opt(2024, 1, 11).unwrap());
        assert_eq!(dates[2], NaiveDate::from_ymd_opt(2024, 1, 12).unwrap());
    }

    #[test]
    fn test_selection_state_cancel() {
        let mut state = SelectionState::new();
        let date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();

        state.start(date);
        assert!(state.is_active);

        state.cancel();
        assert!(!state.is_active);
        assert!(state.start_date().is_none());
        assert!(state.end_date().is_none());
    }

    // EventDragState tests

    #[test]
    fn test_event_drag_state_start() {
        let mut state = EventDragState::new();
        let date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();

        state.start("event-123".to_string(), date, "Test Event".to_string(), "#0000ff".to_string());

        assert!(state.is_active);
        assert_eq!(state.event_uid, Some("event-123".to_string()));
        assert_eq!(state.original_date, Some(date));
        assert_eq!(state.target_date(), Some(date));
        assert_eq!(state.event_summary(), Some("Test Event"));
        assert_eq!(state.event_color(), Some("#0000ff"));
    }

    #[test]
    fn test_event_drag_state_start_with_time() {
        let mut state = EventDragState::new();
        let date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
        let time = NaiveTime::from_hms_opt(9, 0, 0).unwrap();

        state.start_with_time("event-123".to_string(), date, time, "Test Event".to_string(), "#0000ff".to_string());

        assert!(state.is_active);
        assert_eq!(state.original_time, Some(time));
        assert_eq!(state.target_time(), Some(time));
    }

    #[test]
    fn test_event_drag_state_update() {
        let mut state = EventDragState::new();
        let original = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
        let target = NaiveDate::from_ymd_opt(2024, 1, 18).unwrap();

        state.start("event-123".to_string(), original, "Test Event".to_string(), "#0000ff".to_string());
        state.update(target);

        assert!(state.is_active);
        assert_eq!(state.original_date, Some(original));
        assert_eq!(state.target_date(), Some(target));
    }

    #[test]
    fn test_event_drag_state_end_with_move() {
        let mut state = EventDragState::new();
        let original = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
        let target = NaiveDate::from_ymd_opt(2024, 1, 18).unwrap();

        state.start("event-123".to_string(), original, "Test Event".to_string(), "#0000ff".to_string());
        state.update(target);
        let result = state.end();

        assert!(!state.is_active);
        assert!(result.is_some());
        let (uid, orig, tgt) = result.unwrap();
        assert_eq!(uid, "event-123");
        assert_eq!(orig, original);
        assert_eq!(tgt, target);
    }

    #[test]
    fn test_event_drag_state_end_same_date() {
        let mut state = EventDragState::new();
        let date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();

        state.start("event-123".to_string(), date, "Test Event".to_string(), "#0000ff".to_string());
        // Don't update - target stays same as original
        let result = state.end();

        assert!(!state.is_active);
        assert!(result.is_none()); // No move needed
    }

    #[test]
    fn test_event_drag_state_end_with_time() {
        let mut state = EventDragState::new();
        let date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
        let original_time = NaiveTime::from_hms_opt(9, 0, 0).unwrap();
        let target_time = NaiveTime::from_hms_opt(14, 0, 0).unwrap();

        state.start_with_time("event-123".to_string(), date, original_time, "Test Event".to_string(), "#0000ff".to_string());
        state.update_with_time(date, target_time);
        let result = state.end_with_time();

        assert!(!state.is_active);
        assert!(result.is_some());
        let (uid, orig_date, orig_time, tgt_date, tgt_time) = result.unwrap();
        assert_eq!(uid, "event-123");
        assert_eq!(orig_date, date);
        assert_eq!(orig_time, Some(original_time));
        assert_eq!(tgt_date, date);
        assert_eq!(tgt_time, Some(target_time));
    }

    #[test]
    fn test_event_drag_state_get_offset() {
        let mut state = EventDragState::new();
        let original = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
        let target = NaiveDate::from_ymd_opt(2024, 1, 18).unwrap();

        state.start("event-123".to_string(), original, "Test Event".to_string(), "#0000ff".to_string());
        state.update(target);

        assert_eq!(state.get_offset(), Some(3)); // 3 days forward
    }

    #[test]
    fn test_event_drag_state_cancel() {
        let mut state = EventDragState::new();
        let date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();

        state.start("event-123".to_string(), date, "Test Event".to_string(), "#0000ff".to_string());
        assert!(state.is_active);

        state.cancel();
        assert!(!state.is_active);
        assert!(state.event_uid.is_none());
        assert!(state.event_summary().is_none());
        assert!(state.event_color().is_none());
    }

    #[test]
    fn test_drag_preview_info() {
        let mut preview = DragPreviewInfo::new();

        preview.set_event_info("Test".to_string(), "#ff0000".to_string());
        assert_eq!(preview.summary, Some("Test".to_string()));
        assert_eq!(preview.color, Some("#ff0000".to_string()));

        preview.update_cursor(100.0, 200.0);
        assert_eq!(preview.cursor_position, Some((100.0, 200.0)));

        preview.reset();
        assert!(preview.summary.is_none());
        assert!(preview.color.is_none());
        assert!(preview.cursor_position.is_none());
    }
}
