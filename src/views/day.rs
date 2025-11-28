use cosmic::iced::Length;
use cosmic::widget::{column, container};
use cosmic::{widget, Element};

use crate::message::Message;

pub fn render_day_view() -> Element<'static, Message> {
    let content = column()
        .spacing(20)
        .padding(40)
        .push(widget::text::title2("Day View"))
        .push(widget::text("Day view coming soon..."));

    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into()
}
