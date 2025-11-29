use crate::views::CalendarView;

#[derive(Debug, Clone)]
pub enum Message {
    ChangeView(CalendarView),
    PreviousPeriod,
    NextPeriod,
    Today,
    SelectDay(i32, u32, u32), // (year, month, day)
    ToggleSidebar,
    ToggleSearch,
    ToggleWeekNumbers,
    ToggleCalendar(String),
    OpenColorPicker(String),
    ChangeCalendarColor(String, String),
    CloseColorPicker,
    MiniCalendarPrevMonth,
    MiniCalendarNextMonth,
    NewEvent,
    Settings,
    About,
    LaunchUrl(String),
    ToggleContextDrawer,
    Surface(cosmic::surface::Action),
}
