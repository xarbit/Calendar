/// UI constants for consistent styling across the application

// Layout dimensions
pub const SIDEBAR_WIDTH: f32 = 280.0;
pub const MINI_CALENDAR_DAY_BUTTON_SIZE: f32 = 32.0;

// Spacing values
pub const SPACING_TINY: u16 = 1;
pub const SPACING_SMALL: u16 = 4;
pub const SPACING_MEDIUM: u16 = 8;
pub const SPACING_LARGE: u16 = 20;

// Padding values
pub const PADDING_SMALL: u16 = 8;
pub const PADDING_MEDIUM: u16 = 12;
pub const PADDING_STANDARD: u16 = 16;
pub const PADDING_LARGE: u16 = 40;

// Border styling
pub const BORDER_RADIUS: [f32; 4] = [4.0, 4.0, 4.0, 4.0];
pub const LIGHT_BORDER_OPACITY: f32 = 0.2;

// Icon names
pub const ICON_PREVIOUS: &str = "go-previous-symbolic";
pub const ICON_NEXT: &str = "go-next-symbolic";
pub const ICON_SIDEBAR: &str = "sidebar-show-symbolic";
pub const ICON_SEARCH: &str = "system-search-symbolic";

// Weekday names
pub const WEEKDAYS_FULL: [&str; 7] = [
    "Monday",
    "Tuesday",
    "Wednesday",
    "Thursday",
    "Friday",
    "Saturday",
    "Sunday"
];

pub const WEEKDAYS_SHORT: [&str; 7] = ["M", "T", "W", "T", "F", "S", "S"];

// Font sizes
pub const FONT_SIZE_SMALL: u16 = 11;
pub const FONT_SIZE_MEDIUM: u16 = 12;
pub const FONT_SIZE_BODY: u16 = 14;
