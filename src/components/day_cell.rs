use cosmic::iced::{alignment, Background, Border, Color, Length};
use cosmic::widget::{container, mouse_area};
use cosmic::{widget, Element};

use crate::message::Message;

pub fn render_day_cell(
    day: u32,
    is_today: bool,
    is_selected: bool,
) -> Element<'static, Message> {
    let day_cell = if is_today {
        // Today: outlined with accent color border (not filled)
        container(
            container(widget::text::title4(day.to_string()))
                .padding([4, 8, 0, 0]) // Top-right padding
                .width(Length::Fill)
                .align_x(alignment::Horizontal::Right),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(4)
        .style(|theme: &cosmic::Theme| container::Style {
            background: None,
            border: Border {
                color: theme.cosmic().accent_color().into(),
                width: 2.0,
                radius: [4.0, 4.0, 4.0, 4.0].into(), // Force 4px radius
            },
            ..Default::default()
        })
    } else if is_selected {
        // Selected: filled with accent color
        container(
            container(widget::text::title4(day.to_string()))
                .padding([4, 8, 0, 0]) // Top-right padding
                .width(Length::Fill)
                .align_x(alignment::Horizontal::Right),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(4)
        .style(|theme: &cosmic::Theme| container::Style {
            background: Some(Background::Color(theme.cosmic().accent_color().into())),
            border: Border {
                radius: [4.0, 4.0, 4.0, 4.0].into(), // Force 4px radius
                ..Default::default()
            },
            ..Default::default()
        })
    } else {
        // Normal day - light border
        container(
            container(widget::text(day.to_string()))
                .padding([4, 8, 0, 0]) // Top-right padding
                .width(Length::Fill)
                .align_x(alignment::Horizontal::Right),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(4)
        .style(|_theme: &cosmic::Theme| container::Style {
            background: None,
            border: Border {
                color: Color::from_rgba(0.5, 0.5, 0.5, 0.2).into(), // Light gray border
                width: 1.0,
                radius: [4.0, 4.0, 4.0, 4.0].into(), // Force 4px radius
            },
            ..Default::default()
        })
    };

    // Wrap in mouse_area for click handling - no theme button styling
    mouse_area(day_cell)
        .on_press(Message::SelectDay(day))
        .into()
}
