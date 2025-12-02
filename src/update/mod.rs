//! Message handling and state updates
//!
//! This module contains the main message handler and delegates to specialized
//! submodules for different categories of messages:
//!
//! - `navigation`: View navigation (previous/next period, view changes)
//! - `calendar`: Calendar management (create, edit, delete, toggle, color)
//! - `event`: Event management (quick events, create, delete)
//! - `selection`: Drag selection for multi-day event creation
//!
//! ## Helper Functions
//!
//! Common operations are extracted into helper functions to reduce code duplication:
//!
//! - [`dismiss_on_focus_loss`]: Clear transient UI state when user navigates away
//! - [`focus_quick_event_input`]: Focus the quick event text input
//! - [`scroll_week_to_current_time`]: Scroll week view to show current time
//! - [`scroll_week_to_hour`]: Scroll week view to a specific hour
//! - [`close_legacy_event_dialog`]: Close deprecated event dialog field
//! - [`schedule_deferred_scroll_restore`]: Schedule scroll position restoration
//! - [`close_quick_event_with_scroll_restore`]: Close quick event and restore scroll

mod calendar;
mod event;
mod import;
mod navigation;
mod selection;

use chrono::{NaiveDate, Timelike};
use cosmic::app::Task;
use cosmic::iced::widget::scrollable;
use log::{debug, error, info, warn};

use crate::app::CosmicCalendar;
use crate::components::quick_event_input_id;
use crate::dialogs::{ActiveDialog, DialogManager};
use crate::message::Message;
use crate::services::{ExportHandler, SettingsHandler};
use crate::views::{week_time_grid_id, CalendarView};
use crate::ui_constants::HOUR_ROW_HEIGHT;
use cosmic::iced_widget::text_input;

/// Helper to dismiss empty quick events on focus-loss actions (navigation, day selection)
/// This centralizes the pattern of clearing transient UI state when the user navigates away
#[inline]
fn dismiss_on_focus_loss(app: &mut CosmicCalendar) {
    DialogManager::dismiss_empty_quick_event(&mut app.active_dialog);
}

/// Focus the quick event input field
/// Returns a Task that focuses the text input for immediate typing
#[inline]
fn focus_quick_event_input() -> Task<Message> {
    text_input::focus(quick_event_input_id())
}

/// Scroll the week view time grid to the current time
/// Returns a Task that scrolls to show the current hour (offset by 1-2 hours to show some past)
#[inline]
fn scroll_week_to_current_time() -> Task<Message> {
    let now = chrono::Local::now();
    let current_hour = now.hour();

    // Scroll to show current time with some context (show 1-2 hours before current time)
    // Each hour row is HOUR_ROW_HEIGHT pixels tall
    let scroll_to_hour = current_hour.saturating_sub(1) as f32;
    let scroll_offset = scroll_to_hour * HOUR_ROW_HEIGHT;

    // Use scroll_to with AbsoluteOffset for vertical scrolling
    scrollable::scroll_to(
        week_time_grid_id(),
        scrollable::AbsoluteOffset { x: 0.0, y: scroll_offset },
    )
}

/// Scroll the week view time grid to a specific hour
/// Used to keep the view stable when focusing quick event input
#[inline]
fn scroll_week_to_hour(hour: u32) -> Task<Message> {
    // Show the hour with 1 hour of context above
    let scroll_to_hour = hour.saturating_sub(1) as f32;
    let scroll_offset = scroll_to_hour * HOUR_ROW_HEIGHT;

    scrollable::scroll_to(
        week_time_grid_id(),
        scrollable::AbsoluteOffset { x: 0.0, y: scroll_offset },
    )
}

/// Close the legacy event dialog field
/// This helper is kept because text_editor::Content doesn't implement Clone
#[allow(deprecated)]
#[inline]
fn close_legacy_event_dialog(app: &mut CosmicCalendar) {
    app.event_dialog = None;
}

/// Schedule a deferred scroll restore if there's a saved restore position
/// Uses two-field pattern: scroll_opt tracks current, scroll_restore holds the pre-dialog position
#[inline]
fn schedule_deferred_scroll_restore(app: &CosmicCalendar) -> Task<Message> {
    if app.week_view_scroll_restore.is_some() {
        // Schedule restore for next update cycle
        Task::done(cosmic::Action::App(Message::RestoreWeekViewScroll))
    } else {
        Task::none()
    }
}

/// Close quick event dialog if active and schedule scroll restore
/// Returns Task::none() if no quick event was active, otherwise schedules restore
#[inline]
fn close_quick_event_with_scroll_restore(app: &mut CosmicCalendar) -> Task<Message> {
    if app.active_dialog.is_quick_event() {
        DialogManager::close(&mut app.active_dialog);
        schedule_deferred_scroll_restore(app)
    } else {
        DialogManager::close(&mut app.active_dialog);
        Task::none()
    }
}

/// Handle exporting a calendar to an iCalendar file
fn handle_export_calendar_to_file(
    app: &mut CosmicCalendar,
    calendar_id: String,
    path: std::path::PathBuf,
) -> Task<Message> {
    debug!("handle_export_calendar_to_file: Exporting calendar '{}' to {:?}", calendar_id, path);

    match ExportHandler::export_to_file(&app.calendar_manager, &calendar_id, &path) {
        Ok(()) => {
            info!("Exported calendar '{}' to {:?}", calendar_id, path);
            // TODO: Show success toast notification
        }
        Err(e) => {
            error!("Failed to export calendar '{}': {}", calendar_id, e);
            // TODO: Show error toast notification
        }
    }

    Task::none()
}

