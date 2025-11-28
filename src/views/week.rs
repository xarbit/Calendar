use cosmic::iced::Length;
use cosmic::widget::{column, container};
use cosmic::{widget, Element};

use crate::message::Message;

pub fn render_week_view() -> Element<'static, Message> {
    let content = column()
        .spacing(20)
        .padding(40)
        .push(widget::text::title2("Week View"))
        .push(widget::text("Week view coming soon..."));

    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into()
}
