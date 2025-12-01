//! Event rendering for the week view
//!
//! Contains timed event chip rendering and event overlay positioning.

use chrono::{NaiveDate, Timelike};
use cosmic::iced::{Background, Border, Length};
use cosmic::widget::{column, container, mouse_area, row};
use cosmic::{widget, Element};

use crate::components::{parse_color_safe, DisplayEvent};
use crate::components::spacer::vertical_spacer;
use crate::message::Message;
use crate::ui_constants::{HOUR_ROW_HEIGHT, BORDER_RADIUS};

use super::utils::{event_time_range, PositionedEvent};

/// Render the events overlay layer with events positioned based on their time spans
/// Uses a row of columns approach where each column renders its events independently
pub fn render_events_overlay_layer(
    date: NaiveDate,
    positioned_events: &[PositionedEvent],
    max_columns: usize,
    selected_event_uid: Option<&str>,
) -> Element<'static, Message> {
    // Each column renders its events independently with proper vertical positioning
    // This ensures overlapping events appear side-by-side
    let mut columns_row = row().spacing(1);

    for col_idx in 0..max_columns {
        // Get all events for this column, sorted by start time
        let mut col_events: Vec<&PositionedEvent> = positioned_events.iter()
            .filter(|pe| pe.column == col_idx)
            .collect();
        col_events.sort_by_key(|pe| event_time_range(&pe.event).0);

        // Build this column's content with spacers and events
        let col_content = render_column_events(date, &col_events, selected_event_uid);

        columns_row = columns_row.push(
            container(col_content)
                .width(Length::Fill)
        );
    }

    columns_row.into()
}

/// Render a single column of events with proper vertical spacing
fn render_column_events(
    date: NaiveDate,
    events: &[&PositionedEvent],
    selected_event_uid: Option<&str>,
) -> Element<'static, Message> {
    let mut col = column().spacing(0);
    let mut current_mins: u32 = 0;

    for pe in events {
        let (start_mins, end_mins) = event_time_range(&pe.event);

        // Add spacer to position this event correctly
        if start_mins > current_mins {
            let spacer_height = ((start_mins - current_mins) as f32 / 60.0) * HOUR_ROW_HEIGHT;
            col = col.push(vertical_spacer(spacer_height));
        }

        // Render the event
        let ev_height = ((end_mins - start_mins) as f32 / 60.0) * HOUR_ROW_HEIGHT;
        let event_block = render_positioned_event_block(
            date,
            &pe.event,
            ev_height.max(20.0), // Minimum height for visibility
            selected_event_uid,
        );
        col = col.push(event_block);

        current_mins = end_mins;
    }

    // Fill remaining space to maintain column height
    let total_mins = 24 * 60;
    if current_mins < total_mins {
        let remaining_height = ((total_mins - current_mins) as f32 / 60.0) * HOUR_ROW_HEIGHT;
        col = col.push(vertical_spacer(remaining_height));
    }

    col.into()
}

/// Render a positioned event block with the specified height
fn render_positioned_event_block(
    date: NaiveDate,
    event: &DisplayEvent,
    height: f32,
    selected_event_uid: Option<&str>,
) -> Element<'static, Message> {
    let color = parse_color_safe(&event.color);
    let uid = event.uid.clone();
    let is_selected = selected_event_uid == Some(&event.uid);

    let (bg_opacity, border_width) = if is_selected {
        (0.9, 2.0)
    } else {
        (0.85, 0.0)
    };

    // Build the label with time and summary
    let time_str = event.start_time
        .map(|t| format!("{:02}:{:02}", t.hour(), t.minute()))
        .unwrap_or_default();
    let label = format!("{} {}", time_str, event.summary);

    let chip = container(
        widget::text(label.clone())
            .size(10)
    )
    .padding([2, 4])
    .width(Length::Fill)
    .height(Length::Fixed(height))
    .style(move |theme: &cosmic::Theme| container::Style {
        background: Some(Background::Color(cosmic::iced::Color {
            a: bg_opacity,
            ..color
        })),
        text_color: Some(cosmic::iced::Color::WHITE),
        border: Border {
            radius: BORDER_RADIUS.into(),
            width: border_width,
            color: if is_selected {
                theme.cosmic().accent_color().into()
            } else {
                cosmic::iced::Color::TRANSPARENT
            },
        },
        ..Default::default()
    });

    // Get color hex for drag preview
    let color_hex = event.color.clone();

    mouse_area(chip)
        .on_press(Message::DragEventStart(uid.clone(), date, event.summary.clone(), color_hex))
        .on_release(Message::DragEventEnd)
        .on_double_click(Message::OpenEditEventDialog(uid))
        .on_enter(Message::DragEventUpdate(date))
        .into()
}