/// Handle URL processing (webcal://, ics://, calendar://)
fn handle_process_url(app: &mut CosmicCalendar, url: String) -> Task<Message> {
    use crate::url_handler::{parse_url, UrlAction};

    info!("handle_process_url: Processing URL: {}", url);

    match parse_url(&url) {
        Ok(action) => {
            match action {
                UrlAction::ImportRemote { url: https_url } => {
                    info!("URL Action: ImportRemote from {}", https_url);
                    // Download calendar asynchronously
                    return Task::perform(
                        async move {
                            use crate::url_handler::download_calendar;
                            match download_calendar(&https_url).await {
                                Ok(calendar_data) => Some((https_url, calendar_data)),
                                Err(e) => {
                                    error!("Failed to download calendar: {}", e);
                                    None
                                }
                            }
                        },
                        |result| {
                            if let Some((url, calendar_data)) = result {
                                cosmic::Action::App(Message::ProcessDownloadedCalendar(url, calendar_data))
                            } else {
                                cosmic::Action::App(Message::None)
                            }
                        },
                    );
                }
                UrlAction::OpenView { view } => {
                    info!("URL Action: OpenView to {}", view);
                    // Change view based on URL
                    use crate::views::CalendarView;
                    let new_view = match view.as_str() {
                        "month" => CalendarView::Month,
                        "week" => CalendarView::Week,
                        "day" => CalendarView::Day,
                        "year" => CalendarView::Year,
                        _ => {
                            warn!("Unknown view: {}, defaulting to Month", view);
                            CalendarView::Month
                        }
                    };
                    app.current_view = new_view;
                }
                UrlAction::CreateEvent { summary, start, end, location } => {
                    info!("URL Action: CreateEvent - summary={:?}, start={:?}, end={:?}, location={:?}",
                          summary, start, end, location);
                    // TODO: Open event dialog with pre-filled data
                    info!("Would open event dialog with pre-filled data");
                }
                UrlAction::ViewEvent { uid } => {
                    info!("URL Action: ViewEvent uid={}", uid);
                    // TODO: Find and select the event
                    info!("Would select and show event: {}", uid);
                }
            }
        }
        Err(e) => {
            error!("Failed to parse URL '{}': {}", url, e);
            // TODO: Show error dialog
        }
    }

    Task::none()
}

fn handle_process_downloaded_calendar(
    app: &mut CosmicCalendar,
    url: String,
    calendar_data: String,
) -> Task<Message> {
    use crate::services::ExportHandler;

    info!("Parsing downloaded calendar data from {}", url);

    // Parse the iCalendar data
    match ExportHandler::parse_ical_string_with_name(&calendar_data) {
        Ok((calendar_name, events)) => {
            info!("Successfully parsed {} events from calendar '{}'", events.len(), calendar_name);

            // Show subscription dialog
            app.active_dialog = ActiveDialog::SubscribeCalendar {
                url: url.clone(),
                calendar_name: calendar_name.clone(),
                events: events.clone(),
                selected_calendar_id: None,
                create_new_calendar: true,  // Default to creating new calendar
                new_calendar_name: calendar_name.clone(),
            };

            Task::none()
        }
        Err(e) => {
            error!("Failed to parse calendar data: {}", e);
            // TODO: Show error dialog
            Task::none()
        }
    }
}

fn handle_show_subscribe_dialog(
    app: &mut CosmicCalendar,
    url: String,
    calendar_name: String,
    events: Vec<crate::caldav::CalendarEvent>,
) -> Task<Message> {
    app.active_dialog = ActiveDialog::SubscribeCalendar {
        url,
        calendar_name: calendar_name.clone(),
        events,
        selected_calendar_id: None,
        create_new_calendar: true,
        new_calendar_name: calendar_name,
    };
    Task::none()
}

fn handle_select_subscription_calendar(
    app: &mut CosmicCalendar,
    calendar_id: String,
) -> Task<Message> {
    if let ActiveDialog::SubscribeCalendar {
        selected_calendar_id,
        ..
    } = &mut app.active_dialog
    {
        *selected_calendar_id = Some(calendar_id);
    }
    Task::none()
}

fn handle_toggle_create_new_calendar(
    app: &mut CosmicCalendar,
    create_new: bool,
) -> Task<Message> {
    if let ActiveDialog::SubscribeCalendar {
        create_new_calendar,
        ..
    } = &mut app.active_dialog
    {
        *create_new_calendar = create_new;
    }
    Task::none()
}

fn handle_update_subscription_calendar_name(
    app: &mut CosmicCalendar,
    name: String,
) -> Task<Message> {
    if let ActiveDialog::SubscribeCalendar {
        new_calendar_name,
        ..
    } = &mut app.active_dialog
    {
        *new_calendar_name = name;
    }
    Task::none()
}

fn handle_confirm_subscription(app: &mut CosmicCalendar) -> Task<Message> {
    use crate::services::{CalendarHandler, NewCalendarData};

    if let ActiveDialog::SubscribeCalendar {
        url,
        calendar_name: _,
        events,
        selected_calendar_id,
        create_new_calendar,
        new_calendar_name,
    } = &app.active_dialog
    {
        info!("Confirming subscription from {}", url);

        let target_calendar_id = if *create_new_calendar {
            // Create new calendar
            let new_name = new_calendar_name.trim();
            if new_name.is_empty() {
                warn!("Cannot create calendar with empty name");
                return Task::none();
            }

            let new_calendar_data = NewCalendarData {
                name: new_name.to_string(),
                color: "#3584e4".to_string(), // Default blue color
            };

            match CalendarHandler::create(&mut app.calendar_manager, new_calendar_data) {
                Ok(calendar_id) => {
                    info!("Created new calendar '{}' with id={}", new_name, calendar_id);
                    calendar_id
                }
                Err(e) => {
                    error!("Failed to create calendar: {}", e);
                    return Task::none();
                }
            }
        } else {
            // Use existing calendar
            match selected_calendar_id {
                Some(id) => id.clone(),
                None => {
                    warn!("No calendar selected for subscription");
                    return Task::none();
                }
            }
        };

        // Transition to import dialog with the target calendar selected
        let events_to_import = events.clone();
        info!("Importing {} events into calendar {}", events_to_import.len(), target_calendar_id);

        // Set up import dialog with target calendar pre-selected
        app.active_dialog = ActiveDialog::Import {
            events: events_to_import,
            source_file_name: url.clone(),
            selected_calendar_id: Some(target_calendar_id),
        };

        // Immediately confirm the import
        return import::handle_confirm_import(app);
    }

    Task::none()
}

