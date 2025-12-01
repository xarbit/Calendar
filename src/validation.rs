//! Validation logic for application data
//!
//! This module contains pure validation functions that are separate from
//! UI and state management concerns.
//!
//! These functions are a foundation for future refactoring to move validation
//! logic out of update handlers.

// Allow unused for now - these are foundation functions for future refactoring
#![allow(dead_code)]

use chrono::NaiveDate;

/// Validate and parse a date string in YYYY-MM-DD format
pub fn parse_date(input: &str) -> Option<NaiveDate> {
    NaiveDate::parse_from_str(input, "%Y-%m-%d").ok()
}

/// Validate that an event title is not empty
pub fn validate_event_title(title: &str) -> bool {
    !title.trim().is_empty()
}

/// Validate an email address (basic check)
pub fn validate_email(email: &str) -> bool {
    let email = email.trim();
    !email.is_empty() && email.contains('@') && email.contains('.')
}

/// Ensure end date is not before start date, adjusting if necessary
pub fn ensure_end_after_start(start: NaiveDate, end: NaiveDate) -> NaiveDate {
    if end < start {
        start
    } else {
        end
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== parse_date Tests ====================

    #[test]
    fn test_parse_date_valid() {
        assert!(parse_date("2024-01-15").is_some());
        assert!(parse_date("2024-12-31").is_some());
        assert!(parse_date("2024-02-29").is_some()); // Leap year
        assert!(parse_date("1999-01-01").is_some());
        assert!(parse_date("2099-12-31").is_some());
    }

    #[test]
    fn test_parse_date_invalid_format() {
        assert!(parse_date("invalid").is_none());
        assert!(parse_date("01-15-2024").is_none()); // US format
        assert!(parse_date("15/01/2024").is_none()); // European format
        assert!(parse_date("2024/01/15").is_none()); // Slash separator
        assert!(parse_date("").is_none());
        assert!(parse_date("   ").is_none());
    }

    #[test]
    fn test_parse_date_invalid_values() {
        assert!(parse_date("2024-13-01").is_none()); // Invalid month
        assert!(parse_date("2024-00-01").is_none()); // Month 0
        assert!(parse_date("2024-01-32").is_none()); // Day 32
        assert!(parse_date("2024-02-30").is_none()); // Feb 30
        assert!(parse_date("2023-02-29").is_none()); // Not a leap year
    }

    #[test]
    fn test_parse_date_returns_correct_value() {
        let date = parse_date("2024-06-15").unwrap();
        assert_eq!(date.year(), 2024);
        assert_eq!(date.month(), 6);
        assert_eq!(date.day(), 15);
    }

    // ==================== validate_event_title Tests ====================

    #[test]
    fn test_validate_event_title_valid() {
        assert!(validate_event_title("Meeting"));
        assert!(validate_event_title("A")); // Single char
        assert!(validate_event_title("会议")); // Chinese
        assert!(validate_event_title("Meeting 📅")); // With emoji
        assert!(validate_event_title("  Meeting  ")); // Leading/trailing whitespace (trimmed)
    }

    #[test]
    fn test_validate_event_title_invalid() {
        assert!(!validate_event_title(""));
        assert!(!validate_event_title("   "));
        assert!(!validate_event_title("\t"));
        assert!(!validate_event_title("\n"));
        assert!(!validate_event_title("\t\n  \t"));
    }

    #[test]
    fn test_validate_event_title_edge_cases() {
        assert!(validate_event_title("Line1\nLine2")); // Multiline
        assert!(validate_event_title("Very long title ".repeat(100).as_str())); // Long title
        assert!(validate_event_title("Special chars: !@#$%^&*()"));
    }

    // ==================== validate_email Tests ====================

    #[test]
    fn test_validate_email_valid() {
        assert!(validate_email("test@example.com"));
        assert!(validate_email("user.name@domain.co.uk"));
        assert!(validate_email("a@b.c"));
        assert!(validate_email("  test@example.com  ")); // With whitespace (trimmed)
        assert!(validate_email("user+tag@example.com")); // Plus addressing
    }

    #[test]
    fn test_validate_email_invalid() {
        assert!(!validate_email("invalid"));
        assert!(!validate_email(""));
        assert!(!validate_email("   "));
        assert!(!validate_email("@example.com")); // Missing local part
        assert!(!validate_email("test@")); // Missing domain
        assert!(!validate_email("test@example")); // Missing TLD dot
        assert!(!validate_email("testexample.com")); // Missing @
    }

    #[test]
    fn test_validate_email_edge_cases() {
        // Note: This is a basic validator, not RFC 5322 compliant
        assert!(validate_email("a.b@c.d")); // Minimal valid
        assert!(!validate_email("test@localhost")); // No dot in domain (fails basic check)
    }

    // ==================== ensure_end_after_start Tests ====================

    #[test]
    fn test_ensure_end_after_start_valid() {
        let start = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let end = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
        assert_eq!(ensure_end_after_start(start, end), end);
    }

    #[test]
    fn test_ensure_end_after_start_same_day() {
        let date = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        assert_eq!(ensure_end_after_start(date, date), date);
    }

    #[test]
    fn test_ensure_end_after_start_end_before_start() {
        let start = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
        let end = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        // When end < start, should return start
        assert_eq!(ensure_end_after_start(start, end), start);
    }

    #[test]
    fn test_ensure_end_after_start_cross_year() {
        let start = NaiveDate::from_ymd_opt(2024, 12, 31).unwrap();
        let end = NaiveDate::from_ymd_opt(2025, 1, 1).unwrap();
        assert_eq!(ensure_end_after_start(start, end), end);
    }

    #[test]
    fn test_ensure_end_after_start_cross_year_invalid() {
        let start = NaiveDate::from_ymd_opt(2025, 1, 1).unwrap();
        let end = NaiveDate::from_ymd_opt(2024, 12, 31).unwrap();
        assert_eq!(ensure_end_after_start(start, end), start);
    }
}
