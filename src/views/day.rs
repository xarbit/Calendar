use cosmic::iced::{alignment, Border, Length};
use cosmic::widget::{column, container, row, scrollable};
use cosmic::{widget, Element};

use crate::components::{render_time_grid, render_time_column_placeholder, bordered_cell_style, DayColumn};
use crate::locale::LocalePreferences;
use crate::message::Message;
use crate::models::DayState;
use crate::ui_constants::{
    SPACING_TINY, PADDING_SMALL, PADDING_MEDIUM,
    FONT_SIZE_MEDIUM, FONT_SIZE_LARGE, BORDER_RADIUS,
    ALL_DAY_HEADER_HEIGHT
};

pub fn render_day_view(day_state: &DayState, locale: &LocalePreferences) -> Element<'static, Message> {
    let all_day_section = render_all_day_section(day_state);

    // Single day column for day view (never weekend-styled in day view)
    let day_columns = vec![DayColumn::regular()];
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
fn render_all_day_section(day_state: &DayState) -> Element<'static, Message> {
    let mut header_row = row().spacing(0);

    // Clone strings to own them for 'static lifetime
    let is_today = day_state.is_today();
    let day_text = day_state.day_text.clone();
    let date_number = day_state.date_number.clone();

    // Time column placeholder
    header_row = header_row.push(render_time_column_placeholder(ALL_DAY_HEADER_HEIGHT));

    // Create day header with larger size for single day view
    let day_number_container = if is_today {
        container(
            widget::text(date_number.clone()).size(FONT_SIZE_LARGE)
        )
        .padding(PADDING_MEDIUM)
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
        container(widget::text(date_number.clone()).size(FONT_SIZE_LARGE))
            .padding(PADDING_MEDIUM)
    };

    let day_header = column()
        .spacing(SPACING_TINY)
        .align_x(alignment::Horizontal::Center)
        .push(widget::text(day_text).size(FONT_SIZE_MEDIUM))
        .push(day_number_container);

    header_row = header_row.push(
        container(day_header)
            .width(Length::Fill)
            .height(Length::Fixed(ALL_DAY_HEADER_HEIGHT))
            .padding(PADDING_SMALL)
            .style(|_theme: &cosmic::Theme| bordered_cell_style())
    );

    header_row.into()
}

