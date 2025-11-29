/// Locale-aware formatting and settings based on system configuration
use std::env;
use chrono::Datelike;

/// Date format order
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DateFormat {
    DMY, // Day-Month-Year (e.g., 31/12/2024) - Most of world
    MDY, // Month-Day-Year (e.g., 12/31/2024) - US
    YMD, // Year-Month-Day (e.g., 2024-12-31) - ISO 8601, East Asia
}

/// Locale preferences for calendar display
#[derive(Debug, Clone, PartialEq)]
pub struct LocalePreferences {
    pub use_24_hour: bool,
    pub first_day_of_week: chrono::Weekday,
    pub date_format: DateFormat,
    pub locale_string: String,
}

impl LocalePreferences {
    /// Detect locale preferences from environment variables
    pub fn detect_from_system() -> Self {
        let locale_string = env::var("LC_TIME")
            .or_else(|_| env::var("LC_ALL"))
            .or_else(|_| env::var("LANG"))
            .unwrap_or_else(|_| "en_US.UTF-8".to_string());

        let use_24_hour = detect_24_hour_format(&locale_string);
        let first_day_of_week = detect_first_day_of_week(&locale_string);
        let date_format = detect_date_format(&locale_string);

        LocalePreferences {
            use_24_hour,
            first_day_of_week,
            date_format,
            locale_string,
        }
    }

    /// Format hour for display (12h or 24h format)
    pub fn format_hour(&self, hour: u32) -> String {
        if self.use_24_hour {
            format!("{:02}:00", hour)
        } else {
            // 12-hour format with AM/PM
            if hour == 0 {
                "12 AM".to_string()
            } else if hour < 12 {
                format!("{} AM", hour)
            } else if hour == 12 {
                "12 PM".to_string()
            } else {
                format!("{} PM", hour - 12)
            }
        }
    }

    /// Get the number of days to subtract from Monday to get first day of week
    /// Monday = 0, Tuesday = 1, ..., Sunday = 6
    pub fn days_from_monday(&self) -> i64 {
        use chrono::Weekday;
        match self.first_day_of_week {
            Weekday::Mon => 0,
            Weekday::Tue => -1,
            Weekday::Wed => -2,
            Weekday::Thu => -3,
            Weekday::Fri => -4,
            Weekday::Sat => -5,
            Weekday::Sun => -6,
        }
    }

    /// Check if a given weekday is a weekend day
    /// Weekend is typically Saturday and Sunday, but this can vary by locale
    pub fn is_weekend(&self, weekday: chrono::Weekday) -> bool {
        use chrono::Weekday;

        // For most locales, weekend is Saturday and Sunday
        // In some Middle Eastern countries, weekend is Friday and Saturday
        // For now, use standard Saturday/Sunday
        matches!(weekday, Weekday::Sat | Weekday::Sun)
    }

