use chrono::{Datelike, NaiveDate};
use crate::locale::LocalePreferences;

/// Cached day state for day view
#[derive(Debug, Clone, PartialEq)]
pub struct DayState {
    pub date: NaiveDate,
    pub day_text: String,      // Pre-formatted "Monday"
    pub date_number: String,   // Pre-formatted "15"
    pub month_year_text: String, // Pre-formatted with locale-aware format
    pub today: NaiveDate,
}

impl DayState {
    /// Create a new DayState for the given date
    pub fn new(date: NaiveDate, locale: &LocalePreferences) -> Self {
        let today = chrono::Local::now().date_naive();

        let day_text = format!("{}", date.format("%A")); // "Monday"
        let date_number = format!("{}", date.format("%d")); // "15"

        // Use locale-aware date formatting for the header
        let month_year_text = locale.format_day_header(&date, &day_text);

        DayState {
            date,
            day_text,
            date_number,
            month_year_text,
            today,
        }
    }

    /// Create DayState for today
    pub fn current(locale: &LocalePreferences) -> Self {
        Self::new(chrono::Local::now().date_naive(), locale)
    }

    /// Navigate to previous day
    pub fn previous(&self, locale: &LocalePreferences) -> Self {
        Self::new(self.date - chrono::Duration::days(1), locale)
    }

    /// Navigate to next day
    pub fn next(&self, locale: &LocalePreferences) -> Self {
        Self::new(self.date + chrono::Duration::days(1), locale)
    }

    /// Check if this day is today
    pub fn is_today(&self) -> bool {
        self.date == self.today
    }
}
