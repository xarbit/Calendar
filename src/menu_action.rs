use cosmic::widget::menu;

use crate::message::Message;
use crate::views::CalendarView;

/// Menu actions for the application menu bar
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MenuAction {
    NewEvent,
    NewCalendar,
    ImportICal,
    ExportICal,
    Settings,
    Today,
    ViewYear,
    ViewMonth,
    ViewWeek,
    ViewDay,
    ToggleWeekNumbers,
    About,
}

impl menu::action::MenuAction for MenuAction {
    type Message = Message;

    fn message(&self) -> Self::Message {
        match self {
            MenuAction::NewEvent => Message::NewEvent,
            MenuAction::NewCalendar => Message::OpenNewCalendarDialog,
            MenuAction::ImportICal => Message::ImportICal,
            MenuAction::ExportICal => Message::ExportICal,
            MenuAction::Settings => Message::Settings,
            MenuAction::Today => Message::Today,
            MenuAction::ViewYear => Message::ChangeView(CalendarView::Year),
            MenuAction::ViewMonth => Message::ChangeView(CalendarView::Month),
            MenuAction::ViewWeek => Message::ChangeView(CalendarView::Week),
            MenuAction::ViewDay => Message::ChangeView(CalendarView::Day),
            MenuAction::ToggleWeekNumbers => Message::ToggleWeekNumbers,
            MenuAction::About => Message::About,
        }
    }
}
