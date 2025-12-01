//! Timed event chip rendering
//!
//! Renders timed events with colored dot + time + name.

use chrono::NaiveTime;
use cosmic::iced::Length;
use cosmic::iced::widget::text::Wrapping;
use cosmic::widget::{container, row};
use cosmic::{widget, Element};

use crate::message::Message;
use crate::ui_constants::{SPACING_XXS, BORDER_RADIUS, BORDER_WIDTH_HIGHLIGHT};

use super::types::{ChipOpacity, ChipSelectionState};

/// Size of the colored dot for timed events
const TIMED_EVENT_DOT_SIZE: f32 = 8.0;

/// Render a timed event with colored dot + time + name
///
/// # Arguments
/// * `summary` - Event title to display
/// * `start_time` - Optional start time to display before the title
/// * `color` - Event calendar color
/// * `selection` - Optional selection state for interactive chips; None for simple display
pub fn render_timed_event_chip(
    summary: String,
    start_time: Option<NaiveTime>,
    color: cosmic::iced::Color,
    selection: Option<ChipSelectionState>,
) -> Element<'static, Message> {
    // Calculate opacity based on selection state
    let is_being_dragged = selection.map_or(false, |s| s.is_being_dragged);
    let is_selected = selection.map_or(false, |s| s.is_selected);
    let dot_opacity = ChipOpacity::dot_opacity(is_being_dragged);

    // Colored dot
    let dot = container(widget::text(""))
        .width(Length::Fixed(TIMED_EVENT_DOT_SIZE))
        .height(Length::Fixed(TIMED_EVENT_DOT_SIZE))
        .style(move |_theme: &cosmic::Theme| {
            container::Style {
                background: Some(cosmic::iced::Background::Color(color.scale_alpha(dot_opacity))),
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
    .style(move |_theme: &cosmic::Theme| {
        if is_being_dragged {
            // Dimmed style when being dragged
            container::Style {
                text_color: Some(cosmic::iced::Color::from_rgba(0.5, 0.5, 0.5, 0.5)),
                ..Default::default()
            }
        } else if is_selected {
            container::Style {
                background: Some(cosmic::iced::Background::Color(color.scale_alpha(0.15))),
                border: cosmic::iced::Border {
                    color,
                    width: BORDER_WIDTH_HIGHLIGHT,
                    radius: BORDER_RADIUS.into(),
                },
                ..Default::default()
            }
        } else {
            container::Style::default()
        }
    })
    .into()
}
