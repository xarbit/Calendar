//! Selection handlers for multi-day event creation
//!
//! This module provides view-specific selection handling. Each view has its own
//! submodule with tailored behavior:
//!
//! - **month**: Quick create all-day events (no dialog)
//! - **week**: Open event dialog with specific times (future)
//! - **day**: Open event dialog with specific times (future)
//!
//! The core selection logic (start, update, cancel) is shared across all views.

mod day;
mod month;
mod week;

use chrono::{NaiveDate, NaiveTime};
use log::debug;

use crate::app::CosmicCalendar;
use crate::dialogs::{DialogAction, DialogManager};
use crate::views::CalendarView;

/// Start a drag selection at the given date (mouse press on day cell)
/// This is view-agnostic - all views start selection the same way
pub fn handle_selection_start(app: &mut CosmicCalendar, date: NaiveDate) {
    debug!("handle_selection_start: Starting selection at {}", date);
    app.selection_state.start(date);
}

/// Update the selection end point (mouse move while dragging)
/// This is view-agnostic - all views update selection the same way
pub fn handle_selection_update(app: &mut CosmicCalendar, date: NaiveDate) {
    if app.selection_state.is_active {
        debug!("handle_selection_update: Updating selection to {}", date);
        app.selection_state.update(date);
    }
}

/// End the selection (mouse release)
/// Dispatches to view-specific handlers based on current view
pub fn handle_selection_end(app: &mut CosmicCalendar) {
    debug!("handle_selection_end: Ending selection in {:?} view", app.current_view);

    let Some(range) = app.selection_state.end() else {
        return;
    };

    match app.current_view {
        CalendarView::Month => {
            month::handle_selection_end(app, range.start.date, range.end.date);
        }
        CalendarView::Week => {
            week::handle_selection_end(app, range.start.date, range.end.date);
        }
        CalendarView::Day => {
            day::handle_selection_end(app, range.start.date, range.end.date);
        }
        CalendarView::Year => {
            // Year view: just select the day (no special selection behavior)
            app.set_selected_date(range.start.date);
        }
    }
}

/// Cancel the current selection
pub fn handle_selection_cancel(app: &mut CosmicCalendar) {
    debug!("handle_selection_cancel: Cancelling selection");
    app.selection_state.cancel();
}

// === Time-Based Selection - For Week/Day Views ===

/// Start a time-based selection at the given date and time (mouse press on hour cell)
pub fn handle_time_selection_start(app: &mut CosmicCalendar, date: NaiveDate, time: NaiveTime) {
    debug!("handle_time_selection_start: Starting time selection at {} {:?}", date, time);
    app.selection_state.start_with_time(date, time);
}

/// Update the time selection end point (mouse move while dragging)
pub fn handle_time_selection_update(app: &mut CosmicCalendar, date: NaiveDate, time: NaiveTime) {
    if app.selection_state.is_active {
        debug!("handle_time_selection_update: Updating time selection to {} {:?}", date, time);
        app.selection_state.update_with_time(date, time);
    }
}

/// End the time selection (mouse release) - opens quick event input with the selected time range
/// Only triggers if there was actual dragging (different start/end times or dates)
pub fn handle_time_selection_end(app: &mut CosmicCalendar) {
    debug!("handle_time_selection_end: Ending time selection");

    let Some(range) = app.selection_state.end() else {
        return;
    };

    // Get the time range - time selection should always have times
    let Some(start_time) = range.start_time() else {
        debug!("handle_time_selection_end: No start time, skipping");
        return;
    };

    // If no end time was set (no drag to a different cell), skip
    let Some(end_time) = range.end_time() else {
        debug!("handle_time_selection_end: No end time (single click), skipping");
        return;
    };

    // Check if there was actual movement (different start/end)
    // If start == end (same cell, no drag), don't open the quick event dialog
    // This allows double-click to work properly
    let start_date = range.start.date;
    let end_date = range.end.date;

    if start_date == end_date && start_time == end_time {
        debug!("handle_time_selection_end: No drag detected (same cell), skipping quick event");
        return;
    }

    // Normalize: ensure start <= end
    let (start_time, end_time) = if start_time <= end_time {
        (start_time, end_time)
    } else {
        (end_time, start_time)
    };

    debug!(
        "handle_time_selection_end: Creating quick timed event from {:?} to {:?}",
        start_time, end_time
    );

    // Open quick event input with the selected time range
    DialogManager::handle_action(
        &mut app.active_dialog,
        DialogAction::StartQuickTimedEvent {
            date: range.start_date(),
            start_time,
            end_time,
        },
    );
}
