//! Month view module
//!
//! This module contains the month calendar view rendering logic, organized into submodules:
//! - `header`: Weekday header row rendering
//! - `overlay`: Slot computation and date event overlay rendering
//! - `events`: Date event chip rendering
//! - `selection`: Quick event selection overlay

mod header;
mod overlay;
mod events;
mod selection;

use chrono::{Datelike, NaiveDate};
use cosmic::iced::widget::stack;
use cosmic::iced::{alignment, Length, Size};
use cosmic::widget::{column, container, row, responsive};
use cosmic::{widget, Element};

use crate::components::spacer::fill_spacer;
use crate::components::{render_day_cell_with_events, DayCellConfig, DisplayEvent, should_use_compact};
use crate::dialogs::ActiveDialog;
use crate::locale::LocalePreferences;
use crate::message::Message;
use crate::models::{CalendarDay, CalendarState};
use crate::selection::SelectionState;
use crate::ui_constants::{
    FONT_SIZE_SMALL, PADDING_MONTH_GRID, PADDING_SMALL,
    SPACING_TINY, WEEK_NUMBER_WIDTH,
};

use header::render_weekday_header;
use overlay::{compute_week_event_slots, render_date_events_overlay, WEEKDAY_HEADER_HEIGHT};
use selection::render_spanning_overlay;

/// Minimum width per day cell to use full weekday names
/// Below this threshold, short names are used
const MIN_CELL_WIDTH_FOR_FULL_NAMES: f32 = 100.0;

/// Events grouped by day for display in the month view
pub struct MonthViewEvents<'a> {
    /// Events for each day, keyed by full date (supports adjacent month days)
    pub events_by_date: &'a std::collections::HashMap<NaiveDate, Vec<DisplayEvent>>,
    /// Quick event editing state: (date, text, calendar_color)
    pub quick_event: Option<(NaiveDate, &'a str, &'a str)>,
    /// Selection state for drag selection
    pub selection: &'a SelectionState,
    /// Active dialog state (for showing selection highlight during quick event input)
    pub active_dialog: &'a ActiveDialog,
    /// Currently selected event UID (for visual feedback)
    pub selected_event_uid: Option<&'a str>,
    /// Whether an event drag operation is currently active
    pub event_drag_active: bool,
    /// The UID of the event currently being dragged (for dimming original)
    pub dragging_event_uid: Option<&'a str>,
    /// The current drop target date during drag (for highlighting target cell)
    pub drag_target_date: Option<NaiveDate>,
}

