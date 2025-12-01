//! Utility types and functions for the week view
//!
//! Contains event positioning logic, overlap detection, and helper functions.

use chrono::{NaiveDate, NaiveTime, Timelike};
use std::collections::HashMap;

use crate::components::DisplayEvent;
use crate::ui_constants::HOUR_ROW_HEIGHT;

/// Represents an event with its calculated column position for overlap handling
#[derive(Clone)]
pub struct PositionedEvent {
    pub event: DisplayEvent,
    pub column: usize,
    pub total_columns: usize,
}

/// Height of the day header row
pub const DAY_HEADER_HEIGHT: f32 = 60.0;

/// Minimum height for the all-day events section
pub const ALL_DAY_MIN_HEIGHT: f32 = 28.0;

/// Height per all-day event row
pub const ALL_DAY_EVENT_HEIGHT: f32 = 22.0;

/// Spacing between all-day events
pub const ALL_DAY_SPACING: f32 = 2.0;

/// Separate events into all-day and timed categories
pub fn separate_events(
    events_by_date: &HashMap<NaiveDate, Vec<DisplayEvent>>,
    week_days: &[NaiveDate],
) -> (HashMap<NaiveDate, Vec<DisplayEvent>>, HashMap<NaiveDate, Vec<DisplayEvent>>) {
    let mut all_day: HashMap<NaiveDate, Vec<DisplayEvent>> = HashMap::new();
    let mut timed: HashMap<NaiveDate, Vec<DisplayEvent>> = HashMap::new();

    for day in week_days {
        if let Some(day_events) = events_by_date.get(day) {
            for event in day_events {
                if event.all_day {
                    all_day.entry(*day).or_default().push(event.clone());
                } else {
                    timed.entry(*day).or_default().push(event.clone());
                }
            }
        }
    }

    (all_day, timed)
}

/// Calculate the maximum number of all-day event slots needed
pub fn calculate_max_all_day_slots(all_day_events: &HashMap<NaiveDate, Vec<DisplayEvent>>) -> usize {
    all_day_events.values().map(|v| v.len()).max().unwrap_or(0)
}

/// Get the time range of an event in minutes from midnight
pub fn event_time_range(event: &DisplayEvent) -> (u32, u32) {
    let start = event.start_time
        .map(|t| t.hour() * 60 + t.minute())
        .unwrap_or(0);
    let end = event.end_time
        .map(|t| t.hour() * 60 + t.minute())
        .unwrap_or(start + 60); // Default 1 hour if no end time

    // Ensure end is after start
    let end = if end <= start { start + 30 } else { end };

    (start, end)
}

/// Check if two events overlap in time
pub fn events_overlap(e1: &DisplayEvent, e2: &DisplayEvent) -> bool {
    let Some(start1) = e1.start_time else { return false };
    let Some(end1) = e1.end_time else { return false };
    let Some(start2) = e2.start_time else { return false };
    let Some(end2) = e2.end_time else { return false };

    // Events overlap if one starts before the other ends
    start1 < end2 && start2 < end1
}

/// Calculate column positions for overlapping events
/// Returns events with their assigned column and total columns in their overlap group
pub fn calculate_event_columns(events: &[DisplayEvent]) -> Vec<PositionedEvent> {
    if events.is_empty() {
        return Vec::new();
    }

    // Sort events by start time, then by end time (shorter events first)
    let mut sorted: Vec<_> = events.iter().cloned().collect();
    sorted.sort_by(|a, b| {
        let start_cmp = a.start_time.cmp(&b.start_time);
        if start_cmp == std::cmp::Ordering::Equal {
            a.end_time.cmp(&b.end_time)
        } else {
            start_cmp
        }
    });

    let mut positioned: Vec<PositionedEvent> = Vec::new();
    let mut column_ends: Vec<NaiveTime> = Vec::new(); // Track when each column becomes free

    for event in sorted {
        let start = event.start_time.unwrap_or(NaiveTime::from_hms_opt(0, 0, 0).unwrap());
        let end = event.end_time.unwrap_or(NaiveTime::from_hms_opt(23, 59, 59).unwrap());

        // Find the first column where this event can fit (column is free before this event starts)
        let mut assigned_column = None;
        for (col_idx, col_end) in column_ends.iter_mut().enumerate() {
            if *col_end <= start {
                // This column is free, use it
                *col_end = end;
                assigned_column = Some(col_idx);
                break;
            }
        }

        // If no existing column is free, create a new one
        let column = assigned_column.unwrap_or_else(|| {
            column_ends.push(end);
            column_ends.len() - 1
        });

        positioned.push(PositionedEvent {
            event,
            column,
            total_columns: 0, // Will be set in second pass
        });
    }

    // Second pass: for each event, find the max columns in its overlap group
    for i in 0..positioned.len() {
        let mut max_col = positioned[i].column;

        // Check all events that overlap with this one
        for j in 0..positioned.len() {
            if i != j && events_overlap(&positioned[i].event, &positioned[j].event) {
                max_col = max_col.max(positioned[j].column);
            }
        }

        positioned[i].total_columns = max_col + 1;
    }

    positioned
}

/// Calculate the height for a time span in pixels
#[allow(dead_code)]
pub fn time_span_to_height(start_mins: u32, end_mins: u32) -> f32 {
    ((end_mins - start_mins) as f32 / 60.0) * HOUR_ROW_HEIGHT
}

/// Calculate the vertical offset for a time in pixels
#[allow(dead_code)]
pub fn time_to_offset(hour: u32, minute: u32) -> f32 {
    (hour as f32 + minute as f32 / 60.0) * HOUR_ROW_HEIGHT
}
