use cosmic::iced::Length;
use cosmic::widget::{column, container, row, scrollable};
use cosmic::Element;

use crate::components::{render_time_grid, render_time_column_placeholder, DayColumn, render_day_header, DayHeaderConfig};
use crate::styles::bordered_cell_style;
use crate::locale::LocalePreferences;
use crate::message::Message;
use crate::models::DayState;
use crate::ui_constants::{PADDING_SMALL, ALL_DAY_HEADER_HEIGHT};

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

    // Time column placeholder
    header_row = header_row.push(render_time_column_placeholder(ALL_DAY_HEADER_HEIGHT));

    // Create day header with larger size for single day view
    let day_header = render_day_header(DayHeaderConfig::day_view(
        day_state.day_text.clone(),
        day_state.date_number.clone(),
        day_state.is_today(),
    ));

    header_row = header_row.push(
        container(day_header)
            .width(Length::Fill)
            .height(Length::Fixed(ALL_DAY_HEADER_HEIGHT))
            .padding(PADDING_SMALL)
            .style(|_theme: &cosmic::Theme| bordered_cell_style())
    );

    header_row.into()
}