fn handle_cancel_subscription(app: &mut CosmicCalendar) -> Task<Message> {
    info!("Cancelled calendar subscription");
    app.active_dialog = ActiveDialog::None;
    Task::none()
}

// Re-export handlers for use in this module
use calendar::{
    handle_change_calendar_color, handle_confirm_calendar_dialog, handle_confirm_delete_calendar,
    handle_delete_selected_calendar, handle_export_calendar_dialog,
    handle_open_calendar_dialog_create, handle_open_calendar_dialog_edit,
    handle_request_delete_calendar, handle_toggle_calendar,
};
use event::{
    extract_master_uid, extract_occurrence_date, handle_cancel_event_dialog, handle_cancel_quick_event,
    handle_commit_quick_event, handle_confirm_event_dialog, handle_delete_event,
    handle_drag_event_cancel, handle_drag_event_end, handle_drag_event_start,
    handle_drag_event_update, handle_open_edit_event_dialog, handle_open_new_event_dialog,
    handle_quick_event_text_changed, handle_select_event, handle_start_quick_event,
    handle_start_quick_timed_event,
};
use navigation::{handle_next_period, handle_previous_period};
use selection::{
    handle_selection_cancel, handle_selection_end, handle_selection_start, handle_selection_update,
    handle_time_selection_start, handle_time_selection_update, handle_time_selection_end,
};