pub fn render_month_view<'a>(
    calendar_state: &CalendarState,
    selected_date: Option<NaiveDate>,
    locale: &LocalePreferences,
    show_week_numbers: bool,
    events: Option<MonthViewEvents<'a>>,
) -> Element<'a, Message> {
    let mut grid = column().spacing(SPACING_TINY).padding(PADDING_MONTH_GRID);

    // Responsive weekday header - uses short names when cells are narrow
    let week_number_offset = if show_week_numbers { WEEK_NUMBER_WIDTH } else { 0.0 };
    let header = responsive(move |size: Size| {
        // Calculate approximate cell width (7 days + spacing)
        let available_for_days = size.width - week_number_offset - (SPACING_TINY as f32 * 6.0);
        let cell_width = available_for_days / 7.0;
        let use_short_names = cell_width < MIN_CELL_WIDTH_FOR_FULL_NAMES;
        render_weekday_header(show_week_numbers, use_short_names)
    });

    // Fixed height container for the header to prevent it from expanding
    grid = grid.push(container(header).height(Length::Fixed(WEEKDAY_HEADER_HEIGHT)));

    // Get week numbers for the month
    let week_numbers = calendar_state.week_numbers();

    // Use pre-calculated weeks from CalendarState cache (with adjacent month days)
    for (week_index, week) in calendar_state.weeks_full.iter().enumerate() {
        // Compute slot assignments for date events in this week
        let week_slot_info = events
            .as_ref()
            .map(|e| compute_week_event_slots(week, e.events_by_date));

        // Extract the slots map for compatibility with existing code
        let event_slots = week_slot_info.as_ref()
            .map(|info| info.slots.clone())
            .unwrap_or_default();

        // Compute the max slot for this week - all day cells need this for consistent placeholders
        let week_max_slot = event_slots.values().copied().max();

        let mut week_row = row().spacing(SPACING_TINY).height(Length::Fill);

        // Week number cell (only if enabled)
        if show_week_numbers {
            let week_number = week_numbers.get(week_index).copied().unwrap_or(0);
            week_row = week_row.push(
                container(
                    widget::text(format!("{}", week_number))
                        .size(FONT_SIZE_SMALL)
                )
                .width(Length::Fixed(WEEK_NUMBER_WIDTH))
                .height(Length::Fill)
                .padding(PADDING_SMALL)
                .align_y(alignment::Vertical::Center)
            );
        }

        // Day cells
        for (day_col, calendar_day) in week.iter().enumerate() {
            let CalendarDay { year, month, day, is_current_month } = *calendar_day;

            // Check if today (need to compare full date)
            let today = chrono::Local::now();
            let is_today = year == today.year()
                && month == today.month()
                && day == today.day();

            // Check if this day is selected (works for both current and adjacent month days)
            // Don't show cell selection if an event is selected - event selection takes priority
            let cell_date = NaiveDate::from_ymd_opt(year, month, day);
            let has_event_selected = events.as_ref()
                .and_then(|e| e.selected_event_uid)
                .is_some();
            let is_selected = !has_event_selected && selected_date.is_some() && cell_date == selected_date;

            // Get weekday for weekend detection
            let weekday = chrono::NaiveDate::from_ymd_opt(year, month, day)
                .map(|d| d.weekday())
                .unwrap_or(chrono::Weekday::Mon);
            let is_weekend = locale.is_weekend(weekday);

            // Get events for this day using full date as key (works for adjacent months too)
            // Include all events - multi-day events show in each cell they span
            let day_events: Vec<DisplayEvent> = if let Some(date) = NaiveDate::from_ymd_opt(year, month, day) {
                events
                    .as_ref()
                    .and_then(|e| e.events_by_date.get(&date))
                    .cloned()
                    .unwrap_or_default()
            } else {
                vec![]
            };

            // Quick event input is always rendered as a spanning overlay (even for single-day)
            // This provides consistent UX for all quick event creation
            let quick_event_data: Option<(String, String)> = None;

            // Check if this day is in the current drag selection range
            // Also check if there's an active multi-day quick event that includes this date
            let (is_in_selection, selection_active) = if let Some(cell_date) = cell_date {
                events.as_ref().map(|e| {
                    // Show highlight if: actively dragging OR in quick event date range
                    let in_drag_selection = e.selection.contains(cell_date);
                    let in_quick_event_range = e.active_dialog.is_date_in_quick_event_range(cell_date);
                    let is_active = e.selection.is_active || e.active_dialog.is_multi_day_quick_event();
                    (in_drag_selection || in_quick_event_range, is_active)
                }).unwrap_or((false, false))
            } else {
                (false, false)
            };

            // Get selected event UID from events if available
            let selected_event_uid = events.as_ref()
                .and_then(|e| e.selected_event_uid)
                .map(|s| s.to_string());

            // Check if event drag is active
            let event_drag_active = events.as_ref()
                .map(|e| e.event_drag_active)
                .unwrap_or(false);

            // Get the UID of the event being dragged (for dimming its original position)
            let dragging_event_uid = events.as_ref()
                .and_then(|e| e.dragging_event_uid)
                .map(|s| s.to_string());

            // Check if this cell is the current drop target
            let is_drag_target = cell_date.is_some() && events.as_ref()
                .and_then(|e| e.drag_target_date)
                .map(|target| cell_date == Some(target))
                .unwrap_or(false);

            // Get occupied slots for this specific day (for Tetris-style rendering)
            let day_occupied_slots = week_slot_info.as_ref()
                .and_then(|info| info.day_occupied_slots.get(day_col).cloned())
                .unwrap_or_default();

            let cell = render_day_cell_with_events(DayCellConfig {
                year,
                month,
                day,
                is_today,
                is_selected,
                is_weekend,
                is_adjacent_month: !is_current_month,
                events: day_events,
                event_slots: event_slots.clone(),
                week_max_slot,
                day_occupied_slots,
                quick_event: quick_event_data,
                is_in_selection,
                selection_active,
                selected_event_uid,
                event_drag_active,
                dragging_event_uid,
                is_drag_target,
            });

            week_row = week_row.push(
                container(cell)
                    .width(Length::Fill)
                    .height(Length::Fill)
            );
        }
        grid = grid.push(week_row);
    }

    // Check if we need to render a spanning quick event overlay
    // Used for all quick events (single-day and multi-day) for consistent UX
    let has_quick_event_overlay = events
        .as_ref()
        .map(|e| e.active_dialog.is_quick_event())
        .unwrap_or(false);

    // Build the final view with overlays
    let base = container(grid)
        .width(Length::Fill)
        .height(Length::Fill);

    // Collect overlays to stack
    let mut layers: Vec<Element<'a, Message>> = vec![base.into()];

    // Add date events overlay (renders all date events as spanning bars)
    // Wrapped in responsive to determine compact mode based on cell size
    if let Some(ref e) = events {
        // Clone data needed for the responsive closure
        let weeks = calendar_state.weeks_full.clone();
        let events_by_date = e.events_by_date.clone();
        let week_number_offset = if show_week_numbers { WEEK_NUMBER_WIDTH } else { 0.0 };
        let selected_uid = e.selected_event_uid.map(|s| s.to_string());
        let event_drag_active = e.event_drag_active;
        let dragging_uid = e.dragging_event_uid.map(|s| s.to_string());

        let responsive_overlay = responsive(move |size: Size| {
            // Calculate approximate cell width (7 days + spacing)
            let available_for_days = size.width - week_number_offset - (SPACING_TINY as f32 * 6.0);
            let cell_width = available_for_days / 7.0;

            // Calculate approximate cell height (total height minus header, divided by weeks)
            let num_weeks = weeks.len().max(1) as f32;
            let available_height = size.height - WEEKDAY_HEADER_HEIGHT - (SPACING_TINY as f32 * num_weeks);
            let cell_height = available_height / num_weeks;

            // Determine if we should use compact mode
            let compact = should_use_compact(cell_width, cell_height);

            if let Some(overlay) = render_date_events_overlay(
                &weeks,
                &events_by_date,
                show_week_numbers,
                compact,
                selected_uid.as_deref(),
                event_drag_active,
                dragging_uid.as_deref(),
            ) {
                overlay
            } else {
                // Return empty container if no events
                fill_spacer()
            }
        });

        layers.push(responsive_overlay.into());
    }

    // Add quick event input overlay on top if active
    if has_quick_event_overlay {
        if let Some(quick_overlay) = events.as_ref().and_then(|e| {
            e.active_dialog.quick_event_range().map(|(start, end, text)| {
                let color = e.quick_event
                    .as_ref()
                    .map(|(_, _, c)| c.to_string())
                    .unwrap_or_else(|| "#3B82F6".to_string());

                render_spanning_overlay(
                    &calendar_state.weeks_full,
                    start,
                    end,
                    text.to_string(),
                    color,
                    show_week_numbers,
                )
            })
        }) {
            layers.push(quick_overlay);
        }
    }

    // Stack all layers
    if layers.len() == 1 {
        layers.pop().unwrap()
    } else {
        stack(layers).into()
    }
}
