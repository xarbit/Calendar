use crate::cache::CalendarCache;
use crate::calendars::CalendarManager;
use crate::components;
use crate::fl;
use crate::locale::LocalePreferences;
use crate::menu_action::MenuAction;
use crate::message::Message;
use crate::models::{CalendarState, WeekState, DayState, YearState};
use crate::settings::AppSettings;
use crate::storage::LocalStorage;
use crate::views::{self, CalendarView};
use chrono::{Datelike, NaiveDate};
use cosmic::app::Core;
use cosmic::iced::keyboard;
use cosmic::widget::{about, menu};
use cosmic::widget::menu::Action as _; // Import trait for .message() method
use cosmic::{Application, Element};
use std::collections::HashMap;

const APP_ID: &str = "io.github.xarbit.SolCalendar";

/// Main application state
pub struct CosmicCalendar {
    pub core: Core,
    pub current_view: CalendarView,
    /// The anchor date shared by all views - when you select a date or switch views,
    /// all views sync to show the period containing this date
    pub selected_date: NaiveDate,
    #[allow(dead_code)] // Will be used for event storage in future
    pub storage: LocalStorage,
    pub calendar_manager: CalendarManager,
    pub show_sidebar: bool,
    pub show_search: bool,
    pub color_picker_open: Option<String>,
    pub cache: CalendarCache,
    pub week_state: WeekState,
    pub day_state: DayState,
    pub year_state: YearState,
    /// Mini calendar state - independent from main view for browsing
    pub mini_calendar_state: CalendarState,
    pub locale: LocalePreferences,
    pub settings: AppSettings,
    pub about: about::About,
    pub key_binds: HashMap<menu::KeyBind, MenuAction>,
}

impl CosmicCalendar {
    /// Initialize the application with default calendars
    fn initialize_app(core: Core) -> Self {
        let today = chrono::Local::now().date_naive();
        let storage_path = LocalStorage::get_storage_path();
        let storage = LocalStorage::load_from_file(&storage_path).unwrap_or_default();

        let year = today.year();
        let month = today.month();

        // Create cache and pre-cache surrounding months
        let mut cache = CalendarCache::new(year, month);
        cache.precache_surrounding(1, 2);

        // Initialize calendar manager with default calendars
        let calendar_manager = CalendarManager::with_defaults();

        // Load application settings
        let settings = AppSettings::load().unwrap_or_default();

        // Create About dialog
        let about = about::About::default()
            .name(fl!("app-title"))
            .version(env!("CARGO_PKG_VERSION"))
            .license(fl!("about-license"))
            .links([(fl!("about-repository"), "https://github.com/xarbit/sol")]);

        // Detect system locale preferences
        let locale = LocalePreferences::detect_from_system();

        // Initialize keyboard shortcuts from centralized module
        let key_binds = crate::keyboard::init_key_binds();

        // Mini calendar starts showing the current month
        let mini_calendar_state = CalendarState::new(year, month);

        CosmicCalendar {
            core,
            current_view: CalendarView::Month,
            selected_date: today,
            storage,
            calendar_manager,
            show_sidebar: true,
            show_search: false,
            color_picker_open: None,
            cache,
            week_state: WeekState::current_with_first_day(locale.first_day_of_week, &locale),
            day_state: DayState::current(&locale),
            year_state: YearState::current(),
            mini_calendar_state,
            locale,
            settings,
            about,
            key_binds,
        }
    }

    /// Sync all views to show the period containing the selected_date
    pub fn sync_views_to_selected_date(&mut self) {
        let date = self.selected_date;
        let year = date.year();
        let month = date.month();

        // Update month view cache
        self.cache.set_current(year, month);
        self.cache.precache_surrounding(1, 2);

        // Update week view
        self.week_state = WeekState::new(date, self.locale.first_day_of_week, &self.locale);

        // Update day view
        self.day_state = DayState::new(date, &self.locale);

        // Update year view
        self.year_state = YearState::new(year);

        // Sync mini calendar to show the month containing selected_date
        self.mini_calendar_state = CalendarState::new(year, month);
    }

    /// Set the selected date and sync all views
    pub fn set_selected_date(&mut self, date: NaiveDate) {
        self.selected_date = date;
        self.sync_views_to_selected_date();
    }

    /// Navigate to today in the current view
    pub fn navigate_to_today(&mut self) {
        let today = chrono::Local::now().date_naive();
        self.set_selected_date(today);
    }