    /// Format a date range for week view (e.g., "Nov 24 - 30, 2024" or "24 - 30 Nov, 2024")
    pub fn format_week_range(&self, first_day: &chrono::NaiveDate, last_day: &chrono::NaiveDate, week_number: u32) -> String {
        match self.date_format {
            DateFormat::MDY => {
                // US format: "W48 - Nov 24 - 30, 2024"
                if first_day.month() == last_day.month() {
                    format!(
                        "W{} - {} {} - {}, {}",
                        week_number,
                        first_day.format("%b"),
                        first_day.day(),
                        last_day.day(),
                        first_day.year()
                    )
                } else if first_day.year() == last_day.year() {
                    format!(
                        "W{} - {} {} - {} {}, {}",
                        week_number,
                        first_day.format("%b"),
                        first_day.day(),
                        last_day.format("%b"),
                        last_day.day(),
                        first_day.year()
                    )
                } else {
                    format!(
                        "W{} - {} {}, {} - {} {}, {}",
                        week_number,
                        first_day.format("%b"),
                        first_day.day(),
                        first_day.year(),
                        last_day.format("%b"),
                        last_day.day(),
                        last_day.year()
                    )
                }
            }
            DateFormat::DMY => {
                // European format: "W48 - 24 - 30 Nov, 2024"
                if first_day.month() == last_day.month() {
                    format!(
                        "W{} - {} - {} {}, {}",
                        week_number,
                        first_day.day(),
                        last_day.day(),
                        first_day.format("%b"),
                        first_day.year()
                    )
                } else if first_day.year() == last_day.year() {
                    format!(
                        "W{} - {} {} - {} {}, {}",
                        week_number,
                        first_day.day(),
                        first_day.format("%b"),
                        last_day.day(),
                        last_day.format("%b"),
                        first_day.year()
                    )
                } else {
                    format!(
                        "W{} - {} {}, {} - {} {}, {}",
                        week_number,
                        first_day.day(),
                        first_day.format("%b"),
                        first_day.year(),
                        last_day.day(),
                        last_day.format("%b"),
                        last_day.year()
                    )
                }
            }
            DateFormat::YMD => {
                // ISO format: "W48 - 2024-11-24 - 2024-11-30"
                if first_day.year() == last_day.year() && first_day.month() == last_day.month() {
                    format!(
                        "W{} - {}-{:02}-{:02} - {:02}",
                        week_number,
                        first_day.year(),
                        first_day.month(),
                        first_day.day(),
                        last_day.day()
                    )
                } else {
                    format!(
                        "W{} - {} - {}",
                        week_number,
                        first_day.format("%Y-%m-%d"),
                        last_day.format("%Y-%m-%d")
                    )
                }
            }
        }
    }

    /// Format a short date for day view header (e.g., "Monday, Nov 24" or "Monday, 24 Nov")
    pub fn format_day_header(&self, date: &chrono::NaiveDate, day_name: &str) -> String {
        match self.date_format {
            DateFormat::MDY => {
                // US format: "Monday, Nov 24"
                format!("{}, {} {}", day_name, date.format("%b"), date.day())
            }
            DateFormat::DMY => {
                // European format: "Monday, 24 Nov"
                format!("{}, {} {}", day_name, date.day(), date.format("%b"))
            }
            DateFormat::YMD => {
                // ISO format: "Monday, 2024-11-24"
                format!("{}, {}", day_name, date.format("%Y-%m-%d"))
            }
        }
    }
}

impl Default for LocalePreferences {
    fn default() -> Self {
        Self::detect_from_system()
    }
}

/// Detect if locale uses 24-hour format
fn detect_24_hour_format(locale: &str) -> bool {
    // Most locales use 24-hour format except:
    // - US (en_US)
    // - UK traditionally uses 12h but transitioning to 24h
    // - Canada (en_CA) uses 12h
    // - Australia (en_AU) traditionally 12h
    // - Philippines (fil_PH, en_PH)

    let locale_lower = locale.to_lowercase();

    // Explicit 12-hour format locales
    let twelve_hour_locales = [
        "en_us", "en_ca", "en_au", "en_nz", "en_ph",
        "fil_ph", "tl_ph"
    ];

    // Check if it's a known 12-hour locale
    for twelve_hour in &twelve_hour_locales {
        if locale_lower.starts_with(twelve_hour) {
            return false;
        }
    }

    // Default to 24-hour for all other locales
    true
}

/// Detect first day of week from locale
fn detect_first_day_of_week(locale: &str) -> chrono::Weekday {
    use chrono::Weekday;

    let locale_lower = locale.to_lowercase();

    // Locales that start week on Sunday
    let sunday_locales = [
        "en_us", "en_ca", "en_au", "en_nz", "en_ph",
        "ja_jp", "ko_kr", "zh_cn", "zh_tw", "zh_hk",
        "he_il", "ar_sa", "ar_ae", "ar_eg",
        "fil_ph", "tl_ph", "pt_br"
    ];

    // Locales that start week on Saturday
    let saturday_locales = [
        "ar_iq", "ar_ly", "ar_om", "ar_qa", "ar_sd",
        "ar_sy", "ar_ye"
    ];

    // Check for Sunday-starting locales
    for sunday_locale in &sunday_locales {
        if locale_lower.starts_with(sunday_locale) {
            return Weekday::Sun;
        }
    }

    // Check for Saturday-starting locales
    for saturday_locale in &saturday_locales {
        if locale_lower.starts_with(saturday_locale) {
            return Weekday::Sat;
        }
    }

    // Default to Monday (ISO 8601 standard) for most European and other locales
    Weekday::Mon
}

