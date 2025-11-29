//! Event management handlers (quick events and event dialog)

use chrono::{NaiveDate, NaiveTime, TimeZone, Utc};
use cosmic::widget::text_editor;
use uuid::Uuid;

use crate::app::{CosmicCalendar, EventDialogState};
use crate::caldav::{AlertTime, CalendarEvent, RepeatFrequency, TravelTime};

/// Commit the quick event being edited - create a new event in the selected calendar
pub fn handle_commit_quick_event(app: &mut CosmicCalendar) {
    // Get the event data and clear the editing state
    let Some((date, text)) = app.quick_event_editing.take() else {
        return;
    };

    // Don't create empty events
    let text = text.trim();
    if text.is_empty() {
        return;
    }

    // Get the selected calendar ID
    let Some(calendar_id) = app.selected_calendar_id.clone() else {
        eprintln!("No calendar selected for new event");
        return;
    };

    // Create an all-day event for the selected date
    // Use midnight UTC for start, end of day for end
    let start_time = NaiveTime::from_hms_opt(0, 0, 0).unwrap();
    let end_time = NaiveTime::from_hms_opt(23, 59, 59).unwrap();

    let start = Utc.from_utc_datetime(&date.and_time(start_time));
    let end = Utc.from_utc_datetime(&date.and_time(end_time));

    let event = CalendarEvent {
        uid: Uuid::new_v4().to_string(),
        summary: text.to_string(),
        location: None,
        all_day: true,
        start,
        end,
        travel_time: TravelTime::None,
        repeat: RepeatFrequency::Never,
        invitees: vec![],
        alert: AlertTime::None,
        alert_second: None,
        attachments: vec![],
        url: None,
        notes: None,
    };

    // Find the calendar and add the event
    if let Some(calendar) = app
        .calendar_manager
        .sources_mut()
        .iter_mut()
        .find(|c| c.info().id == calendar_id)
    {
        if let Err(e) = calendar.add_event(event) {
            eprintln!("Failed to add event: {}", e);
            return;
        }
        // Sync to persist the event
        if let Err(e) = calendar.sync() {
            eprintln!("Failed to sync calendar: {}", e);
        }
    }

    // Refresh the cached events to show the new event
    app.refresh_cached_events();
}

/// Delete an event by its UID from all calendars
pub fn handle_delete_event(app: &mut CosmicCalendar, uid: String) {
    for calendar in app.calendar_manager.sources_mut().iter_mut() {
        if calendar.delete_event(&uid).is_ok() {
            // Sync to persist the deletion
            let _ = calendar.sync();
            break;
        }
    }
    // Refresh cached events to reflect deletion
    app.refresh_cached_events();
}

/// Start editing a quick event on a specific date
pub fn handle_start_quick_event(app: &mut CosmicCalendar, date: NaiveDate) {
    app.quick_event_editing = Some((date, String::new()));
}

/// Update the quick event text while editing
pub fn handle_quick_event_text_changed(app: &mut CosmicCalendar, text: String) {
    if let Some((date, _)) = app.quick_event_editing.take() {
        app.quick_event_editing = Some((date, text));
    }
}

/// Cancel quick event editing
pub fn handle_cancel_quick_event(app: &mut CosmicCalendar) {
    app.quick_event_editing = None;
}

// === Event Dialog Handlers ===

/// Open the event dialog for creating a new event
pub fn handle_open_new_event_dialog(app: &mut CosmicCalendar) {
    let today = app.selected_date;

    // Default to 9:00 AM - 10:00 AM
    let default_start_time = NaiveTime::from_hms_opt(9, 0, 0);
    let default_end_time = NaiveTime::from_hms_opt(10, 0, 0);

    // Use selected calendar or first available
    let calendar_id = app
        .selected_calendar_id
        .clone()
        .or_else(|| {
            app.calendar_manager
                .sources()
                .first()
                .map(|c| c.info().id.clone())
        })
        .unwrap_or_default();

    app.event_dialog = Some(EventDialogState {
        editing_uid: None,
        title: String::new(),
        location: String::new(),
        all_day: false,
        start_date: today,
        start_date_input: today.format("%Y-%m-%d").to_string(),
        start_time: default_start_time,
        start_time_input: default_start_time
            .map(|t| t.format("%H:%M").to_string())
            .unwrap_or_else(|| "09:00".to_string()),
        end_date: today,
        end_date_input: today.format("%Y-%m-%d").to_string(),
        end_time: default_end_time,
        end_time_input: default_end_time
            .map(|t| t.format("%H:%M").to_string())
            .unwrap_or_else(|| "10:00".to_string()),
        travel_time: TravelTime::None,
        repeat: RepeatFrequency::Never,
        calendar_id,
        invitees: vec![],
        invitee_input: String::new(),
        alert: AlertTime::None,
        alert_second: None,
        attachments: vec![],
        url: String::new(),
        notes_content: text_editor::Content::new(),
        editing_field: None,
    });
}

