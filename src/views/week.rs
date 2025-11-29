use chrono::Datelike;
use cosmic::iced::{alignment, Border, Length};
use cosmic::widget::{column, container, row, scrollable};
use cosmic::{widget, Element};

use crate::components::{render_time_grid, render_time_column_placeholder, grid_cell_style, DayColumn};
use crate::locale::LocalePreferences;
use crate::localized_names;
use crate::message::Message;
use crate::models::WeekState;
use crate::ui_constants::{
    SPACING_TINY, PADDING_SMALL,
    FONT_SIZE_SMALL, FONT_SIZE_MEDIUM, BORDER_RADIUS,
    ALL_DAY_HEADER_HEIGHT
};

pub fn render_week_view(week_state: &WeekState, locale: &LocalePreferences) -> Element<'static, Message> {
    let all_day_section = render_all_day_section(week_state, locale);

    // Build day columns for time grid
    let day_columns: Vec<DayColumn> = week_state.days.iter()
        .map(|date| DayColumn::new(locale.is_weekend(date.weekday())))
        .collect();
    let time_grid = render_time_grid(locale, &day_columns);

    let content = column()
        .spacing(0)
        .push(all_day_section)
        .push(scrollable(time_grid));

    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

/// Render the all-day events section at the top
fn render_all_day_section(week_state: &WeekState, locale: &LocalePreferences) -> Element<'static, Message> {
    let mut header_row = row().spacing(0);

    // Time column placeholder
    header_row = header_row.push(render_time_column_placeholder(ALL_DAY_HEADER_HEIGHT));

    // Day headers
    for date in week_state.days.clone() {
        let is_today = week_state.is_today(&date);
        let is_weekend = locale.is_weekend(date.weekday());
        let day_name = localized_names::get_weekday_short(date.weekday());
        let day_number = format!("{}", date.format("%d"));

        let day_number_container = if is_today {
            container(
                widget::text(day_number).size(FONT_SIZE_MEDIUM)
            )
            .padding(PADDING_SMALL)
            .style(|theme: &cosmic::Theme| {
                container::Style {
                    background: Some(cosmic::iced::Background::Color(
                        theme.cosmic().accent_color().into()
                    )),
                    border: Border {
                        radius: BORDER_RADIUS.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                }
            })
        } else {
            container(widget::text(day_number).size(FONT_SIZE_MEDIUM))
                .padding(PADDING_SMALL)
        };

        let day_header = column()
            .spacing(SPACING_TINY)
            .align_x(alignment::Horizontal::Center)
            .push(widget::text(day_name).size(FONT_SIZE_SMALL))
            .push(day_number_container);

        header_row = header_row.push(
            container(day_header)
                .width(Length::Fill)
                .height(Length::Fixed(ALL_DAY_HEADER_HEIGHT))
                .padding(PADDING_SMALL)
                .style(move |_theme: &cosmic::Theme| grid_cell_style(is_weekend))
        );
    }

    header_row.into()
}
