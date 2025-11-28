use cosmic::iced::Length;
use cosmic::widget::{column, container, divider, row, scrollable};
use cosmic::{widget, Element};

use crate::components::render_mini_calendar;
use crate::message::Message;

pub fn render_sidebar(
    current_year: i32,
    current_month: u32,
    selected_day: Option<u32>,
) -> Element<'static, Message> {
    let mini_calendar = render_mini_calendar(current_year, current_month, selected_day);

    let calendars_section = column()
        .spacing(8)
        .padding(12)
        .push(widget::text::body("Calendars").size(14))
        .push(
            row()
                .spacing(8)
                .push(widget::checkbox("", true))
                .push(widget::text("Personal")),
        )
        .push(
            row()
                .spacing(8)
                .push(widget::checkbox("", true))
                .push(widget::text("Work")),
        );

    let sidebar_content = column()
        .spacing(20)
        .padding(16)
        .push(mini_calendar)
        .push(divider::horizontal::default())
        .push(calendars_section);

    container(scrollable(sidebar_content))
        .width(Length::Fixed(280.0))
        .height(Length::Fill)
        .into()
}
