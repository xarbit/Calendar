//! Time grid rendering for the week view
//!
//! Contains the time labels column and hour cell grid background.

use chrono::{NaiveDate, NaiveTime};
use cosmic::iced::{alignment, Border, Length};
use cosmic::widget::{column, container, mouse_area};
use cosmic::{widget, Element};

use crate::locale::LocalePreferences;
use crate::message::Message;
use crate::selection::SelectionState;
use crate::styles::weekend_background;
use crate::ui_constants::{
    PADDING_SMALL, FONT_SIZE_SMALL, COLOR_DAY_CELL_BORDER,
    HOUR_ROW_HEIGHT, TIME_LABEL_WIDTH, BORDER_WIDTH_THIN, COLOR_CURRENT_TIME,
};

/// Render the time labels column (left side)
pub fn render_time_labels_column<'a>(
    locale: &'a LocalePreferences,
    today_in_view: bool,
    current_hour: u32,
) -> Element<'a, Message> {
    let mut col = column().spacing(0);

    for hour in 0..24 {
        let is_current_hour = today_in_view && hour == current_hour;
        let time_label = locale.format_hour(hour);

        col = col.push(
            container(
                widget::text(time_label)
                    .size(FONT_SIZE_SMALL)
            )
            .width(Length::Fixed(TIME_LABEL_WIDTH))
            .height(Length::Fixed(HOUR_ROW_HEIGHT))
            .padding(PADDING_SMALL)
            .align_y(alignment::Vertical::Top)
            .style(move |_theme: &cosmic::Theme| container::Style {
                text_color: if is_current_hour {
                    Some(COLOR_CURRENT_TIME)
                } else {
                    None
                },
                border: Border {
                    width: BORDER_WIDTH_THIN,
                    color: COLOR_DAY_CELL_BORDER,
                    ..Default::default()
                },
                ..Default::default()
            })
        );
    }

    col.into()
}

/// Render the hour grid background (lines only, no events or time indicator) with clickable time slots
pub fn render_hour_grid_background(
    date: NaiveDate,
    is_weekend: bool,
    selection: Option<&SelectionState>,
) -> Element<'static, Message> {
    let mut hour_cells = column().spacing(0);

    for hour in 0..24u32 {
        // Check if this hour cell is within the current selection
        let is_selected = selection.map(|s| s.is_active && s.contains_time(date, hour)).unwrap_or(false);
        let cell = render_clickable_hour_cell(date, hour, is_weekend, is_selected);
        hour_cells = hour_cells.push(cell);
    }

    hour_cells.into()
}

/// Render a clickable hour cell (for creating new events and drag targets)
fn render_clickable_hour_cell(date: NaiveDate, hour: u32, is_weekend: bool, is_selected: bool) -> Element<'static, Message> {
    // Create the time for this hour cell
    let start_time = NaiveTime::from_hms_opt(hour, 0, 0).unwrap();
    let _end_time = NaiveTime::from_hms_opt(hour, 59, 59).unwrap_or_else(|| {
        NaiveTime::from_hms_opt(23, 59, 59).unwrap()
    });

    let cell = container(widget::text(""))
        .width(Length::Fill)
        .height(Length::Fixed(HOUR_ROW_HEIGHT))
        .style(move |theme: &cosmic::Theme| {
            let background = if is_selected {
                // Use theme accent color for selection (consistent with month view)
                let accent = theme.cosmic().accent_color();
                Some(cosmic::iced::Background::Color(cosmic::iced::Color::from_rgba(
                    accent.red, accent.green, accent.blue, 0.2
                )))
            } else {
                weekend_background(is_weekend)
            };
            container::Style {
                background,
                border: Border {
                    width: BORDER_WIDTH_THIN,
                    color: COLOR_DAY_CELL_BORDER,
                    ..Default::default()
                },
                ..Default::default()
            }
        });

    // Press: start time selection for creating timed events
    // Release: end time selection
    // on_enter: update time selection (for drag selection)
    // Double-click: open new event dialog
    mouse_area(cell)
        .on_press(Message::TimeSelectionStart(date, start_time))
        .on_release(Message::TimeSelectionEnd)
        .on_double_click(Message::OpenNewEventDialog)
        .on_enter(Message::TimeSelectionUpdate(date, start_time))
        .into()
}
