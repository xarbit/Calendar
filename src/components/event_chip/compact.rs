//! Compact events rendering
//!
//! Renders events as small colored indicators when space is limited.

use chrono::NaiveDate;
use cosmic::iced::Length;
use cosmic::widget::{column, container, row};
use cosmic::{widget, Element};

use crate::components::color_picker::parse_hex_color;
use crate::message::Message;
use crate::ui_constants::{
    SPACING_TINY, COLOR_DEFAULT_GRAY, COMPACT_EVENT_HEIGHT, DATE_EVENT_SPACING,
};

use super::types::DisplayEvent;

/// Result of rendering compact events
pub struct CompactEventsResult {
    /// The rendered element containing all compact event indicators
    pub element: Option<Element<'static, Message>>,
    /// Number of events not shown
    pub overflow_count: usize,
}

/// Render a compact timed event indicator (small colored dot)
fn render_compact_timed_indicator(color: cosmic::iced::Color) -> Element<'static, Message> {
    container(widget::text(""))
        .width(Length::Fixed(COMPACT_EVENT_HEIGHT))
        .height(Length::Fixed(COMPACT_EVENT_HEIGHT))
        .style(move |_theme: &cosmic::Theme| {
            container::Style {
                background: Some(cosmic::iced::Background::Color(color)),
                border: cosmic::iced::Border {
                    color: cosmic::iced::Color::TRANSPARENT,
                    width: 0.0,
                    radius: (COMPACT_EVENT_HEIGHT / 2.0).into(),
                },
                ..Default::default()
            }
        })
        .into()
}

/// Empty compact placeholder to maintain slot alignment
fn render_compact_empty_placeholder() -> Element<'static, Message> {
    container(widget::text(""))
        .width(Length::Fill)
        .height(Length::Fixed(COMPACT_EVENT_HEIGHT))
        .into()
}

/// Render events in compact mode (thin colored lines/dots without text)
/// Used when cell size is too small for full event chips.
/// Uses Tetris-style slot filling: timed events fill empty slots where date events aren't present.
///
/// # Arguments
/// * `events` - Events to render
/// * `max_visible` - Maximum number of compact indicators to show
/// * `current_date` - The date of the cell (for calculating span position)
/// * `day_occupied_slots` - Slots occupied by date events on THIS specific day
/// * `week_max_slot` - Maximum slot index for the week (for consistent vertical positioning)
pub fn render_compact_events(
    events: Vec<DisplayEvent>,
    max_visible: usize,
    _current_date: NaiveDate,
    day_occupied_slots: &std::collections::HashSet<usize>,
    week_max_slot: Option<usize>,
) -> CompactEventsResult {
    // Separate all-day and timed events
    let (all_day_events, mut timed_events): (Vec<_>, Vec<_>) =
        events.into_iter().partition(|e| e.all_day);

    // Sort timed events by start time
    timed_events.sort_by(|a, b| a.start_time.cmp(&b.start_time));

    // Calculate total slots from week_max_slot
    let total_slots = week_max_slot.map(|m| m + 1).unwrap_or(0);
    let total_events = all_day_events.len() + timed_events.len();
    let mut shown = 0;

    // Use same spacing as overlay for proper alignment
    let mut col = column().spacing(DATE_EVENT_SPACING as u16);
    let mut has_content = false;

    // Track which timed events we've used
    let mut timed_event_iter = timed_events.iter().peekable();

    // Tetris-style rendering: for each slot position, either:
    // - Show a placeholder if the slot is occupied by a date event (overlay renders it)
    // - Show a timed event dot if slot is empty (fill the gap)
    for slot in 0..total_slots {
        if shown >= max_visible {
            break;
        }

        if day_occupied_slots.contains(&slot) {
            // Slot is occupied by a date event - render placeholder
            col = col.push(render_compact_empty_placeholder());
        } else {
            // Slot is empty - fill with a timed event dot if available
            if let Some(event) = timed_event_iter.next() {
                let color = parse_hex_color(&event.color).unwrap_or(COLOR_DEFAULT_GRAY);
                col = col.push(render_compact_timed_indicator(color));
            } else {
                // No more timed events - render placeholder to maintain slot alignment
                col = col.push(render_compact_empty_placeholder());
            }
        }
        shown += 1;
        has_content = true;
    }

    // Render any remaining timed events as dots in a row
    let remaining_timed: Vec<_> = timed_event_iter.collect();
    if !remaining_timed.is_empty() && shown < max_visible {
        let mut dots_row = row().spacing(SPACING_TINY);
        let remaining_slots = max_visible - shown;

        for (i, event) in remaining_timed.iter().enumerate() {
            if i >= remaining_slots {
                break;
            }
            let color = parse_hex_color(&event.color).unwrap_or(COLOR_DEFAULT_GRAY);
            dots_row = dots_row.push(render_compact_timed_indicator(color));
            shown += 1;
        }

        col = col.push(dots_row);
        has_content = true;
    }

    let overflow_count = if total_events > shown {
        total_events - shown
    } else {
        0
    };

    CompactEventsResult {
        element: if has_content { Some(col.into()) } else { None },
        overflow_count,
    }
}
