//! Week view rendering
//!
//! Displays a full week with hourly time slots and events.
//!
//! ## Module Structure
//!
//! - [`header`] - Day header row and all-day events section
//! - [`time_grid`] - Time labels column and hour cell grid
//! - [`events`] - Timed event chip rendering and positioning
//! - [`time_indicator`] - Current time line and dot
//! - [`quick_event`] - Inline event creation input
//! - [`utils`] - Shared types and utility functions

mod events;
mod header;
mod quick_event;
mod time_grid;
mod time_indicator;
mod utils;

use chrono::{Datelike, NaiveDate, NaiveTime, Timelike};
use cosmic::iced::widget::stack;
use cosmic::iced::Length;
use cosmic::widget::{column, container, scrollable};
use cosmic::Element;
use std::collections::HashMap;

use crate::components::DisplayEvent;
use crate::dialogs::ActiveDialog;
use crate::locale::LocalePreferences;
use crate::message::Message;
use crate::models::WeekState;
use crate::selection::SelectionState;
use crate::ui_constants::HOUR_ROW_HEIGHT;

use events::render_events_overlay_layer;
use header::render_header_section;
use quick_event::render_quick_event_input_layer;
use time_grid::{render_hour_grid_background, render_time_labels_column};
use time_indicator::render_time_indicator_layer;
use utils::{
    calculate_event_columns, calculate_max_all_day_slots, separate_events,
    ALL_DAY_EVENT_HEIGHT, ALL_DAY_MIN_HEIGHT, ALL_DAY_SPACING,
};

/// Returns the scrollable ID for the week view time grid
pub fn week_time_grid_id() -> cosmic::iced_core::id::Id {
    cosmic::iced_core::id::Id::new("week_time_grid")
}

/// Events grouped by day for display in the week view
pub struct WeekViewEvents<'a> {
    /// Events for each day, keyed by date
    pub events_by_date: &'a HashMap<NaiveDate, Vec<DisplayEvent>>,
    /// Currently selected event UID (for visual feedback)
    pub selected_event_uid: Option<&'a str>,
    /// Selection state for time slot highlighting
    pub selection: &'a SelectionState,
    /// Active dialog state (for quick event input)
    pub active_dialog: &'a ActiveDialog,
    /// Selected calendar color (for quick event styling)
    pub calendar_color: &'a str,
}

/// Render the week view with events
pub fn render_week_view<'a>(
    week_state: &'a WeekState,
    locale: &'a LocalePreferences,
    events: Option<WeekViewEvents<'a>>,
) -> Element<'a, Message> {
    // Extract selected event UID for selection highlighting
    let selected_event_uid = events.as_ref().and_then(|e| e.selected_event_uid);

    // Extract selection state for time slot highlighting
    let selection = events.as_ref().map(|e| e.selection);

    // Extract active dialog and calendar color for quick event input
    let active_dialog = events.as_ref().map(|e| e.active_dialog);
    let calendar_color = events.as_ref().map(|e| e.calendar_color);

    // Separate events into all-day and timed
    let (all_day_events, timed_events) = if let Some(ref ev) = events {
        separate_events(ev.events_by_date, &week_state.days)
    } else {
        (HashMap::new(), HashMap::new())
    };

    // Calculate how many rows we need for all-day events
    let max_all_day_slots = calculate_max_all_day_slots(&all_day_events);
    let all_day_section_height = ALL_DAY_MIN_HEIGHT + (max_all_day_slots as f32 * (ALL_DAY_EVENT_HEIGHT + ALL_DAY_SPACING));

    // Day headers with all-day events section
    let header_section = render_header_section(week_state, locale, &all_day_events, all_day_section_height, selected_event_uid);

    // Time grid with timed events
    let time_grid = render_time_grid_with_events(locale, week_state, &timed_events, selected_event_uid, selection, active_dialog, calendar_color);

    let content = column()
        .spacing(0)
        .push(header_section)
        .push(
            scrollable(time_grid)
                .id(week_time_grid_id())
                .on_scroll(Message::WeekViewScroll)
                .height(Length::Fill)
        );

    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

