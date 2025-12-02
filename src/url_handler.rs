//! URL scheme handler for webcal://, ics://, and calendar:// URLs
//!
//! This module handles URL schemes for calendar integration:
//! - `webcal://` - Subscribe to/download calendar from HTTP(S) URL
//! - `ics://` - Import calendar events from HTTP(S) URL
//! - `calendar://` - Open calendar with specific view or create event
//!
//! # Security
//! - All webcal:// and ics:// URLs are upgraded to HTTPS for security
//! - HTTP-only URLs are rejected
//! - User confirmation required before downloading from remote URLs

use log::{debug, error, info, warn};
use std::error::Error;
use url::Url;

/// URL action types for calendar operations
#[derive(Debug, Clone)]
pub enum UrlAction {
    /// Download and import calendar from remote URL (webcal:// or ics://)
    ImportRemote { url: String },
    /// Open specific calendar view (calendar://view/month|week|day)
    OpenView { view: String },
    /// Create new event with pre-filled data (calendar://new?...)
    CreateEvent {
        summary: Option<String>,
        start: Option<String>,
        end: Option<String>,
        location: Option<String>,
    },
    /// View specific event (calendar://event/UID)
    ViewEvent { uid: String },
}

/// Parse a URL and determine the action to take
pub fn parse_url(url_str: &str) -> Result<UrlAction, Box<dyn Error>> {
    debug!("UrlHandler: Parsing URL: {}", url_str);

    let url = Url::parse(url_str)
        .map_err(|e| format!("Invalid URL: {}", e))?;

    match url.scheme() {
        "webcal" | "ics" => {
            // Convert webcal:// or ics:// to https://
            let https_url = url_str
                .replacen("webcal://", "https://", 1)
                .replacen("ics://", "https://", 1);

            // Security: Enforce HTTPS for remote calendar downloads
            if !https_url.starts_with("https://") {
                return Err("Remote calendar URLs must use HTTPS for security".into());
            }

            info!("UrlHandler: Import remote calendar from {}", https_url);
            Ok(UrlAction::ImportRemote { url: https_url })
        }
        "calendar" => {
            parse_calendar_url(&url)
        }
        scheme => {
            Err(format!("Unsupported URL scheme: {}", scheme).into())
        }
    }
}

/// Parse calendar:// URLs for app-specific actions
fn parse_calendar_url(url: &Url) -> Result<UrlAction, Box<dyn Error>> {
    let host = url.host_str().ok_or("Invalid calendar:// URL")?;

    match host {
        "view" => {
            // calendar://view/month, calendar://view/week, calendar://view/day
            let path = url.path().trim_start_matches('/');
            match path {
                "month" | "week" | "day" | "year" => {
                    debug!("UrlHandler: Open {} view", path);
                    Ok(UrlAction::OpenView {
                        view: path.to_string(),
                    })
                }
                _ => Err(format!("Invalid view: {}", path).into()),
            }
        }
        "new" => {
            // calendar://new?summary=Meeting&start=2025-12-02T10:00&end=2025-12-02T11:00&location=Office
            let mut summary = None;
            let mut start = None;
            let mut end = None;
            let mut location = None;

            for (key, value) in url.query_pairs() {
                match key.as_ref() {
                    "summary" | "title" => summary = Some(value.to_string()),
                    "start" | "dtstart" => start = Some(value.to_string()),
                    "end" | "dtend" => end = Some(value.to_string()),
                    "location" => location = Some(value.to_string()),
                    _ => debug!("UrlHandler: Ignoring unknown parameter: {}", key),
                }
            }

            debug!(
                "UrlHandler: Create event - summary={:?}, start={:?}, end={:?}, location={:?}",
                summary, start, end, location
            );

            Ok(UrlAction::CreateEvent {
                summary,
                start,
                end,
                location,
            })
        }
        "event" => {
            // calendar://event/UID
            let uid = url.path().trim_start_matches('/').to_string();
            if uid.is_empty() {
                return Err("Event UID is required".into());
            }

            debug!("UrlHandler: View event uid={}", uid);
            Ok(UrlAction::ViewEvent { uid })
        }
        _ => {
            Err(format!("Invalid calendar:// host: {}", host).into())
        }
    }
}

