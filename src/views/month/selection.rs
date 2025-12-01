//! Month view selection overlay rendering
//!
//! Contains the spanning quick event overlay rendering.

use chrono::NaiveDate;
use cosmic::iced::Length;
use cosmic::widget::{column, container, row};
use cosmic::Element;

use crate::components::render_spanning_quick_event_input;
use crate::components::spacer::{fill_spacer, horizontal_spacer, spacer, vertical_spacer};
use crate::message::Message;
use crate::models::CalendarDay;
use crate::ui_constants::{PADDING_MONTH_GRID, SPACING_TINY, WEEK_NUMBER_WIDTH};

use super::overlay::WEEKDAY_HEADER_HEIGHT;

/// Height of the spanning quick event input overlay
const SPANNING_INPUT_HEIGHT: f32 = 36.0;

/// Render the spanning quick event overlay positioned over the selected date range
pub fn render_spanning_overlay<'a>(
    weeks: &[Vec<CalendarDay>],
    start_date: NaiveDate,
    end_date: NaiveDate,
    text: String,
    calendar_color: String,
    show_week_numbers: bool,
) -> Element<'a, Message> {
    // Find which week(s) the selection spans
    let mut overlay_rows: Vec<(usize, usize, usize)> = Vec::new(); // (week_index, start_col, end_col)

    for (week_idx, week) in weeks.iter().enumerate() {
        let mut week_start_col: Option<usize> = None;
        let mut week_end_col: Option<usize> = None;

        for (day_idx, calendar_day) in week.iter().enumerate() {
            if let Some(cell_date) = NaiveDate::from_ymd_opt(
                calendar_day.year,
                calendar_day.month,
                calendar_day.day,
            ) {
                if cell_date >= start_date && cell_date <= end_date {
                    if week_start_col.is_none() {
                        week_start_col = Some(day_idx);
                    }
                    week_end_col = Some(day_idx);
                }
            }
        }

        if let (Some(start_col), Some(end_col)) = (week_start_col, week_end_col) {
            overlay_rows.push((week_idx, start_col, end_col));
        }
    }

    // Build the overlay structure matching the grid layout
    let mut overlay_column = column()
        .spacing(SPACING_TINY)
        .padding(PADDING_MONTH_GRID);

    // Add header spacer (same height as weekday header)
    overlay_column = overlay_column.push(vertical_spacer(WEEKDAY_HEADER_HEIGHT));

    let num_weeks = weeks.len();

    for week_idx in 0..num_weeks {
        // Check if this week has part of the selection
        let week_overlay = overlay_rows.iter().find(|(idx, _, _)| *idx == week_idx);

        if let Some((_, start_col, end_col)) = week_overlay {
            // This week has the selection - render the spanning input
            let mut week_row = row().spacing(SPACING_TINY).height(Length::Fill);

            // Week number spacer (if enabled)
            if show_week_numbers {
                week_row = week_row.push(horizontal_spacer(WEEK_NUMBER_WIDTH));
            }

            // Add empty spacers for columns before the selection
            for _ in 0..*start_col {
                week_row = week_row.push(spacer(Length::Fill, Length::Shrink));
            }

            // Calculate span (number of columns the input covers)
            let span_columns = end_col - start_col + 1;

            // Create a container that spans the selected columns
            // We use FillPortion to make it take the right amount of space
            let input = render_spanning_quick_event_input(
                text.clone(),
                calendar_color.clone(),
                span_columns,
            );

            // Wrap in a container with the correct proportion
            let spanning_container = container(input)
                .width(Length::FillPortion(span_columns as u16))
                .height(Length::Fixed(SPANNING_INPUT_HEIGHT))
                .center_y(Length::Fixed(SPANNING_INPUT_HEIGHT));

            week_row = week_row.push(spanning_container);

            // Add empty spacers for columns after the selection
            for _ in (*end_col + 1)..7 {
                week_row = week_row.push(spacer(Length::Fill, Length::Shrink));
            }

            overlay_column = overlay_column.push(week_row);
        } else {
            // Empty row - just a spacer with the same height
            overlay_column = overlay_column.push(fill_spacer());
        }
    }

    container(overlay_column)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}
