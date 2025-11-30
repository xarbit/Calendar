use chrono::NaiveTime;
use cosmic::iced::Length;
use cosmic::iced::widget::text::Wrapping;
use cosmic::iced_widget::text_input;
use cosmic::widget::{column, container, row};
use cosmic::{widget, Element};

use crate::components::color_picker::parse_hex_color;
use crate::message::Message;
use crate::ui_constants::{SPACING_TINY, SPACING_XXS, BORDER_RADIUS, COLOR_DEFAULT_GRAY};

/// Size of the colored dot for timed events
const TIMED_EVENT_DOT_SIZE: f32 = 8.0;

/// Event with associated calendar color for display
#[derive(Debug, Clone)]
pub struct DisplayEvent {
    pub uid: String,
    pub summary: String,
    pub color: String,      // Hex color from calendar
    pub all_day: bool,      // Whether this is an all-day event
    pub start_time: Option<NaiveTime>, // Start time for timed events
}

/// Render an all-day event chip with colored background bar
fn render_all_day_chip(summary: String, color: cosmic::iced::Color) -> Element<'static, Message> {
    // Clip container to prevent text overflow, no wrapping
    container(
        widget::text(summary)
            .size(11)
            .wrapping(Wrapping::None) // Prevent text from wrapping to next line
    )
    .padding([2, 4])
    .width(Length::Fill)
    .clip(true) // Clip text that doesn't fit
    .style(move |_theme: &cosmic::Theme| {
        container::Style {
            background: Some(cosmic::iced::Background::Color(color.scale_alpha(0.3))),
            border: cosmic::iced::Border {
                color,
                width: 0.0,
                radius: BORDER_RADIUS.into(),
            },
            text_color: Some(color),
            ..Default::default()
        }
    })
    .into()
}

/// Render a timed event with colored dot + time + name
fn render_timed_event_chip(
    summary: String,
    start_time: Option<NaiveTime>,
    color: cosmic::iced::Color,
) -> Element<'static, Message> {
    // Colored dot
    let dot = container(widget::text(""))
        .width(Length::Fixed(TIMED_EVENT_DOT_SIZE))
        .height(Length::Fixed(TIMED_EVENT_DOT_SIZE))
        .style(move |_theme: &cosmic::Theme| {
            container::Style {
                background: Some(cosmic::iced::Background::Color(color)),
                border: cosmic::iced::Border {
                    color: cosmic::iced::Color::TRANSPARENT,
                    width: 0.0,
                    radius: (TIMED_EVENT_DOT_SIZE / 2.0).into(), // Circular
                },
                ..Default::default()
            }
        });

    // Format time if available
    let display_text = if let Some(time) = start_time {
        format!("{} {}", time.format("%H:%M"), summary)
    } else {
        summary
    };

    let text = widget::text(display_text)
        .size(11)
        .wrapping(Wrapping::None); // Prevent text from wrapping to next line

    // Wrap in container with clip to truncate long text
    container(
        row()
            .spacing(SPACING_XXS)
            .align_y(cosmic::iced::Alignment::Center)
            .push(dot)
            .push(text)
    )
    .width(Length::Fill)
    .clip(true) // Clip text that doesn't fit
    .into()
}

/// Render a small event chip showing the event title with calendar color
/// For all-day events: colored background bar
/// For timed events: colored dot + time + name
pub fn render_event_chip(event: DisplayEvent) -> Element<'static, Message> {
    let color = parse_hex_color(&event.color).unwrap_or(COLOR_DEFAULT_GRAY);

    if event.all_day {
        render_all_day_chip(event.summary, color)
    } else {
        render_timed_event_chip(event.summary, event.start_time, color)
    }
}

/// Render the quick event input field for inline editing
/// Takes ownership of the data to avoid lifetime issues
pub fn render_quick_event_input(
    text: String,
    calendar_color: String,
) -> Element<'static, Message> {
    let color = parse_hex_color(&calendar_color).unwrap_or(COLOR_DEFAULT_GRAY);

    let input = text_input("New event...", &text)
        .on_input(Message::QuickEventTextChanged)
        .on_submit(Message::CommitQuickEvent)
        .size(11)
        .padding([2, 4])
        .width(Length::Fill);

    container(input)
        .width(Length::Fill)
        .style(move |_theme: &cosmic::Theme| {
            container::Style {
                background: Some(cosmic::iced::Background::Color(color.scale_alpha(0.2))),
                border: cosmic::iced::Border {
                    color,
                    width: 1.0,
                    radius: BORDER_RADIUS.into(),
                },
                ..Default::default()
            }
        })
        .into()
}

