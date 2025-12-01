//! Quick event input rendering for the week view
//!
//! Renders the inline event creation input that appears when selecting a time slot.

use chrono::{NaiveTime, Timelike};
use cosmic::iced::{Background, Border, Length};
use cosmic::iced_widget::text_input;
use cosmic::widget::{column, container};
use cosmic::Element;

use crate::components::{parse_color_safe, quick_event_input_id};
use crate::components::spacer::vertical_spacer;
use crate::message::Message;
use crate::ui_constants::{HOUR_ROW_HEIGHT, BORDER_RADIUS};

/// Render the quick event input overlay layer for timed event creation
/// Positions the input at the correct time slot and spans the selected duration
pub fn render_quick_event_input_layer(
    start_time: NaiveTime,
    end_time: NaiveTime,
    text: String,
    calendar_color: String,
) -> Element<'static, Message> {
    // Calculate position and height based on time range
    let start_mins = start_time.hour() * 60 + start_time.minute();
    let end_mins = end_time.hour() * 60 + end_time.minute();

    // Ensure minimum duration and proper order
    let (start_mins, end_mins) = if start_mins <= end_mins {
        (start_mins, end_mins.max(start_mins + 30)) // Minimum 30 min
    } else {
        (end_mins, start_mins.max(end_mins + 30))
    };

    let top_offset = (start_mins as f32 / 60.0) * HOUR_ROW_HEIGHT;
    let height = ((end_mins - start_mins) as f32 / 60.0) * HOUR_ROW_HEIGHT;
    let height = height.max(HOUR_ROW_HEIGHT); // Minimum height of 1 hour cell

    let color = parse_color_safe(&calendar_color);

    // Create text input for the event title
    let input = text_input("New event...", &text)
        .id(quick_event_input_id())
        .on_input(Message::QuickEventTextChanged)
        .on_submit(Message::CommitQuickEvent)
        .size(12)
        .padding([4, 6])
        .width(Length::Fill);

    // Style the container with calendar color
    let input_container = container(input)
        .width(Length::Fill)
        .height(Length::Fixed(height))
        .padding([2, 4])
        .style(move |_theme: &cosmic::Theme| container::Style {
            background: Some(Background::Color(cosmic::iced::Color {
                a: 0.3,
                ..color
            })),
            border: Border {
                color,
                width: 2.0,
                radius: BORDER_RADIUS.into(),
            },
            ..Default::default()
        });

    // Create spacer to position the input at the correct time
    let top_spacer = vertical_spacer(top_offset);

    // Build column with spacer and input
    column()
        .spacing(0)
        .push(top_spacer)
        .push(input_container)
        .width(Length::Fill)
        .into()
}
