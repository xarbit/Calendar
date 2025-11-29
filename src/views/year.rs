use cosmic::iced::{alignment, Border, Length, Size};
use cosmic::widget::{column, container, row, scrollable, responsive};
use cosmic::{widget, Element};

use crate::locale::LocalePreferences;
use crate::localized_names;
use crate::message::Message;
use crate::models::YearState;
use crate::ui_constants::{
    FONT_SIZE_SMALL, PADDING_SMALL, PADDING_MEDIUM, PADDING_TINY,
    SPACING_MEDIUM, SPACING_SMALL, SPACING_XXS, COLOR_DAY_CELL_BORDER, BORDER_WIDTH_THIN
};

// Fixed size for square month boxes - ensures all 6 weeks + header + month name are visible
const MONTH_BOX_SIZE: f32 = 220.0;

pub fn render_year_view(year_state: &YearState, _locale: &LocalePreferences) -> Element<'static, Message> {
    // Clone data needed for the closure
    let months = year_state.months.clone();
    let today = year_state.today;
    let year = year_state.year;

    responsive(move |size: Size| {
        let num_columns = calculate_columns(size.width);
        render_year_grid(&months, today, year, num_columns)
    })
    .into()
}

/// Calculate optimal number of columns based on available width
fn calculate_columns(available_width: f32) -> usize {
    let spacing = SPACING_MEDIUM as f32;
    let padding = PADDING_MEDIUM as f32 * 2.0;

    let usable_width = available_width - padding + spacing;
    let column_width = MONTH_BOX_SIZE + spacing;
    let columns = (usable_width / column_width).floor() as usize;

    columns.clamp(1, 4)
}

/// Render the year grid with specified number of columns
fn render_year_grid(
    months: &[crate::models::CalendarState],
    today: (i32, u32, u32),
    year: i32,
    num_columns: usize,
) -> Element<'static, Message> {
    let mut year_layout = column()
        .spacing(SPACING_MEDIUM)
        .padding(PADDING_MEDIUM);

    // Create rows based on the number of columns
    let mut month_index = 0;
    while month_index < 12 {
        let mut month_row = row().spacing(SPACING_MEDIUM);

        for _ in 0..num_columns {
            if month_index < 12 {
                let month_calendar = render_mini_month(
                    &months[month_index],
                    today,
                    year,
                    month_index + 1,
                );
                month_row = month_row.push(month_calendar);
                month_index += 1;
            }
        }

        year_layout = year_layout.push(month_row);
    }

    // Center the grid and allow vertical scrolling
    scrollable(
        container(year_layout)
            .width(Length::Fill)
            .center_x(Length::Fill)
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

/// Render a single mini month calendar for the year view
fn render_mini_month(
    month_state: &crate::models::CalendarState,
    today: (i32, u32, u32),
    year: i32,
    month: usize,
) -> Element<'static, Message> {
    let mut mini_calendar = column()
        .spacing(SPACING_SMALL)
        .padding(PADDING_SMALL)
        .width(Length::Fixed(MONTH_BOX_SIZE))
        .height(Length::Fixed(MONTH_BOX_SIZE));

    // Month name header
    let month_name = localized_names::get_month_name(month as u32);
    mini_calendar = mini_calendar.push(
        container(widget::text::title4(month_name))
            .width(Length::Fill)
            .center_x(Length::Fill)
            .padding([0, 0, PADDING_SMALL, 0])
    );

    // Weekday headers (abbreviated, single letter for space)
    let weekday_names = localized_names::get_weekday_names_short();
    let mut header_row = row().spacing(SPACING_XXS);
    for weekday in &weekday_names {
        let first_char = weekday.chars().next().unwrap_or(' ').to_string();
        header_row = header_row.push(
            container(widget::text(first_char).size(FONT_SIZE_SMALL))
                .width(Length::Fill)
                .center_x(Length::Fill)
        );
    }
    mini_calendar = mini_calendar.push(header_row);

    // Day grid
    for week in &month_state.weeks {
        let mut week_row = row().spacing(SPACING_XXS);
        for day_opt in week {
            if let Some(day) = day_opt {
                let is_today = today == (year, month as u32, *day);

                let day_container = if is_today {
                    container(widget::text(format!("{}", day)).size(FONT_SIZE_SMALL))
                        .width(Length::Fill)
                        .padding(PADDING_TINY)
                        .center_x(Length::Fill)
                        .align_y(alignment::Vertical::Center)
                        .style(|theme: &cosmic::Theme| {
                            container::Style {
                                text_color: Some(theme.cosmic().accent_color().into()),
                                background: Some(cosmic::iced::Background::Color(
                                    theme.cosmic().accent_color().into()
                                )),
                                border: Border {
                                    radius: 4.0.into(),
                                    ..Default::default()
                                },
                                ..Default::default()
                            }
                        })
                } else {
                    container(widget::text(format!("{}", day)).size(FONT_SIZE_SMALL))
                        .width(Length::Fill)
                        .padding(PADDING_TINY)
                        .center_x(Length::Fill)
                        .align_y(alignment::Vertical::Center)
                };

                week_row = week_row.push(day_container);
            } else {
                week_row = week_row.push(
                    container(widget::text(""))
                        .width(Length::Fill)
                        .padding(PADDING_TINY)
                );
            }
        }
        mini_calendar = mini_calendar.push(week_row);
    }

    container(mini_calendar)
        .width(Length::Fixed(MONTH_BOX_SIZE))
        .height(Length::Fixed(MONTH_BOX_SIZE))
        .style(|_theme: &cosmic::Theme| {
            container::Style {
                border: Border {
                    width: BORDER_WIDTH_THIN,
                    color: COLOR_DAY_CELL_BORDER,
                    radius: 8.0.into(),
                },
                ..Default::default()
            }
        })
        .into()
}