/// Render a spanning quick event input that covers multiple day columns
/// Used for multi-day event creation from drag selection
///
/// # Arguments
/// * `text` - Current input text
/// * `calendar_color` - Hex color of the selected calendar
/// * `span_columns` - Number of day columns to span (1-7)
/// * `show_week_numbers` - Whether week numbers column is visible (affects left padding)
pub fn render_spanning_quick_event_input(
    text: String,
    calendar_color: String,
    _span_columns: usize, // Reserved for future layout adjustments
) -> Element<'static, Message> {
    let color = parse_hex_color(&calendar_color).unwrap_or(COLOR_DEFAULT_GRAY);

    let input = text_input("New event...", &text)
        .on_input(Message::QuickEventTextChanged)
        .on_submit(Message::CommitQuickEvent)
        .size(14)
        .padding([6, 10])
        .width(Length::Fill);

    // The input spans across the specified number of columns
    // We use Length::Fill and let the parent container handle the width
    container(input)
        .width(Length::Fill)
        .padding([4, 6])
        .style(move |_theme: &cosmic::Theme| {
            container::Style {
                background: Some(cosmic::iced::Background::Color(color.scale_alpha(0.3))),
                border: cosmic::iced::Border {
                    color,
                    width: 2.0,
                    radius: crate::ui_constants::BORDER_RADIUS.into(),
                },
                ..Default::default()
            }
        })
        .into()
}

/// Render a column of events for a day cell (month view)
/// All-day events are shown first as colored bars,
/// then timed events sorted chronologically with colored dots.
pub fn render_events_column(
    events: Vec<DisplayEvent>,
    max_visible: usize,
) -> Element<'static, Message> {
    // Separate all-day and timed events
    let (mut all_day_events, mut timed_events): (Vec<_>, Vec<_>) =
        events.into_iter().partition(|e| e.all_day);

    // Sort timed events by start time
    timed_events.sort_by(|a, b| a.start_time.cmp(&b.start_time));

    // Combine: all-day first, then timed
    let sorted_events: Vec<DisplayEvent> = all_day_events
        .drain(..)
        .chain(timed_events.drain(..))
        .collect();

    let mut col = column().spacing(SPACING_TINY);
    let total = sorted_events.len();

    for (i, event) in sorted_events.into_iter().enumerate() {
        if i >= max_visible {
            // Show "+N more" indicator
            let remaining = total - max_visible;
            col = col.push(
                widget::text(format!("+{} more", remaining))
                    .size(10)
            );
            break;
        }
        col = col.push(render_event_chip(event));
    }

    col.into()
}

/// Result of splitting events into all-day and timed categories
pub struct SplitEventsResult {
    /// All-day events column (edge-to-edge, no margin)
    pub all_day: Option<Element<'static, Message>>,
    /// Timed events column (with spacing/margin)
    pub timed: Option<Element<'static, Message>>,
    /// Number of events not shown (for "+N more" indicator)
    pub overflow_count: usize,
}

/// Render events split into two separate containers:
/// - All-day events: edge-to-edge colored bars (no margin)
/// - Timed events: dot + time format with margin/spacing
///
/// This allows different visual treatment for each event type in day cells.
pub fn render_split_events(
    events: Vec<DisplayEvent>,
    max_visible: usize,
) -> SplitEventsResult {
    // Separate all-day and timed events
    let (all_day_events, mut timed_events): (Vec<_>, Vec<_>) =
        events.into_iter().partition(|e| e.all_day);

    // Sort timed events by start time
    timed_events.sort_by(|a, b| a.start_time.cmp(&b.start_time));

    let total_events = all_day_events.len() + timed_events.len();
    let mut shown = 0;

    // Render all-day events (no spacing between them for seamless look)
    let all_day = if !all_day_events.is_empty() {
        let mut col = column().spacing(SPACING_TINY);
        for event in all_day_events {
            if shown >= max_visible {
                break;
            }
            col = col.push(render_event_chip(event));
            shown += 1;
        }
        Some(col.into())
    } else {
        None
    };

    // Render timed events
    let timed = if !timed_events.is_empty() && shown < max_visible {
        let mut col = column().spacing(SPACING_TINY);
        for event in timed_events {
            if shown >= max_visible {
                break;
            }
            col = col.push(render_event_chip(event));
            shown += 1;
        }
        Some(col.into())
    } else {
        None
    };

    let overflow_count = if total_events > max_visible {
        total_events - max_visible
    } else {
        0
    };

    SplitEventsResult {
        all_day,
        timed,
        overflow_count,
    }
}
