use cosmic::widget::{column, container, row};
use cosmic::{widget, Element};

use crate::calendars::CalendarSource;
use crate::components::{render_color_indicator, render_quick_color_picker};
use crate::fl;
use crate::message::Message;
use crate::ui_constants::{SPACING_MEDIUM, PADDING_MEDIUM, FONT_SIZE_BODY, PADDING_COLOR_PICKER_NESTED, COLOR_INDICATOR_SIZE};

/// Render the list of calendars with checkboxes and color pickers
pub fn render_calendar_list<'a>(
    calendars: &'a [Box<dyn CalendarSource>],
    color_picker_open: Option<&String>,
) -> Element<'a, Message> {
    let mut calendar_list = column()
        .spacing(SPACING_MEDIUM)
        .padding(PADDING_MEDIUM)
        .push(widget::text::body(fl!("sidebar-calendars")).size(FONT_SIZE_BODY));

    for calendar in calendars {
        let info = calendar.info();
        let is_enabled = calendar.is_enabled();
        let is_picker_open = color_picker_open.map(|id| id == &info.id).unwrap_or(false);

        // Use the color picker component for the indicator
        let color_indicator = render_color_indicator(
            info.id.clone(),
            &info.color,
            COLOR_INDICATOR_SIZE,
        );

        let calendar_row = row()
            .spacing(SPACING_MEDIUM)
            .push(widget::checkbox("", is_enabled).on_toggle(move |_| {
                Message::ToggleCalendar(info.id.clone())
            }))
            .push(color_indicator)
            .push(widget::text(&info.name));

        calendar_list = calendar_list.push(calendar_row);

        // Show inline color picker if this calendar's picker is open
        if is_picker_open {
            let color_picker = render_quick_color_picker(info.id.clone(), &info.color);

            calendar_list = calendar_list.push(
                container(color_picker)
                    .padding(PADDING_COLOR_PICKER_NESTED)
            );
        }
    }

    calendar_list.into()
}
