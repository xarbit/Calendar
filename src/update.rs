use chrono::{Datelike, NaiveDate};
use crate::app::CosmicCalendar;
use crate::message::Message;
use crate::views::CalendarView;
use cosmic::app::Task;

/// Handle all application messages and update state
pub fn handle_message(app: &mut CosmicCalendar, message: Message) -> Task<Message> {
    match message {
        Message::ChangeView(view) => {
            // When changing views, sync views to the selected_date so the new view
            // shows the period containing the anchor date
            app.current_view = view;
            app.sync_views_to_selected_date();
        }
        Message::PreviousPeriod => {
            handle_previous_period(app);
        }
        Message::NextPeriod => {
            handle_next_period(app);
        }
        Message::Today => {
            // Today button navigates to today in all views
            app.navigate_to_today();
        }
        Message::SelectDay(year, month, day) => {
            // Set the selected date - this syncs all views automatically
            if let Some(date) = NaiveDate::from_ymd_opt(year, month, day) {
                app.set_selected_date(date);
            }
        }
        Message::ToggleSidebar => {
            app.show_sidebar = !app.show_sidebar;
        }
        Message::ToggleSearch => {
            app.show_search = !app.show_search;
        }
        Message::ToggleWeekNumbers => {
            app.settings.show_week_numbers = !app.settings.show_week_numbers;
            // Save settings to persist the change
            app.settings.save().ok();
        }
        Message::ToggleCalendar(id) => {
            handle_toggle_calendar(app, id);
        }
        Message::OpenColorPicker(id) => {
            app.color_picker_open = Some(id);
        }
        Message::ChangeCalendarColor(id, color) => {
            handle_change_calendar_color(app, id, color);
        }
        Message::CloseColorPicker => {
            app.color_picker_open = None;
        }
        Message::MiniCalendarPrevMonth => {
            app.navigate_mini_calendar_previous();
        }
        Message::MiniCalendarNextMonth => {
            app.navigate_mini_calendar_next();
        }
        Message::NewEvent => {
            // TODO: Open new event dialog
            println!("New Event requested");
        }
        Message::Settings => {
            // TODO: Open settings dialog
            println!("Settings requested");
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
    }

    Task::none()
}

/// Handle previous period navigation based on current view
/// This moves the view backwards but updates selected_date to stay in sync
fn handle_previous_period(app: &mut CosmicCalendar) {
    let new_date = match app.current_view {
        CalendarView::Year => {
            // Move back one year
            NaiveDate::from_ymd_opt(
                app.selected_date.year() - 1,
                app.selected_date.month(),
                app.selected_date.day().min(28) // Handle edge cases like Feb 29
            ).or_else(|| NaiveDate::from_ymd_opt(
                app.selected_date.year() - 1,
                app.selected_date.month(),
                28
            ))
        }
        CalendarView::Month => {
            // Move back one month
            let (year, month) = if app.selected_date.month() == 1 {
                (app.selected_date.year() - 1, 12)
            } else {
                (app.selected_date.year(), app.selected_date.month() - 1)
            };
            NaiveDate::from_ymd_opt(year, month, app.selected_date.day().min(28))
                .or_else(|| NaiveDate::from_ymd_opt(year, month, 28))
        }
        CalendarView::Week => {
            // Move back one week
            Some(app.selected_date - chrono::Duration::days(7))
        }
        CalendarView::Day => {
            // Move back one day
            Some(app.selected_date - chrono::Duration::days(1))
        }
    };

    if let Some(date) = new_date {
        app.set_selected_date(date);
    }
}

/// Handle next period navigation based on current view
/// This moves the view forward but updates selected_date to stay in sync
fn handle_next_period(app: &mut CosmicCalendar) {
    let new_date = match app.current_view {
        CalendarView::Year => {
            // Move forward one year
            NaiveDate::from_ymd_opt(
                app.selected_date.year() + 1,
                app.selected_date.month(),
                app.selected_date.day().min(28)
            ).or_else(|| NaiveDate::from_ymd_opt(
                app.selected_date.year() + 1,
                app.selected_date.month(),
                28
            ))
        }
        CalendarView::Month => {
            // Move forward one month
            let (year, month) = if app.selected_date.month() == 12 {
                (app.selected_date.year() + 1, 1)
            } else {
                (app.selected_date.year(), app.selected_date.month() + 1)
            };
            NaiveDate::from_ymd_opt(year, month, app.selected_date.day().min(28))
                .or_else(|| NaiveDate::from_ymd_opt(year, month, 28))
        }
        CalendarView::Week => {
            // Move forward one week
            Some(app.selected_date + chrono::Duration::days(7))
        }
        CalendarView::Day => {
            // Move forward one day
            Some(app.selected_date + chrono::Duration::days(1))
        }
    };

    if let Some(date) = new_date {
        app.set_selected_date(date);
    }
}

/// Toggle a calendar's enabled state and save configuration
fn handle_toggle_calendar(app: &mut CosmicCalendar, id: String) {
    if let Some(calendar) = app
        .calendar_manager
        .sources_mut()
        .iter_mut()
        .find(|c| c.info().id == id)
    {
        calendar.set_enabled(!calendar.is_enabled());
    }
    // Save configuration after toggle
    app.calendar_manager.save_config().ok();
}

/// Change a calendar's color and save configuration
fn handle_change_calendar_color(app: &mut CosmicCalendar, id: String, color: String) {
    if let Some(calendar) = app
        .calendar_manager
        .sources_mut()
        .iter_mut()
        .find(|c| c.info().id == id)
    {
        calendar.info_mut().color = color;
    }
    // Save configuration after color change
    app.calendar_manager.save_config().ok();
    // Close the color picker after selection
    app.color_picker_open = None;
}
