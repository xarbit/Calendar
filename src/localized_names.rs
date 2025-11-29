/// Get localized weekday and month names using the fl! macro

use crate::fl;

/// Get full weekday names (Monday through Sunday)
pub fn get_weekday_names_full() -> [String; 7] {
    [
        fl!("day-monday"),
        fl!("day-tuesday"),
        fl!("day-wednesday"),
        fl!("day-thursday"),
        fl!("day-friday"),
        fl!("day-saturday"),
        fl!("day-sunday"),
    ]
}

/// Get abbreviated weekday names
pub fn get_weekday_names_short() -> [String; 7] {
    [
        fl!("day-mon"),
        fl!("day-tue"),
        fl!("day-wed"),
        fl!("day-thu"),
        fl!("day-fri"),
        fl!("day-sat"),
        fl!("day-sun"),
    ]
}

/// Get month names
pub fn get_month_names() -> [String; 12] {
    [
        fl!("month-january"),
        fl!("month-february"),
        fl!("month-march"),
        fl!("month-april"),
        fl!("month-may"),
        fl!("month-june"),
        fl!("month-july"),
        fl!("month-august"),
        fl!("month-september"),
        fl!("month-october"),
        fl!("month-november"),
        fl!("month-december"),
    ]
}

/// Get a specific month name by number (1-12)
pub fn get_month_name(month: u32) -> String {
    let months = get_month_names();
    if month >= 1 && month <= 12 {
        months[(month - 1) as usize].clone()
    } else {
        String::new()
    }
}

/// Get abbreviated weekday name by chrono::Weekday
pub fn get_weekday_short(weekday: chrono::Weekday) -> String {
    let short_names = get_weekday_names_short();
    // Monday = 0, ..., Sunday = 6 in chrono's num_days_from_monday()
    short_names[weekday.num_days_from_monday() as usize].clone()
}

/// Get full weekday name by chrono::Weekday
pub fn get_weekday_full(weekday: chrono::Weekday) -> String {
    let full_names = get_weekday_names_full();
    // Monday = 0, ..., Sunday = 6 in chrono's num_days_from_monday()
    full_names[weekday.num_days_from_monday() as usize].clone()
}
