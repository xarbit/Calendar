use chrono::NaiveDate;
use crate::views::CalendarView;

#[derive(Debug, Clone)]
pub enum Message {
    // View navigation
    ChangeView(CalendarView),
    PreviousPeriod,
    NextPeriod,
    Today,
    SelectDay(i32, u32, u32), // (year, month, day)

    // UI state
    ToggleSidebar,
    /// Triggered on window resize to sync sidebar with condensed state
    WindowResized,
    ToggleSearch,
    ToggleWeekNumbers,

    // Calendar management
    ToggleCalendar(String),
    /// Select a calendar as the active calendar for new events
    SelectCalendar(String),
    OpenColorPicker(String),
    ChangeCalendarColor(String, String),
    CloseColorPicker,
    /// Open the new calendar dialog
    OpenNewCalendarDialog,
    /// Update new calendar name while typing
    NewCalendarNameChanged(String),
    /// Update new calendar color selection
    NewCalendarColorChanged(String),
    /// Confirm creating the new calendar
    ConfirmNewCalendar,
    /// Cancel new calendar dialog
    CancelNewCalendar,
    /// Delete the currently selected calendar (with confirmation)
    DeleteSelectedCalendar,
    /// Request to delete a specific calendar by ID (opens confirmation dialog)
    RequestDeleteCalendar(String),
    /// Select calendar by index (from context menu)
    SelectCalendarByIndex(usize),
    /// Delete calendar by index (from context menu)
    DeleteCalendarByIndex(usize),
    /// Confirm calendar deletion
    ConfirmDeleteCalendar,
    /// Cancel calendar deletion
    CancelDeleteCalendar,

    // Event management
    /// Start creating a quick event on a specific date
    StartQuickEvent(NaiveDate),
    /// Update the quick event text while editing
    QuickEventTextChanged(String),
    /// Commit the quick event (on Enter press)
    CommitQuickEvent,
    /// Cancel quick event editing (on Escape or click outside)
    CancelQuickEvent,
    /// Delete an event by its UID
    DeleteEvent(String),

    // Mini calendar
    MiniCalendarPrevMonth,
    MiniCalendarNextMonth,

    // Menu actions
    NewEvent,
    ImportICal,
    ExportICal,
    Settings,
    About,
    LaunchUrl(String),
    ToggleContextDrawer,
    Surface(cosmic::surface::Action),
}
