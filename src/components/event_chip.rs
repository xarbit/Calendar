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