    /// Navigate to the previous period based on current view
    pub fn navigate_mini_calendar_previous(&mut self) {
        let state = &self.mini_calendar_state;
        let (year, month) = if state.month == 1 {
            (state.year - 1, 12)
        } else {
            (state.year, state.month - 1)
        };
        self.mini_calendar_state = CalendarState::new(year, month);
    }

    /// Navigate to the next period based on current view
    pub fn navigate_mini_calendar_next(&mut self) {
        let state = &self.mini_calendar_state;
        let (year, month) = if state.month == 12 {
            (state.year + 1, 1)
        } else {
            (state.year, state.month + 1)
        };
        self.mini_calendar_state = CalendarState::new(year, month);
    }

    /// Render the sidebar
    pub fn render_sidebar(&self) -> Element<'_, Message> {
        let selected_day = if self.mini_calendar_state.year == self.selected_date.year()
            && self.mini_calendar_state.month == self.selected_date.month()
        {
            Some(self.selected_date.day())
        } else {
            None
        };

        views::render_sidebar(
            &self.mini_calendar_state,
            self.calendar_manager.sources(),
            selected_day,
            self.color_picker_open.as_ref(),
        )
    }

    /// Render the main content area (toolbar + calendar view)
    pub fn render_main_content(&self) -> Element<'_, Message> {
        // For month view, only show selection if we're viewing the month containing selected_date
        let selected_day = {
            let cache_state = self.cache.current_state();
            if cache_state.year == self.selected_date.year()
                && cache_state.month == self.selected_date.month()
            {
                Some(self.selected_date.day())
            } else {
                None
            }
        };

        views::render_main_content(&self.cache, &self.week_state, &self.day_state, &self.year_state, &self.locale, self.current_view, selected_day, self.settings.show_week_numbers)
    }
}

impl Default for CosmicCalendar {
    fn default() -> Self {
        Self::initialize_app(Core::default())
    }
}

impl Application for CosmicCalendar {
    type Executor = cosmic::executor::Default;
    type Flags = ();
    type Message = Message;
    const APP_ID: &'static str = APP_ID;

    fn core(&self) -> &Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut Core {
        &mut self.core
    }

    fn init(core: Core, _flags: Self::Flags) -> (Self, cosmic::app::Task<Self::Message>) {
        (Self::initialize_app(core), cosmic::app::Task::none())
    }

    fn header_start(&self) -> Vec<Element<'_, Self::Message>> {
        components::render_header_start(&self.core, &self.key_binds, self.show_sidebar, self.settings.show_week_numbers)
    }

    fn header_end(&self) -> Vec<Element<'_, Self::Message>> {
        components::render_header_end()
    }

    fn view(&self) -> Element<'_, Self::Message> {
        crate::layout::render_layout(self)
    }

    fn update(&mut self, message: Self::Message) -> cosmic::app::Task<Self::Message> {
        crate::update::handle_message(self, message)
    }

    fn context_drawer(&self) -> Option<cosmic::app::context_drawer::ContextDrawer<'_, Self::Message>> {
        if !self.core.window.show_context {
            return None;
        }

        Some(cosmic::app::context_drawer::about(
            &self.about,
            |url| Message::LaunchUrl(url.to_string()),
            Message::ToggleContextDrawer,
        ))
    }

    fn subscription(&self) -> cosmic::iced::Subscription<Self::Message> {
        cosmic::iced::event::listen_with(|event, _status, _window_id| {
            if let cosmic::iced::Event::Keyboard(keyboard::Event::KeyPressed {
                key,
                modifiers,
                ..
            }) = event
            {
                // Convert modifiers to menu modifiers
                let mut menu_modifiers = Vec::new();
                if modifiers.control() {
                    menu_modifiers.push(menu::key_bind::Modifier::Ctrl);
                }
                if modifiers.shift() {
                    menu_modifiers.push(menu::key_bind::Modifier::Shift);
                }
                if modifiers.alt() {
                    menu_modifiers.push(menu::key_bind::Modifier::Alt);
                }
                if modifiers.logo() {
                    menu_modifiers.push(menu::key_bind::Modifier::Super);
                }

                let key_bind = menu::KeyBind {
                    modifiers: menu_modifiers,
                    key: key.clone(),
                };

                // Look up the action in the global keyboard shortcuts
                if let Some(action) = crate::keyboard::get_key_binds().get(&key_bind) {
                    return Some(action.message());
                }
            }
            None
        })
    }
}