/// Render the time grid with timed events spanning their full duration
fn render_time_grid_with_events<'a>(
    locale: &'a LocalePreferences,
    week_state: &'a WeekState,
    timed_events: &HashMap<NaiveDate, Vec<DisplayEvent>>,
    selected_event_uid: Option<&'a str>,
    selection: Option<&'a SelectionState>,
    active_dialog: Option<&'a ActiveDialog>,
    calendar_color: Option<&'a str>,
) -> Element<'a, Message> {
    // Get current time for the "now" indicator
    let now = chrono::Local::now();
    let today = now.date_naive();
    let current_hour = now.hour();
    let current_minute = now.minute();

    // Check if today is in the current week
    let today_column_index = week_state.days.iter().position(|d| *d == today);

    // Check if there's an active timed quick event to display
    let quick_event_data = active_dialog.and_then(|dialog| {
        if let ActiveDialog::QuickEvent { start_date, start_time: Some(start_time), end_time: Some(end_time), text, .. } = dialog {
            Some((*start_date, *start_time, *end_time, text.as_str()))
        } else {
            None
        }
    });

    // Build the grid as a row: time labels column + day columns
    let mut main_row = cosmic::widget::row().spacing(0);

    // Time labels column
    let time_labels = render_time_labels_column(locale, today_column_index.is_some(), current_hour);
    main_row = main_row.push(time_labels);

    // Day columns with events
    for (day_idx, date) in week_state.days.iter().enumerate() {
        let is_weekend = locale.is_weekend(date.weekday());
        let is_today_column = today_column_index == Some(day_idx);
        let day_events = timed_events.get(date).cloned().unwrap_or_default();

        // Check if this day has the quick event input
        let day_quick_event = quick_event_data.and_then(|(qe_date, start, end, text)| {
            if qe_date == *date {
                Some((start, end, text, calendar_color.unwrap_or("#3B82F6")))
            } else {
                None
            }
        });

        let day_column = render_day_column_with_events(
            *date,
            &day_events,
            is_weekend,
            is_today_column,
            today_column_index.is_some(), // today_in_week - true if today is visible in this week
            current_hour,
            current_minute,
            selected_event_uid,
            selection,
            day_quick_event,
        );

        main_row = main_row.push(day_column);
    }

    main_row.into()
}

/// Render a single day column with events spanning their full duration using stack overlay
fn render_day_column_with_events(
    date: NaiveDate,
    events: &[DisplayEvent],
    is_weekend: bool,
    is_today: bool,
    today_in_week: bool,
    current_hour: u32,
    current_minute: u32,
    selected_event_uid: Option<&str>,
    selection: Option<&SelectionState>,
    quick_event: Option<(NaiveTime, NaiveTime, &str, &str)>, // (start_time, end_time, text, color)
) -> Element<'static, Message> {
    // Build the base hour grid (background layer) - without time indicator
    let hour_grid = render_hour_grid_background(date, is_weekend, selection);

    // Build the time indicator layer (rendered on top of events)
    let time_indicator_layer = if today_in_week {
        let minute_offset = (current_minute as f32 / 60.0) * HOUR_ROW_HEIGHT;
        Some(render_time_indicator_layer(current_hour, minute_offset, is_today))
    } else {
        None
    };

    // Build quick event input layer if active
    let quick_event_layer = quick_event.map(|(start_time, end_time, text, color)| {
        render_quick_event_input_layer(start_time, end_time, text.to_string(), color.to_string())
    });

    // If no events and no quick event, just return the grid with time indicator on top
    if events.is_empty() && quick_event_layer.is_none() {
        return if let Some(time_layer) = time_indicator_layer {
            container(stack![hour_grid, time_layer])
                .width(Length::Fill)
                .into()
        } else {
            container(hour_grid)
                .width(Length::Fill)
                .into()
        };
    }

    // Calculate column assignments for overlapping events
    let positioned_events = calculate_event_columns(events);
    let max_columns = positioned_events.iter().map(|p| p.total_columns).max().unwrap_or(1).max(1);

    // Build the events overlay layer
    let events_layer = render_events_overlay_layer(date, &positioned_events, max_columns, selected_event_uid);

    // Stack order: grid (bottom) -> events -> time indicator -> quick event (top)
    // Time indicator must be above events so it's always visible
    let stacked: Element<'static, Message> = match (time_indicator_layer, quick_event_layer) {
        (Some(time_layer), Some(qe_layer)) => stack![
            hour_grid,
            events_layer,
            time_layer,
            qe_layer
        ].into(),
        (Some(time_layer), None) => stack![
            hour_grid,
            events_layer,
            time_layer
        ].into(),
        (None, Some(qe_layer)) => stack![
            hour_grid,
            events_layer,
            qe_layer
        ].into(),
        (None, None) => stack![
            hour_grid,
            events_layer
        ].into(),
    };

    container(stacked)
        .width(Length::Fill)
        .into()
}
