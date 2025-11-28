use chrono::Datelike;
use cosmic::iced::Length;
use cosmic::widget::{button, column, container, row};
use cosmic::{widget, Element};

use crate::message::Message;

pub fn render_mini_calendar(
    current_year: i32,
    current_month: u32,
    selected_day: Option<u32>,
) -> Element<'static, Message> {
    let date =
        chrono::NaiveDate::from_ymd_opt(current_year, current_month, 1).unwrap();
    let month_year = format!("{}", date.format("%B %Y"));

    let header = row()
        .spacing(8)
        .push(
            button::icon(widget::icon::from_name("go-previous-symbolic"))
                .on_press(Message::MiniCalendarPrevMonth)
                .padding(4),
        )
        .push(container(widget::text::body(month_year).size(14)).width(Length::Fill))
        .push(
            button::icon(widget::icon::from_name("go-next-symbolic"))
                .on_press(Message::MiniCalendarNextMonth)
                .padding(4),
        );

    let first_day =
        chrono::NaiveDate::from_ymd_opt(current_year, current_month, 1).unwrap();
    let first_weekday = first_day.weekday().num_days_from_monday();

    let days_in_month = if current_month == 12 {
        chrono::NaiveDate::from_ymd_opt(current_year + 1, 1, 1)
            .unwrap()
            .signed_duration_since(first_day)
            .num_days()
    } else {
        chrono::NaiveDate::from_ymd_opt(current_year, current_month + 1, 1)
            .unwrap()
            .signed_duration_since(first_day)
            .num_days()
    };

    let mut grid = column().spacing(4);

    // Weekday headers (abbreviated)
    let header_row = row()
        .spacing(2)
        .push(widget::text("M").width(Length::Fill).size(11))
        .push(widget::text("T").width(Length::Fill).size(11))
        .push(widget::text("W").width(Length::Fill).size(11))
        .push(widget::text("T").width(Length::Fill).size(11))
        .push(widget::text("F").width(Length::Fill).size(11))
        .push(widget::text("S").width(Length::Fill).size(11))
        .push(widget::text("S").width(Length::Fill).size(11));

    grid = grid.push(header_row);

    // Calendar days
    let mut weeks = vec![];
    let mut current_week = vec![];

    for _ in 0..first_weekday {
        current_week.push(None);
    }

    for day in 1..=days_in_month {
        current_week.push(Some(day as u32));
        if current_week.len() == 7 {
            weeks.push(current_week.clone());
            current_week.clear();
        }
    }

    if !current_week.is_empty() {
        while current_week.len() < 7 {
            current_week.push(None);
        }
        weeks.push(current_week);
    }

    let today = chrono::Local::now();
    let is_current_month = today.year() == current_year && today.month() == current_month;

    for week in weeks {
        let mut week_row = row().spacing(2);
        for day_opt in week {
            if let Some(day) = day_opt {
                let is_today = is_current_month && today.day() == day;
                let is_selected = selected_day == Some(day);

                let day_button = if is_today {
                    widget::button::suggested(day.to_string())
                        .on_press(Message::SelectDay(day))
                        .padding(4)
                        .width(Length::Fixed(32.0))
                } else if is_selected {
                    widget::button::standard(day.to_string())
                        .on_press(Message::SelectDay(day))
                        .padding(4)
                        .width(Length::Fixed(32.0))
                } else {
                    widget::button::text(day.to_string())
                        .on_press(Message::SelectDay(day))
                        .padding(4)
                        .width(Length::Fixed(32.0))
                };
                week_row = week_row.push(day_button);
            } else {
                week_row = week_row.push(container(widget::text("")).width(Length::Fixed(32.0)));
            }
        }
        grid = grid.push(week_row);
    }

    column().spacing(12).push(header).push(grid).into()
}
