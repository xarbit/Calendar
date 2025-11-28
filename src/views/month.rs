use chrono::Datelike;
use cosmic::iced::Length;
use cosmic::widget::{column, container, mouse_area, row};
use cosmic::{widget, Element};

use crate::components::render_day_cell;
use crate::message::Message;

pub fn render_month_view(
    current_year: i32,
    current_month: u32,
    selected_day: Option<u32>,
) -> Element<'static, Message> {
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

    let mut grid = column().spacing(1).padding(20);

    // Weekday headers
    let header_row = row()
        .spacing(1)
        .push(
            container(widget::text("Monday").size(12))
                .width(Length::Fill)
                .padding(8)
                .center_x(Length::Fill),
        )
        .push(
            container(widget::text("Tuesday").size(12))
                .width(Length::Fill)
                .padding(8)
                .center_x(Length::Fill),
        )
        .push(
            container(widget::text("Wednesday").size(12))
                .width(Length::Fill)
                .padding(8)
                .center_x(Length::Fill),
        )
        .push(
            container(widget::text("Thursday").size(12))
                .width(Length::Fill)
                .padding(8)
                .center_x(Length::Fill),
        )
        .push(
            container(widget::text("Friday").size(12))
                .width(Length::Fill)
                .padding(8)
                .center_x(Length::Fill),
        )
        .push(
            container(widget::text("Saturday").size(12))
                .width(Length::Fill)
                .padding(8)
                .center_x(Length::Fill),
        )
        .push(
            container(widget::text("Sunday").size(12))
                .width(Length::Fill)
                .padding(8)
                .center_x(Length::Fill),
        );

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

    // Render weeks with cells
    for week in weeks {
        let mut week_row = row().spacing(1).height(Length::Fill);
        for day_opt in week {
            let cell = if let Some(day) = day_opt {
                let is_today = is_current_month && today.day() == day;
                let is_selected = selected_day == Some(day);

                render_day_cell(day, is_today, is_selected)
            } else {
                mouse_area(container(widget::text("")).padding(8)).into()
            };

            week_row = week_row.push(container(cell).width(Length::Fill).height(Length::Fill));
        }
        grid = grid.push(week_row);
    }

    container(grid)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}
