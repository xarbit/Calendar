//! All-day event chip rendering
//!
//! Renders all-day events as colored background bars.

use cosmic::iced::Length;
use cosmic::iced::widget::text::Wrapping;
use cosmic::widget::container;
use cosmic::{widget, Element};

use crate::message::Message;
use crate::ui_constants::{BORDER_RADIUS, BORDER_WIDTH_HIGHLIGHT};

use super::types::{ChipOpacity, ChipSelectionState, SpanPosition, span_border_radius, span_padding};

/// Render an all-day event chip with colored background bar.
///
/// # Arguments
/// * `summary` - Event title to display
/// * `color` - Event calendar color
/// * `span_position` - Position within a multi-day span (affects border radius)
/// * `selection` - Optional selection state for interactive chips; None for simple display
pub fn render_all_day_chip(
    summary: String,
    color: cosmic::iced::Color,
    span_position: SpanPosition,
    selection: Option<ChipSelectionState>,
) -> Element<'static, Message> {
    let border_radius = span_border_radius(span_position, BORDER_RADIUS[0]);
    let padding = span_padding(span_position);

    let content: Element<'static, Message> = widget::text(summary)
        .size(11)
        .wrapping(Wrapping::None)
        .into();

    // Calculate opacity based on selection state
    let opacity = selection.map_or(
        ChipOpacity { background: 0.3, text: 1.0 },
        |s| ChipOpacity::from_state(s.is_selected, s.is_being_dragged)
    );
    let is_selected = selection.map_or(false, |s| s.is_selected);

    container(content)
        .padding(padding)
        .width(Length::Fill)
        .clip(true)
        .style(move |_theme: &cosmic::Theme| {
            container::Style {
                background: Some(cosmic::iced::Background::Color(
                    color.scale_alpha(opacity.background)
                )),
                border: cosmic::iced::Border {
                    color: if is_selected { color } else { cosmic::iced::Color::TRANSPARENT },
                    width: if is_selected { BORDER_WIDTH_HIGHLIGHT } else { 0.0 },
                    radius: border_radius.into(),
                },
                text_color: Some(color.scale_alpha(opacity.text)),
                ..Default::default()
            }
        })
        .into()
}