/// Open the event dialog for editing an existing event
pub fn handle_open_edit_event_dialog(app: &mut CosmicCalendar, uid: String) {
    // Find the event in all calendars
    let mut found_event: Option<(CalendarEvent, String)> = None;

    for calendar in app.calendar_manager.sources() {
        if let Ok(events) = calendar.fetch_events() {
            if let Some(event) = events.iter().find(|e| e.uid == uid) {
                found_event = Some((event.clone(), calendar.info().id.clone()));
                break;
            }
        }
    }

    let Some((event, calendar_id)) = found_event else {
        eprintln!("Event not found: {}", uid);
        return;
    };

    // Convert UTC times to local dates/times
    let start_date = event.start.date_naive();
    let end_date = event.end.date_naive();
    let start_time = Some(event.start.time());
    let end_time = Some(event.end.time());

    let actual_start_time = if event.all_day { None } else { start_time };
    let actual_end_time = if event.all_day { None } else { end_time };

    app.event_dialog = Some(EventDialogState {
        editing_uid: Some(uid),
        title: event.summary,
        location: event.location.unwrap_or_default(),
        all_day: event.all_day,
        start_date,
        start_date_input: start_date.format("%Y-%m-%d").to_string(),
        start_time: actual_start_time,
        start_time_input: actual_start_time
            .map(|t| t.format("%H:%M").to_string())
            .unwrap_or_else(|| "09:00".to_string()),
        end_date,
        end_date_input: end_date.format("%Y-%m-%d").to_string(),
        end_time: actual_end_time,
        end_time_input: actual_end_time
            .map(|t| t.format("%H:%M").to_string())
            .unwrap_or_else(|| "10:00".to_string()),
        travel_time: event.travel_time,
        repeat: event.repeat,
        calendar_id,
        invitees: event.invitees,
        invitee_input: String::new(),
        alert: event.alert,
        alert_second: event.alert_second,
        attachments: event.attachments,
        url: event.url.unwrap_or_default(),
        notes_content: text_editor::Content::with_text(&event.notes.unwrap_or_default()),
        editing_field: None,
    });
}

/// Confirm the event dialog - create or update the event
pub fn handle_confirm_event_dialog(app: &mut CosmicCalendar) {
    let Some(dialog) = app.event_dialog.take() else {
        return;
    };

    // Validate: title is required
    let title = dialog.title.trim();
    if title.is_empty() {
        // Put dialog back - can't save without title
        app.event_dialog = Some(dialog);
        return;
    }

    // Build start and end times
    let start_time = if dialog.all_day {
        NaiveTime::from_hms_opt(0, 0, 0).unwrap()
    } else {
        dialog.start_time.unwrap_or_else(|| NaiveTime::from_hms_opt(9, 0, 0).unwrap())
    };

    let end_time = if dialog.all_day {
        NaiveTime::from_hms_opt(23, 59, 59).unwrap()
    } else {
        dialog.end_time.unwrap_or_else(|| NaiveTime::from_hms_opt(10, 0, 0).unwrap())
    };

    let start = Utc.from_utc_datetime(&dialog.start_date.and_time(start_time));
    let end = Utc.from_utc_datetime(&dialog.end_date.and_time(end_time));

    let event = CalendarEvent {
        uid: dialog.editing_uid.clone().unwrap_or_else(|| Uuid::new_v4().to_string()),
        summary: title.to_string(),
        location: if dialog.location.is_empty() {
            None
        } else {
            Some(dialog.location)
        },
        all_day: dialog.all_day,
        start,
        end,
        travel_time: dialog.travel_time,
        repeat: dialog.repeat,
        invitees: dialog.invitees,
        alert: dialog.alert,
        alert_second: dialog.alert_second,
        attachments: dialog.attachments,
        url: if dialog.url.is_empty() {
            None
        } else {
            Some(dialog.url)
        },
        notes: {
            let notes_text = dialog.notes_content.text();
            if notes_text.trim().is_empty() {
                None
            } else {
                Some(notes_text)
            }
        },
    };

    // If editing, delete the old event first (it might be in a different calendar)
    if let Some(ref old_uid) = dialog.editing_uid {
        for calendar in app.calendar_manager.sources_mut().iter_mut() {
            if calendar.delete_event(old_uid).is_ok() {
                let _ = calendar.sync();
                break;
            }
        }
    }

    // Add to the selected calendar
    if let Some(calendar) = app
        .calendar_manager
        .sources_mut()
        .iter_mut()
        .find(|c| c.info().id == dialog.calendar_id)
    {
        if let Err(e) = calendar.add_event(event) {
            eprintln!("Failed to save event: {}", e);
            return;
        }
        if let Err(e) = calendar.sync() {
            eprintln!("Failed to sync calendar: {}", e);
        }
    }

    // Refresh cached events
    app.refresh_cached_events();
}

/// Cancel the event dialog
pub fn handle_cancel_event_dialog(app: &mut CosmicCalendar) {
    app.event_dialog = None;
}