/// Handle all application messages and update state
pub fn handle_message(app: &mut CosmicCalendar, message: Message) -> Task<Message> {
    // Sync sidebar with condensed state on every update
    let is_condensed = app.core.is_condensed();
    if is_condensed != app.last_condensed {
        app.last_condensed = is_condensed;
        // Auto-collapse sidebar when entering condensed mode, show when leaving
        app.show_sidebar = !is_condensed;
    }

    match message {
        // === Dialog Management (Centralized) ===
        Message::Dialog(action) => {
            debug!("Message::Dialog: {:?}", action);
            // Handle dialog actions through DialogManager
            DialogManager::handle_action(&mut app.active_dialog, action);
        }
        Message::CloseDialog => {
            debug!("Message::CloseDialog: Closing dialogs");
            // Close legacy event dialog
            close_legacy_event_dialog(app);
            // For quick events: only dismiss if empty (focus loss behavior)
            // For other dialogs: close unconditionally
            let was_quick_event = app.active_dialog.is_quick_event();
            if was_quick_event {
                dismiss_on_focus_loss(app);
                // Schedule scroll restore when closing quick event via Escape
                return schedule_deferred_scroll_restore(app);
            } else {
                DialogManager::close(&mut app.active_dialog);
            }
        }

        // === View Navigation ===
        // All navigation actions dismiss empty quick events (focus loss behavior)
        Message::ChangeView(view) => {
            dismiss_on_focus_loss(app);
            app.current_view = view;
            app.sync_views_to_selected_date();
            // Auto-scroll to current time when entering week view
            if view == CalendarView::Week {
                return scroll_week_to_current_time();
            }
        }
        Message::CycleViewNext => {
            dismiss_on_focus_loss(app);
            let new_view = app.current_view.next();
            app.current_view = new_view;
            app.sync_views_to_selected_date();
            // Auto-scroll to current time when entering week view
            if new_view == CalendarView::Week {
                return scroll_week_to_current_time();
            }
        }
        Message::CycleViewPrevious => {
            dismiss_on_focus_loss(app);
            let new_view = app.current_view.previous();
            app.current_view = new_view;
            app.sync_views_to_selected_date();
            // Auto-scroll to current time when entering week view
            if new_view == CalendarView::Week {
                return scroll_week_to_current_time();
            }
        }
        Message::PreviousPeriod => {
            dismiss_on_focus_loss(app);
            handle_previous_period(app);
        }
        Message::NextPeriod => {
            dismiss_on_focus_loss(app);
            handle_next_period(app);
        }
        Message::Today => {
            dismiss_on_focus_loss(app);
            app.navigate_to_today();
        }
        Message::SelectDay(year, month, day) => {
            dismiss_on_focus_loss(app);
            if let Some(date) = NaiveDate::from_ymd_opt(year, month, day) {
                app.set_selected_date(date);
            }
        }
        Message::SelectDayNoNavigate(date) => {
            dismiss_on_focus_loss(app);
            app.selected_date = date;
        }

        // === UI State ===
        Message::TimeTick => {
            // Timer tick to update the current time indicator
            // The view will re-render with the new time automatically
        }
        Message::ToggleSidebar => {
            app.show_sidebar = !app.show_sidebar;
        }
        Message::WindowResized => {
            // Sync is handled at start of update(), nothing else needed
        }
        Message::ToggleSearch => {
            app.show_search = !app.show_search;
        }
        Message::ToggleWeekNumbers => {
            debug!("Message::ToggleWeekNumbers");
            if let Err(e) = SettingsHandler::toggle_week_numbers(&mut app.settings) {
                log::error!("Failed to toggle week numbers: {}", e);
            }
        }
        Message::WeekViewScroll(viewport) => {
            // Track scroll position via on_scroll callback (COSMIC Files pattern)
            // This stores the actual pixel offset so we can restore it precisely
            app.week_view_scroll_opt = Some(viewport.absolute_offset());
        }
        Message::RestoreWeekViewScroll => {
            // Restore scroll position from the saved restore point
            // This uses the position captured BEFORE quick event started (not the continuously tracked one)
            if let Some(offset) = app.week_view_scroll_restore.take() {
                return scrollable::scroll_to(week_time_grid_id(), offset);
            }
        }
        Message::ScrollTimelineUp => {
            // Only works in Day or Week view - scroll up by one hour
            if matches!(app.current_view, CalendarView::Day | CalendarView::Week) {
                if let Some(offset) = app.week_view_scroll_opt {
                    // Scroll up by one hour (decrease y offset, minimum 0)
                    let new_y = (offset.y - HOUR_ROW_HEIGHT).max(0.0);
                    let new_offset = scrollable::AbsoluteOffset { x: offset.x, y: new_y };
                    return scrollable::scroll_to(week_time_grid_id(), new_offset);
                }
            }
        }
        Message::ScrollTimelineDown => {
            // Only works in Day or Week view - scroll down by one hour
            if matches!(app.current_view, CalendarView::Day | CalendarView::Week) {
                if let Some(offset) = app.week_view_scroll_opt {
                    // Scroll down by one hour (increase y offset, max 24 hours)
                    let max_scroll = HOUR_ROW_HEIGHT * 23.0; // Can scroll to show hour 23
                    let new_y = (offset.y + HOUR_ROW_HEIGHT).min(max_scroll);
                    let new_offset = scrollable::AbsoluteOffset { x: offset.x, y: new_y };
                    return scrollable::scroll_to(week_time_grid_id(), new_offset);
                }
            }
        }

        // === Calendar Management ===
        Message::ToggleCalendar(id) => {
            // Close dialogs when interacting with other elements (with scroll restore if quick event)
            let task = close_quick_event_with_scroll_restore(app);
            handle_toggle_calendar(app, id);
            return task;
        }
        Message::SelectCalendar(id) => {
            // Close dialogs when selecting a different calendar (with scroll restore if quick event)
            let task = close_quick_event_with_scroll_restore(app);
            app.selected_calendar_id = Some(id);
            app.update_selected_calendar_color();
            return task;
        }
        Message::ToggleColorPicker(id) => {
            // Toggle: if already open for this calendar, close it; otherwise open it
            if app.active_dialog.color_picker_calendar_id() == Some(&id) {
                DialogManager::close(&mut app.active_dialog);
            } else {
                DialogManager::open(&mut app.active_dialog, ActiveDialog::ColorPicker { calendar_id: id });
            }
        }
        Message::CloseColorPicker => {
            DialogManager::close(&mut app.active_dialog);
        }
        Message::ChangeCalendarColor(id, color) => {
            handle_change_calendar_color(app, id, color);
        }
        Message::OpenNewCalendarDialog => {
            DialogManager::close(&mut app.active_dialog);
            handle_open_calendar_dialog_create(app);
        }
        Message::OpenEditCalendarDialog(id) => {
            DialogManager::close(&mut app.active_dialog);
            handle_open_calendar_dialog_edit(app, id);
        }
        Message::EditCalendarByIndex(index) => {
            DialogManager::close(&mut app.active_dialog);
            if let Some(calendar) = app.calendar_manager.sources().get(index) {
                let id = calendar.info().id.clone();
                handle_open_calendar_dialog_edit(app, id);
            }
        }
        Message::ExportCalendarByIndex(index) => {
            DialogManager::close(&mut app.active_dialog);
            if let Some(calendar) = app.calendar_manager.sources().get(index) {
                let calendar_id = calendar.info().id.clone();
                let calendar_name = calendar.info().name.clone();
                return handle_export_calendar_dialog(app, calendar_id, calendar_name);
            }
        }
        Message::CalendarDialogNameChanged(name) => {
            // Update calendar dialog name via active_dialog
            match &mut app.active_dialog {
                ActiveDialog::CalendarCreate { name: n, .. }
                | ActiveDialog::CalendarEdit { name: n, .. } => {
                    *n = name;
                }
                _ => {}
            }
        }
        Message::CalendarDialogColorChanged(color) => {
            // Update calendar dialog color via active_dialog
            match &mut app.active_dialog {
                ActiveDialog::CalendarCreate { color: c, .. }
                | ActiveDialog::CalendarEdit { color: c, .. } => {
                    *c = color;
                }
                _ => {}
            }
        }
        Message::ConfirmCalendarDialog => {
            handle_confirm_calendar_dialog(app);
        }
        Message::CancelCalendarDialog => {
            DialogManager::close(&mut app.active_dialog);
        }
        Message::DeleteSelectedCalendar => {
            DialogManager::close(&mut app.active_dialog);
            handle_delete_selected_calendar(app);
        }
        Message::RequestDeleteCalendar(id) => {
            DialogManager::close(&mut app.active_dialog);
            handle_request_delete_calendar(app, id);
        }
        Message::SelectCalendarByIndex(index) => {
            DialogManager::close(&mut app.active_dialog);
            if let Some(calendar) = app.calendar_manager.sources().get(index) {
                let id = calendar.info().id.clone();
                app.selected_calendar_id = Some(id);
                app.update_selected_calendar_color();
            }
        }
        Message::DeleteCalendarByIndex(index) => {
            DialogManager::close(&mut app.active_dialog);
            if let Some(calendar) = app.calendar_manager.sources().get(index) {
                let id = calendar.info().id.clone();
                handle_request_delete_calendar(app, id);
            }
        }
        Message::ConfirmDeleteCalendar => {
            handle_confirm_delete_calendar(app);
        }
        Message::CancelDeleteCalendar => {
            DialogManager::close(&mut app.active_dialog);
        }

        // === Selection - Drag Selection for Multi-Day Events ===
        Message::SelectionStart(date) => {
            // Cancel any empty quick event when starting a selection
            DialogManager::dismiss_empty_quick_event(&mut app.active_dialog);
            handle_selection_start(app, date);
        }
        Message::SelectionUpdate(date) => {
            handle_selection_update(app, date);
        }
        Message::SelectionEnd => {
            handle_selection_end(app);
            // Focus the quick event input if a quick event was started
            if app.active_dialog.is_quick_event() {
                return focus_quick_event_input();
            }
        }
        Message::SelectionCancel => {
            handle_selection_cancel(app);
        }

        // === Time-Based Selection - Drag Selection for Timed Events ===
        Message::TimeSelectionStart(date, time) => {
            // Cancel any empty quick event when starting a selection
            DialogManager::dismiss_empty_quick_event(&mut app.active_dialog);
            handle_time_selection_start(app, date, time);
        }
        Message::TimeSelectionUpdate(date, time) => {
            handle_time_selection_update(app, date, time);
        }
        Message::TimeSelectionEnd => {
            handle_time_selection_end(app);
            // Focus the quick event input if a quick event was started
            if let Some((start_time, _end_time)) = app.active_dialog.quick_event_times() {
                // Save the current scroll position BEFORE we scroll to show the input
                // This will be used to restore when the quick event is canceled/committed
                app.week_view_scroll_restore = app.week_view_scroll_opt;
                // Chain: focus input, then scroll to the start time position
                // The scroll ensures the input is visible after focusing
                let focus_task = focus_quick_event_input();
                let scroll_task = scroll_week_to_hour(start_time.hour());
                return Task::batch([focus_task, scroll_task]);
            } else if app.active_dialog.is_quick_event() {
                return focus_quick_event_input();
            }
        }

        // === Event Management - Quick Events ===
        Message::StartQuickEvent(date) => {
            handle_start_quick_event(app, date);
            return focus_quick_event_input();
        }
        Message::StartQuickTimedEvent(date, start_time, end_time) => {
            handle_start_quick_timed_event(app, date, start_time, end_time);
            return focus_quick_event_input();
        }
        Message::QuickEventTextChanged(text) => {
            handle_quick_event_text_changed(app, text);
        }
        Message::CommitQuickEvent => {
            handle_commit_quick_event(app);
            // Schedule deferred scroll restore after UI updates
            return schedule_deferred_scroll_restore(app);
        }
        Message::CancelQuickEvent => {
            handle_cancel_quick_event(app);
            // Schedule deferred scroll restore after UI updates
            return schedule_deferred_scroll_restore(app);
        }
        Message::DeleteEvent(uid) => {
            handle_delete_event(app, uid);
        }
        Message::RequestDeleteSelectedEvent => {
            // Request delete of selected event - opens confirmation dialog
            if let Some(uid) = app.selected_event_uid.clone() {
                // Extract master UID for recurring events (occurrence UIDs have format master-uid_YYYYMMDD)
                let master_uid = extract_master_uid(&uid);
                // Extract the occurrence date from the UID (if it's an occurrence)
                let occurrence_date = extract_occurrence_date(&uid);
                // Find the event to get its name and check if it's recurring
                if let Ok((event, _calendar_id)) = crate::services::EventHandler::find_event(&app.calendar_manager, master_uid) {
                    let is_recurring = !matches!(event.repeat, crate::caldav::RepeatFrequency::Never);
                    DialogManager::open(
                        &mut app.active_dialog,
                        ActiveDialog::EventDelete {
                            event_uid: uid,
                            event_name: event.summary,
                            is_recurring,
                            occurrence_date,
                        },
                    );
                } else {
                    debug!("RequestDeleteSelectedEvent: Event not found: {} (master_uid={})", uid, master_uid);
                }
            } else {
                debug!("RequestDeleteSelectedEvent: No event selected");
            }
        }
        Message::ConfirmDeleteEvent => {
            // Confirm event deletion from the dialog (deletes all occurrences for recurring events)
            if let Some((event_uid, _event_name, _is_recurring, _occurrence_date)) = app.active_dialog.event_delete_data() {
                let uid = event_uid.to_string();
                // For recurring events, we need to delete the master event
                let master_uid = extract_master_uid(&uid).to_string();
                // Close dialog first
                DialogManager::close(&mut app.active_dialog);
                // Clear selection
                app.selected_event_uid = None;
                // Delete the master event (all occurrences)
                handle_delete_event(app, master_uid);
            }
        }
        Message::DeleteSingleOccurrence => {
            // Delete only the selected occurrence of a recurring event by adding an exception date
            if let Some((event_uid, _event_name, is_recurring, occurrence_date)) = app.active_dialog.event_delete_data() {
                if !is_recurring {
                    debug!("DeleteSingleOccurrence: Event is not recurring, ignoring");
                    DialogManager::close(&mut app.active_dialog);
                    return Task::none();
                }

                let Some(date) = occurrence_date else {
                    debug!("DeleteSingleOccurrence: No occurrence date available");
                    DialogManager::close(&mut app.active_dialog);
                    return Task::none();
                };

                // Extract the master UID from the occurrence UID
                let master_uid = extract_master_uid(event_uid).to_string();

                // Close dialog first
                DialogManager::close(&mut app.active_dialog);
                // Clear selection
                app.selected_event_uid = None;

                // Add the exception date to the master event
                if let Err(e) = crate::services::EventHandler::add_exception_date(
                    &mut app.calendar_manager,
                    &master_uid,
                    date,
                ) {
                    error!("DeleteSingleOccurrence: Failed to add exception date: {}", e);
                } else {
                    info!("DeleteSingleOccurrence: Successfully added exception date {} to event {}", date, master_uid);
                    // Refresh calendar cache to reflect the change
                    app.refresh_cached_events();
                }
            }
        }
        Message::CancelDeleteEvent => {
            // Cancel event deletion
            DialogManager::close(&mut app.active_dialog);
        }
        Message::SelectEvent(uid) => {
            handle_select_event(app, uid);
        }

        // === Event Drag-and-Drop ===
        Message::DragEventStart(calendar_id, uid, date, summary, color) => {
            handle_drag_event_start(app, calendar_id, uid, date, summary, color);
        }
        Message::DragEventUpdate(date) => {
            handle_drag_event_update(app, date);
        }
        Message::DragEventCursorMove(x, y) => {
            app.event_drag_state.update_cursor(x, y);
        }
        Message::DragEventEnd => {
            handle_drag_event_end(app);
        }
        Message::DragEventCancel => {
            handle_drag_event_cancel(app);
        }

        // === Event Management - Event Dialog ===
        Message::OpenNewEventDialog => {
            handle_open_new_event_dialog(app);
        }
        Message::OpenEditEventDialog(calendar_id, uid) => {
            // Cancel any drag operation that may have started from the first click of double-click
            app.event_drag_state.cancel();
            // Clear selection since we're opening the edit dialog
            app.selected_event_uid = None;
            handle_open_edit_event_dialog(app, calendar_id, uid);
        }
        Message::EventDialogToggleEdit(field, editing) => {
            #[allow(deprecated)]
            if let Some(ref mut dialog) = app.event_dialog {
                dialog.editing_field = if editing { Some(field) } else { None };
            }
        }
        Message::EventDialogTitleChanged(title) => {
            #[allow(deprecated)]
            if let Some(ref mut dialog) = app.event_dialog {
                dialog.title = title;
            }
        }
        Message::EventDialogLocationChanged(location) => {
            #[allow(deprecated)]
            if let Some(ref mut dialog) = app.event_dialog {
                dialog.location = location;
            }
        }
        Message::EventDialogAllDayToggled(all_day) => {
            #[allow(deprecated)]
            if let Some(ref mut dialog) = app.event_dialog {
                dialog.all_day = all_day;
            }
        }
        Message::EventDialogStartDateInputChanged(input) => {
            #[allow(deprecated)]
            if let Some(ref mut dialog) = app.event_dialog {
                dialog.start_date_input = input.clone();
                // Try to parse the date
                if let Ok(date) = chrono::NaiveDate::parse_from_str(&input, "%Y-%m-%d") {
                    dialog.start_date = date;
                    // If end date is before start, adjust it
                    if dialog.end_date < date {
                        dialog.end_date = date;
                        dialog.end_date_input = date.format("%Y-%m-%d").to_string();
                    }
                }
            }
        }
        Message::EventDialogStartDateChanged(date) => {
            #[allow(deprecated)]
            if let Some(ref mut dialog) = app.event_dialog {
                dialog.start_date = date;
                dialog.start_date_input = date.format("%Y-%m-%d").to_string();
                dialog.start_date_calendar.set_selected_visible(date);
                dialog.start_date_picker_open = false; // Close picker after selection
                // If end date is before start, adjust it
                if dialog.end_date < date {
                    dialog.end_date = date;
                    dialog.end_date_input = date.format("%Y-%m-%d").to_string();
                    dialog.end_date_calendar.set_selected_visible(date);
                }
            }
        }
        Message::EventDialogToggleStartDatePicker => {
            #[allow(deprecated)]
            if let Some(ref mut dialog) = app.event_dialog {
                dialog.start_date_picker_open = !dialog.start_date_picker_open;
                dialog.end_date_picker_open = false; // Close the other picker
            }
        }
        Message::EventDialogStartDateCalendarPrev => {
            #[allow(deprecated)]
            if let Some(ref mut dialog) = app.event_dialog {
                dialog.start_date_calendar.show_prev_month();
            }
        }
        Message::EventDialogStartDateCalendarNext => {
            #[allow(deprecated)]
            if let Some(ref mut dialog) = app.event_dialog {
                dialog.start_date_calendar.show_next_month();
            }
        }
        Message::EventDialogToggleStartTimePicker => {
            #[allow(deprecated)]
            if let Some(ref mut dialog) = app.event_dialog {
                let was_open = dialog.start_time_picker_open;
                dialog.start_time_picker_open = !was_open;
                dialog.end_time_picker_open = false;
                dialog.start_date_picker_open = false;
                dialog.end_date_picker_open = false;

                // When opening the picker, scroll to the already-selected time
                if !was_open {
                    use crate::components::time_picker::scroll_start_time_to;
                    use chrono::Timelike;

                    if let Some(time) = dialog.start_time {
                        return scroll_start_time_to(time.hour(), time.minute());
                    }
                }
            }
        }
        Message::EventDialogStartTimeHourChanged(hour) => {
            #[allow(deprecated)]
            if let Some(ref mut dialog) = app.event_dialog {
                let current = dialog.start_time.unwrap_or_else(|| chrono::NaiveTime::from_hms_opt(9, 0, 0).unwrap());
                if let Some(new_time) = chrono::NaiveTime::from_hms_opt(hour, current.minute(), 0) {
                    dialog.start_time = Some(new_time);
                    dialog.start_time_input = new_time.format("%H:%M").to_string();
                    // Auto-update end time to 1 hour after start time
                    let end_hour = (new_time.hour() + 1) % 24;
                    if let Some(end_time) = chrono::NaiveTime::from_hms_opt(end_hour, new_time.minute(), 0) {
                        dialog.end_time = Some(end_time);
                        dialog.end_time_input = end_time.format("%H:%M").to_string();
                    }
                }
            }
        }
        Message::EventDialogStartTimeMinuteChanged(minute) => {
            #[allow(deprecated)]
            if let Some(ref mut dialog) = app.event_dialog {
                let current = dialog.start_time.unwrap_or_else(|| chrono::NaiveTime::from_hms_opt(9, 0, 0).unwrap());
                if let Some(new_time) = chrono::NaiveTime::from_hms_opt(current.hour(), minute, 0) {
                    dialog.start_time = Some(new_time);
                    dialog.start_time_input = new_time.format("%H:%M").to_string();
                    // Auto-update end time to 1 hour after start time
                    let end_hour = (new_time.hour() + 1) % 24;
                    if let Some(end_time) = chrono::NaiveTime::from_hms_opt(end_hour, new_time.minute(), 0) {
                        dialog.end_time = Some(end_time);
                        dialog.end_time_input = end_time.format("%H:%M").to_string();
                    }
                }
            }
        }
        Message::EventDialogEndDateInputChanged(input) => {
            #[allow(deprecated)]
            if let Some(ref mut dialog) = app.event_dialog {
                dialog.end_date_input = input.clone();
                // Try to parse the date
                if let Ok(date) = chrono::NaiveDate::parse_from_str(&input, "%Y-%m-%d") {
                    dialog.end_date = date;
                }
            }
        }
        Message::EventDialogEndDateChanged(date) => {
            #[allow(deprecated)]
            if let Some(ref mut dialog) = app.event_dialog {
                dialog.end_date = date;
                dialog.end_date_input = date.format("%Y-%m-%d").to_string();
                dialog.end_date_calendar.set_selected_visible(date);
                dialog.end_date_picker_open = false; // Close picker after selection
            }
        }
        Message::EventDialogToggleEndDatePicker => {
            #[allow(deprecated)]
            if let Some(ref mut dialog) = app.event_dialog {
                dialog.end_date_picker_open = !dialog.end_date_picker_open;
                dialog.start_date_picker_open = false; // Close the other picker
            }
        }
        Message::EventDialogEndDateCalendarPrev => {
            #[allow(deprecated)]
            if let Some(ref mut dialog) = app.event_dialog {
                dialog.end_date_calendar.show_prev_month();
            }
        }
        Message::EventDialogEndDateCalendarNext => {
            #[allow(deprecated)]
            if let Some(ref mut dialog) = app.event_dialog {
                dialog.end_date_calendar.show_next_month();
            }
        }
        Message::EventDialogToggleEndTimePicker => {
            #[allow(deprecated)]
            if let Some(ref mut dialog) = app.event_dialog {
                let was_open = dialog.end_time_picker_open;
                dialog.end_time_picker_open = !was_open;
                dialog.start_time_picker_open = false;
                dialog.start_date_picker_open = false;
                dialog.end_date_picker_open = false;

                // When opening the picker, scroll to the already-selected time
                if !was_open {
                    use crate::components::time_picker::scroll_end_time_to;
                    use chrono::Timelike;

                    if let Some(time) = dialog.end_time {
                        return scroll_end_time_to(time.hour(), time.minute());
                    }
                }
            }
        }
        Message::EventDialogEndTimeHourChanged(hour) => {
            #[allow(deprecated)]
            if let Some(ref mut dialog) = app.event_dialog {
                let current = dialog.end_time.unwrap_or_else(|| chrono::NaiveTime::from_hms_opt(10, 0, 0).unwrap());
                if let Some(new_time) = chrono::NaiveTime::from_hms_opt(hour, current.minute(), 0) {
                    dialog.end_time = Some(new_time);
                    dialog.end_time_input = new_time.format("%H:%M").to_string();
                }
            }
        }
        Message::EventDialogEndTimeMinuteChanged(minute) => {
            #[allow(deprecated)]
            if let Some(ref mut dialog) = app.event_dialog {
                let current = dialog.end_time.unwrap_or_else(|| chrono::NaiveTime::from_hms_opt(10, 0, 0).unwrap());
                if let Some(new_time) = chrono::NaiveTime::from_hms_opt(current.hour(), minute, 0) {
                    dialog.end_time = Some(new_time);
                    dialog.end_time_input = new_time.format("%H:%M").to_string();
                }
            }
        }
        Message::EventDialogTravelTimeChanged(travel_time) => {
            #[allow(deprecated)]
            if let Some(ref mut dialog) = app.event_dialog {
                dialog.travel_time = travel_time;
            }
        }
        Message::EventDialogRepeatChanged(repeat) => {
            #[allow(deprecated)]
            if let Some(ref mut dialog) = app.event_dialog {
                dialog.repeat = repeat;
            }
        }
        Message::EventDialogCalendarChanged(calendar_id) => {
            #[allow(deprecated)]
            if let Some(ref mut dialog) = app.event_dialog {
                dialog.calendar_id = calendar_id;
            }
        }
        Message::EventDialogInviteeInputChanged(input) => {
            #[allow(deprecated)]
            if let Some(ref mut dialog) = app.event_dialog {
                dialog.invitee_input = input;
            }
        }
        Message::EventDialogAddInvitee => {
            #[allow(deprecated)]
            if let Some(ref mut dialog) = app.event_dialog {
                let email = dialog.invitee_input.trim().to_string();
                if !email.is_empty() && !dialog.invitees.contains(&email) {
                    dialog.invitees.push(email);
                    dialog.invitee_input.clear();
                }
            }
        }
        Message::EventDialogRemoveInvitee(index) => {
            #[allow(deprecated)]
            if let Some(ref mut dialog) = app.event_dialog {
                if index < dialog.invitees.len() {
                    dialog.invitees.remove(index);
                }
            }
        }
        Message::EventDialogAlertChanged(alert) => {
            #[allow(deprecated)]
            if let Some(ref mut dialog) = app.event_dialog {
                dialog.alert = alert;
            }
        }
        Message::EventDialogAlertSecondChanged(alert) => {
            #[allow(deprecated)]
            if let Some(ref mut dialog) = app.event_dialog {
                dialog.alert_second = alert;
            }
        }
        Message::EventDialogAddAttachment(path) => {
            #[allow(deprecated)]
            if let Some(ref mut dialog) = app.event_dialog {
                if !dialog.attachments.contains(&path) {
                    dialog.attachments.push(path);
                }
            }
        }
        Message::EventDialogRemoveAttachment(index) => {
            #[allow(deprecated)]
            if let Some(ref mut dialog) = app.event_dialog {
                if index < dialog.attachments.len() {
                    dialog.attachments.remove(index);
                }
            }
        }
        Message::EventDialogUrlChanged(url) => {
            #[allow(deprecated)]
            if let Some(ref mut dialog) = app.event_dialog {
                dialog.url = url;
            }
        }
        Message::EventDialogNotesAction(action) => {
            #[allow(deprecated)]
            if let Some(ref mut dialog) = app.event_dialog {
                dialog.notes_content.perform(action);
            }
        }
        Message::ConfirmEventDialog => {
            handle_confirm_event_dialog(app);
        }
        Message::CancelEventDialog => {
            handle_cancel_event_dialog(app);
        }

        // === Mini Calendar ===
        Message::MiniCalendarPrevMonth => {
            app.navigate_mini_calendar_previous();
        }
        Message::MiniCalendarNextMonth => {
            app.navigate_mini_calendar_next();
        }

        // === Menu Actions ===
        Message::NewEvent => {
            handle_open_new_event_dialog(app);
        }
        Message::ImportICal => {
            // Open file picker dialog using XDG portal (Flatpak-compatible)
            info!("Message::ImportICal: Opening file picker");

            return Task::perform(
                async {
                    rfd::AsyncFileDialog::new()
                        .add_filter("iCalendar files", &["ics", "ical", "ifb", "icalendar"])
                        .add_filter("All files", &["*"])
                        .set_title("Import iCalendar File")
                        .pick_file()
                        .await
                        .map(|handle| handle.path().to_path_buf())
                },
                |option_path| {
                    if let Some(path) = option_path {
                        cosmic::Action::App(Message::ImportFile(path))
                    } else {
                        // User cancelled the file picker
                        cosmic::Action::App(Message::CancelImport)
                    }
                },
            );
        }
        Message::ExportICal => {
            // Export the first calendar to Downloads folder
            info!("Message::ExportICal: Export iCal requested");

            if let Some(calendar) = app.calendar_manager.sources().first() {
                let calendar_id = calendar.info().id.clone();
                let calendar_name = calendar.info().name.clone();

                // Create export filename with timestamp
                let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
                let filename = format!("{}_{}.ics", calendar_name.replace(" ", "_"), timestamp);

                // Export to Downloads folder
                if let Some(downloads_dir) = dirs::download_dir() {
                    let export_path = downloads_dir.join(&filename);

                    match ExportHandler::export_to_file(
                        &app.calendar_manager,
                        &calendar_id,
                        &export_path,
                    ) {
                        Ok(_) => {
                            info!("Message::ExportICal: Exported to {:?}", export_path);
                            // TODO: Show success notification
                        }
                        Err(e) => {
                            error!("Message::ExportICal: Export failed: {}", e);
                            // TODO: Show error notification
                        }
                    }
                } else {
                    error!("Message::ExportICal: Could not determine Downloads folder");
                }
            } else {
                error!("Message::ExportICal: No calendars available to export");
            }
        }
        Message::Settings => {
            // TODO: Open settings dialog
            info!("Message::Settings: Settings requested (not yet implemented)");
        }
        Message::About => {
            app.core.window.show_context = !app.core.window.show_context;
        }
        Message::LaunchUrl(url) => {
            // Open URL in default browser
            let _ = open::that(&url);
        }
        Message::ToggleContextDrawer => {
            app.core.window.show_context = !app.core.window.show_context;
        }
        Message::Surface(action) => {
            return cosmic::task::message(cosmic::Action::Cosmic(
                cosmic::app::Action::Surface(action),
            ));
        }

        // === Import/Export Operations ===
        Message::ImportFile(path) => {
            return import::handle_import_file(app, path);
        }
        Message::ShowImportDialog(events, source_file_name) => {
            return import::handle_show_import_dialog(app, events, source_file_name);
        }
        Message::SelectImportCalendar(calendar_id) => {
            return import::handle_select_import_calendar(app, calendar_id);
        }
        Message::ConfirmImport => {
            return import::handle_confirm_import(app);
        }
        Message::CancelImport => {
            return import::handle_cancel_import(app);
        }
        Message::ImportProgressUpdate(index, summary) => {
            // Update progress dialog state
            if let ActiveDialog::ImportProgress {
                current,
                current_event,
                ..
            } = &mut app.active_dialog
            {
                *current = index;
                *current_event = summary;
            }
        }
        Message::CancelImportProgress => {
            return import::handle_cancel_import_progress(app);
        }
        Message::RevertImport => {
            return import::handle_revert_import(app);
        }
        Message::ExportCalendarToFile(calendar_id, path) => {
            return handle_export_calendar_to_file(app, calendar_id, path);
        }

        Message::ProcessUrl(url) => {
            return handle_process_url(app, url);
        }

        Message::ProcessDownloadedCalendar(url, calendar_data) => {
            return handle_process_downloaded_calendar(app, url, calendar_data);
        }

        Message::ShowSubscribeDialog(url, calendar_name, events) => {
            return handle_show_subscribe_dialog(app, url, calendar_name, events);
        }

        Message::SelectSubscriptionCalendar(calendar_id) => {
            return handle_select_subscription_calendar(app, calendar_id);
        }

        Message::ToggleCreateNewCalendar(create_new) => {
            return handle_toggle_create_new_calendar(app, create_new);
        }

        Message::UpdateSubscriptionCalendarName(name) => {
            return handle_update_subscription_calendar_name(app, name);
        }

        Message::ConfirmSubscription => {
            return handle_confirm_subscription(app);
        }

        Message::CancelSubscription => {
            return handle_cancel_subscription(app);
        }

        // No-op for cancelled operations
        Message::None => {}
    }

    Task::none()
}