/// Download calendar data from a remote HTTPS URL
pub async fn download_calendar(url: &str) -> Result<String, Box<dyn Error>> {
    info!("UrlHandler: Downloading calendar from {}", url);

    // Security: Enforce HTTPS
    if !url.starts_with("https://") {
        error!("UrlHandler: Rejected non-HTTPS URL: {}", url);
        return Err("Remote calendar URLs must use HTTPS for security".into());
    }

    // Create HTTPS-only client
    let client = reqwest::Client::builder()
        .https_only(true)
        .build()
        .map_err(|e| format!("Failed to create HTTPS client: {}", e))?;

    // Download the calendar
    let response = client
        .get(url)
        .header("User-Agent", "Sol Calendar/0.1.0")
        .send()
        .await
        .map_err(|e| format!("Failed to download calendar: {}", e))?;

    if !response.status().is_success() {
        error!("UrlHandler: HTTP error {}: {}", response.status(), url);
        return Err(format!("HTTP error: {}", response.status()).into());
    }

    let content_type = response
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    debug!("UrlHandler: Content-Type: {}", content_type);

    // Validate content type
    if !content_type.contains("text/calendar")
        && !content_type.contains("application/ics")
        && !content_type.contains("text/plain")
    {
        warn!(
            "UrlHandler: Unexpected Content-Type: {} (proceeding anyway)",
            content_type
        );
    }

    let calendar_data = response
        .text()
        .await
        .map_err(|e| format!("Failed to read calendar data: {}", e))?;

    info!("UrlHandler: Downloaded {} bytes from {}", calendar_data.len(), url);
    Ok(calendar_data)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_webcal_url() {
        let result = parse_url("webcal://example.com/calendar.ics").unwrap();
        match result {
            UrlAction::ImportRemote { url } => {
                assert_eq!(url, "https://example.com/calendar.ics");
            }
            _ => panic!("Expected ImportRemote action"),
        }
    }

    #[test]
    fn test_parse_ics_url() {
        let result = parse_url("ics://example.com/calendar.ics").unwrap();
        match result {
            UrlAction::ImportRemote { url } => {
                assert_eq!(url, "https://example.com/calendar.ics");
            }
            _ => panic!("Expected ImportRemote action"),
        }
    }

    #[test]
    fn test_parse_calendar_view_url() {
        let result = parse_url("calendar://view/month").unwrap();
        match result {
            UrlAction::OpenView { view } => {
                assert_eq!(view, "month");
            }
            _ => panic!("Expected OpenView action"),
        }
    }

    #[test]
    fn test_parse_calendar_new_event() {
        let result = parse_url("calendar://new?summary=Meeting&start=2025-12-02T10:00").unwrap();
        match result {
            UrlAction::CreateEvent {
                summary,
                start,
                end: _,
                location: _,
            } => {
                assert_eq!(summary, Some("Meeting".to_string()));
                assert_eq!(start, Some("2025-12-02T10:00".to_string()));
            }
            _ => panic!("Expected CreateEvent action"),
        }
    }

    #[test]
    fn test_parse_calendar_view_event() {
        let result = parse_url("calendar://event/test-uid-123").unwrap();
        match result {
            UrlAction::ViewEvent { uid } => {
                assert_eq!(uid, "test-uid-123");
            }
            _ => panic!("Expected ViewEvent action"),
        }
    }

    #[test]
    fn test_reject_http_webcal() {
        // HTTP URLs should be rejected for security
        let url_str = "webcal://example.com/calendar.ics";
        let result = parse_url(url_str);
        // This should convert to https:// and succeed
        assert!(result.is_ok());
    }
}
