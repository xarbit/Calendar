use cosmic::iced::{alignment, Background, Border, Length};
use cosmic::widget::{container, mouse_area};
use cosmic::{widget, Element};

use crate::message::Message;
use crate::ui_constants::{BORDER_RADIUS, PADDING_DAY_CELL, COLOR_DAY_CELL_BORDER, COLOR_WEEKEND_BACKGROUND, BORDER_WIDTH_HIGHLIGHT, BORDER_WIDTH_NORMAL};

pub fn render_day_cell(
    day: u32,
    is_today: bool,
    is_selected: bool,
    is_weekend: bool,
) -> Element<'static, Message> {
    // Create single mouse_area with styled container - reduces widget count
    let day_text = if is_today || is_selected {
        widget::text::title4(day.to_string())
    } else {
        widget::text(day.to_string())
    };

    // Build styled container based on state
    let styled_container = if is_today {
        // Today: outlined with accent color border (not filled)
        container(day_text)
            .padding(PADDING_DAY_CELL)
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(alignment::Horizontal::Right)
            .style(|theme: &cosmic::Theme| container::Style {
                background: None,
                border: Border {
                    color: theme.cosmic().accent_color().into(),
                    width: BORDER_WIDTH_HIGHLIGHT,
                    radius: BORDER_RADIUS.into(),
                },
                ..Default::default()
            })
    } else if is_selected {
        // Selected: filled with accent color
        container(day_text)
            .padding(PADDING_DAY_CELL)
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(alignment::Horizontal::Right)
            .style(|theme: &cosmic::Theme| container::Style {
                background: Some(Background::Color(theme.cosmic().accent_color().into())),
                border: Border {
                    radius: BORDER_RADIUS.into(),
                    ..Default::default()
                },
                ..Default::default()
            })
    } else {
        // Normal day - light border with optional weekend background
        container(day_text)
            .padding(PADDING_DAY_CELL)
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(alignment::Horizontal::Right)
            .style(move |_theme: &cosmic::Theme| container::Style {
                background: if is_weekend {
                    Some(Background::Color(COLOR_WEEKEND_BACKGROUND))
                } else {
                    None
                },
                border: Border {
                    color: COLOR_DAY_CELL_BORDER.into(),
                    width: BORDER_WIDTH_NORMAL,
                    radius: BORDER_RADIUS.into(),
                },
                ..Default::default()
            })
    };

    // Single mouse_area wrapping the styled container
    mouse_area(styled_container)
        .on_press(Message::SelectDay(day))
        .into()
}