/// Detect date format from locale
fn detect_date_format(locale: &str) -> DateFormat {
    let locale_lower = locale.to_lowercase();

    // MDY (Month-Day-Year) - Primarily US
    let mdy_locales = [
        "en_us", "en_ca", "en_ph", "fil_ph", "tl_ph"
    ];

    // YMD (Year-Month-Day) - ISO 8601, East Asian countries
    let ymd_locales = [
        "ja_jp", "ko_kr", "zh_cn", "zh_tw", "zh_hk", "zh_sg",
        "hu_hu", "lt_lt", "mn_mn", "ko_kp"
    ];

    // Check for MDY locales
    for mdy_locale in &mdy_locales {
        if locale_lower.starts_with(mdy_locale) {
            return DateFormat::MDY;
        }
    }

    // Check for YMD locales
    for ymd_locale in &ymd_locales {
        if locale_lower.starts_with(ymd_locale) {
            return DateFormat::YMD;
        }
    }

    // Default to DMY (Day-Month-Year) for most of the world
    // This includes: UK, Europe, Australia, India, Middle East, Africa, South America, etc.
    DateFormat::DMY
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Weekday;

    #[test]
    fn test_24_hour_detection() {
        assert_eq!(detect_24_hour_format("de_DE.UTF-8"), true);
        assert_eq!(detect_24_hour_format("en_GB.UTF-8"), true);
        assert_eq!(detect_24_hour_format("fr_FR.UTF-8"), true);
        assert_eq!(detect_24_hour_format("en_US.UTF-8"), false);
        assert_eq!(detect_24_hour_format("en_CA.UTF-8"), false);
    }

    #[test]
    fn test_first_day_detection() {
        assert_eq!(detect_first_day_of_week("de_DE.UTF-8"), Weekday::Mon);
        assert_eq!(detect_first_day_of_week("en_GB.UTF-8"), Weekday::Mon);
        assert_eq!(detect_first_day_of_week("en_US.UTF-8"), Weekday::Sun);
        assert_eq!(detect_first_day_of_week("ja_JP.UTF-8"), Weekday::Sun);
        assert_eq!(detect_first_day_of_week("ar_SA.UTF-8"), Weekday::Sun);
    }

    #[test]
    fn test_hour_formatting() {
        let locale_24h = LocalePreferences {
            use_24_hour: true,
            first_day_of_week: Weekday::Mon,
            date_format: DateFormat::DMY,
            locale_string: "de_DE.UTF-8".to_string(),
        };

        assert_eq!(locale_24h.format_hour(0), "00:00");
        assert_eq!(locale_24h.format_hour(13), "13:00");
        assert_eq!(locale_24h.format_hour(23), "23:00");

        let locale_12h = LocalePreferences {
            use_24_hour: false,
            first_day_of_week: Weekday::Sun,
            date_format: DateFormat::MDY,
            locale_string: "en_US.UTF-8".to_string(),
        };

        assert_eq!(locale_12h.format_hour(0), "12 AM");
        assert_eq!(locale_12h.format_hour(1), "1 AM");
        assert_eq!(locale_12h.format_hour(12), "12 PM");
        assert_eq!(locale_12h.format_hour(13), "1 PM");
    }

    #[test]
    fn test_date_format_detection() {
        assert_eq!(detect_date_format("en_US.UTF-8"), DateFormat::MDY);
        assert_eq!(detect_date_format("en_GB.UTF-8"), DateFormat::DMY);
        assert_eq!(detect_date_format("de_DE.UTF-8"), DateFormat::DMY);
        assert_eq!(detect_date_format("ja_JP.UTF-8"), DateFormat::YMD);
        assert_eq!(detect_date_format("zh_CN.UTF-8"), DateFormat::YMD);
        assert_eq!(detect_date_format("ko_KR.UTF-8"), DateFormat::YMD);
    }
}
