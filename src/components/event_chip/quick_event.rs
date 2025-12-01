//! Quick event input rendering
//!
//! Inline text input for creating new events quickly.

use cosmic::iced::Length;
use cosmic::iced_widget::text_input;
use cosmic::widget::container;
use cosmic::Element;

use crate::components::color_picker::parse_hex_color;
use crate::message::Message;
use crate::ui_constants::{BORDER_RADIUS, COLOR_DEFAULT_GRAY};

/// ID for the quick event text input - used for auto-focus
pub fn quick_event_input_id() -> text_input::Id {
    text_input::Id::new("quick_event_input")
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
/// * `_span_columns` - Number of day columns to span (1-7) - reserved for future use
pub fn render_spanning_quick_event_input(
    text: String,
    calendar_color: String,
    _span_columns: usize, // Reserved for future layout adjustments
) -> Element<'static, Message> {
    let color = parse_hex_color(&calendar_color).unwrap_or(COLOR_DEFAULT_GRAY);

    let input = text_input("New event...", &text)
        .id(quick_event_input_id())
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
                    radius: BORDER_RADIUS.into(),
                },
                ..Default::default()
            }
        })
        .into()
}
