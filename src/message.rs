use crate::views::CalendarView;

#[derive(Debug, Clone)]
pub enum Message {
    ChangeView(CalendarView),
    PreviousPeriod,
    NextPeriod,
    Today,
    SelectDay(u32),
    ToggleSidebar,
    ToggleSearch,
    MiniCalendarPrevMonth,
    MiniCalendarNextMonth,
    NewEvent,
    Settings,
    About,
}
