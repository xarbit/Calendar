use cosmic::iced::Length;
use cosmic::widget::{button, column, container, dialog, radio, text, text_input};
use cosmic::{widget, Element};

use crate::app::CosmicCalendar;
use crate::fl;
use crate::message::Message;

pub fn view_subscribe_dialog<'a>(
    app: &'a CosmicCalendar,
    url: &'a str,
    calendar_name: &'a str,
    events: &'a [crate::caldav::CalendarEvent],
    selected_calendar_id: &'a Option<String>,
    create_new_calendar: bool,
    new_calendar_name: &'a str,
) -> Element<'a, Message> {
    let event_count = events.len();

    // URL info
    let url_section = column()
        .spacing(4)
        .push(text(fl!("subscribe-dialog-url")).size(14))
        .push(text(url).size(12));

    // Calendar info
    let calendar_section = column()
        .spacing(4)
        .push(text(fl!("subscribe-dialog-calendar-name")).size(14))
        .push(text(calendar_name).size(12));

    // Event count
    let event_info = column().spacing(8).push(
        text(fl!(
            "subscribe-dialog-event-count",
            count = (event_count as i64)
        ))
        .size(14),
    );

    // Calendar selection
    let mut calendar_control = column()
        .spacing(8)
        .push(text(fl!("subscribe-dialog-select-calendar")).size(14));

    // Radio button for existing calendar
    calendar_control = calendar_control.push(radio(
        "Use existing calendar",
        false,
        Some(create_new_calendar),
        |_| Message::ToggleCreateNewCalendar(false),
    ));

    // List of existing calendars (only show if not creating new)
    if !create_new_calendar {
        for calendar in app.calendar_manager.sources() {
            let info = calendar.info();
            let calendar_radio = radio(
                info.name.as_str(),
                info.id.as_str(),
                selected_calendar_id.as_deref(),
                |id| Message::SelectSubscriptionCalendar(id.to_string()),
            );
            calendar_control =
                calendar_control.push(container(calendar_radio).padding([0, 0, 0, 16]));
        }
    }

    // Radio button for new calendar
    calendar_control = calendar_control.push(radio(
        "Create new calendar",
        true,
        Some(create_new_calendar),
        |_| Message::ToggleCreateNewCalendar(true),
    ));

    // Text input for new calendar name (only show if creating new)
    if create_new_calendar {
        calendar_control = calendar_control.push(
            container(
                text_input("New calendar name", new_calendar_name)
                    .on_input(Message::UpdateSubscriptionCalendarName),
            )
            .padding([0, 0, 0, 16]),
        );
    }

    // Determine if subscribe button should be enabled
    let can_subscribe = if create_new_calendar {
        !new_calendar_name.trim().is_empty()
    } else {
        selected_calendar_id.is_some()
    };

    let primary_btn = if can_subscribe {
        button::suggested(fl!("subscribe-dialog-subscribe")).on_press(Message::ConfirmSubscription)
    } else {
        button::suggested(fl!("subscribe-dialog-subscribe"))
    };

    // Use COSMIC's dialog widget
    dialog()
        .title(fl!("subscribe-dialog-title"))
        .icon(widget::icon::from_name("emblem-web-symbolic").size(64))
        .control(url_section)
        .control(calendar_section)
        .control(event_info)
        .control(calendar_control)
        .secondary_action(button::text(fl!("button-cancel")).on_press(Message::CancelSubscription))
        .primary_action(primary_btn)
        .width(Length::Fixed(500.0))
        .into()
}
