//! Current time indicator rendering for the week view
//!
//! Renders the red line and dot showing the current time.

use cosmic::iced::{alignment, Background, Border, Length};
use cosmic::widget::{column, container, row};
use cosmic::{widget, Element};

use crate::components::spacer::vertical_spacer;
use crate::message::Message;
use crate::ui_constants::{HOUR_ROW_HEIGHT, COLOR_CURRENT_TIME, BORDER_RADIUS};

/// Render the current time indicator as a separate overlay layer
/// This is rendered on top of events so the red line is always visible
pub fn render_time_indicator_layer(
    current_hour: u32,
    minute_offset: f32,
    show_dot: bool,
) -> Element<'static, Message> {
    // Total height of the grid (24 hours)
    let total_height = 24.0 * HOUR_ROW_HEIGHT;

    // Calculate position from start of day
    let hour_offset = current_hour as f32 * HOUR_ROW_HEIGHT;
    let total_offset = hour_offset + minute_offset;

    // Indicator dimensions
    let dot_size = 8.0_f32;
    let line_height = 2.0_f32;

    // Position the indicator with a top spacer (centered on the line)
    let adjusted_offset = (total_offset - (dot_size / 2.0)).max(0.0);
    let top_spacer = vertical_spacer(adjusted_offset);

    // Time indicator - line in all columns, dot only on today's column
    let time_indicator: Element<'static, Message> = if show_dot {
        // Today's column: dot on left, line filling the rest
        let dot = container(widget::text(""))
            .width(Length::Fixed(dot_size))
            .height(Length::Fixed(dot_size))
            .style(|_theme: &cosmic::Theme| container::Style {
                background: Some(Background::Color(COLOR_CURRENT_TIME)),
                border: Border {
                    radius: BORDER_RADIUS.into(),
                    ..Default::default()
                },
                ..Default::default()
            });

        let time_line = container(
            container(widget::text(""))
                .width(Length::Fill)
                .height(Length::Fixed(line_height))
                .style(|_theme: &cosmic::Theme| container::Style {
                    background: Some(Background::Color(COLOR_CURRENT_TIME)),
                    ..Default::default()
                })
        )
        .width(Length::Fill)
        .height(Length::Fixed(dot_size))
        .align_y(alignment::Vertical::Center);

        row()
            .spacing(0)
            .align_y(alignment::Vertical::Center)
            .push(dot)
            .push(time_line)
            .into()
    } else {
        // Other columns: just the line
        container(
            container(widget::text(""))
                .width(Length::Fill)
                .height(Length::Fixed(line_height))
                .style(|_theme: &cosmic::Theme| container::Style {
                    background: Some(Background::Color(COLOR_CURRENT_TIME)),
                    ..Default::default()
                })
        )
        .width(Length::Fill)
        .height(Length::Fixed(dot_size))
        .align_y(alignment::Vertical::Center)
        .into()
    };

    // Fill remaining space after indicator
    let remaining_height = (total_height - adjusted_offset - dot_size).max(0.0);
    let bottom_spacer = vertical_spacer(remaining_height);

    column()
        .spacing(0)
        .push(top_spacer)
        .push(time_indicator)
        .push(bottom_spacer)
        .width(Length::Fill)
        .into()
}
