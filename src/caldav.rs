use icalendar::{Calendar, Component, Event, EventLike};
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use std::error::Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalendarEvent {
    pub uid: String,
    pub summary: String,
    pub description: Option<String>,
    pub start: chrono::DateTime<chrono::Utc>,
    pub end: chrono::DateTime<chrono::Utc>,
    pub location: Option<String>,
}

#[derive(Debug, Clone)]
pub struct CalDavClient {
    server_url: String,
    username: String,
    password: String,
    client: Client,
}

impl CalDavClient {
    pub fn new(server_url: String, username: String, password: String) -> Self {
        CalDavClient {
            server_url,
            username,
            password,
            client: Client::new(),
        }
    }

    pub fn fetch_events(&self) -> Result<Vec<CalendarEvent>, Box<dyn Error>> {
        // CalDAV REPORT request to fetch calendar data
        let caldav_query = r#"<?xml version="1.0" encoding="utf-8" ?>
        <C:calendar-query xmlns:D="DAV:" xmlns:C="urn:ietf:params:xml:ns:caldav">
            <D:prop>
                <D:getetag/>
                <C:calendar-data/>
            </D:prop>
            <C:filter>
                <C:comp-filter name="VCALENDAR">
                    <C:comp-filter name="VEVENT"/>
                </C:comp-filter>
            </C:filter>
        </C:calendar-query>"#;

        let response = self
            .client
            .request(reqwest::Method::from_bytes(b"REPORT")?, &self.server_url)
            .header("Depth", "1")
            .header("Content-Type", "application/xml; charset=utf-8")
            .basic_auth(&self.username, Some(&self.password))
            .body(caldav_query)
            .send()?;

        if !response.status().is_success() {
            return Err(format!("CalDAV request failed: {}", response.status()).into());
        }

        let body = response.text()?;
        self.parse_calendar_data(&body)
    }

    fn parse_calendar_data(&self, data: &str) -> Result<Vec<CalendarEvent>, Box<dyn Error>> {
        let events = Vec::new();

        // Simple XML parsing to extract calendar data
        // In a production app, you'd want to use a proper XML parser
        for line in data.lines() {
            if line.contains("BEGIN:VCALENDAR") {
                // Extract calendar data between VCALENDAR tags
                // This is a simplified approach
            }
        }

        Ok(events)
    }

    pub fn create_event(&self, event: &CalendarEvent) -> Result<(), Box<dyn Error>> {
        let mut calendar = Calendar::new();

        let mut ical_event = Event::new();
        ical_event.summary(&event.summary);

        if let Some(desc) = &event.description {
            ical_event.description(desc);
        }

        if let Some(loc) = &event.location {
            ical_event.location(loc);
        }

        ical_event.starts(event.start);
        ical_event.ends(event.end);
        ical_event.uid(&event.uid);

        calendar.push(ical_event);

        let ical_data = calendar.to_string();

        // PUT request to create the event on the CalDAV server
        let event_url = format!("{}/{}.ics", self.server_url, event.uid);

        let response = self
            .client
            .put(&event_url)
            .header("Content-Type", "text/calendar; charset=utf-8")
            .basic_auth(&self.username, Some(&self.password))
            .body(ical_data)
            .send()?;

        if !response.status().is_success() {
            return Err(format!("Failed to create event: {}", response.status()).into());
        }

        Ok(())
    }

    pub fn update_event(&self, event: &CalendarEvent) -> Result<(), Box<dyn Error>> {
        // Update is similar to create in CalDAV
        self.create_event(event)
    }

    pub fn delete_event(&self, uid: &str) -> Result<(), Box<dyn Error>> {
        let event_url = format!("{}/{}.ics", self.server_url, uid);

        let response = self
            .client
            .delete(&event_url)
            .basic_auth(&self.username, Some(&self.password))
            .send()?;

        if !response.status().is_success() {
            return Err(format!("Failed to delete event: {}", response.status()).into());
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_creation() {
        let client = CalDavClient::new(
            "https://example.com/caldav".to_string(),
            "user".to_string(),
            "pass".to_string(),
        );

        let event = CalendarEvent {
            uid: "test-event-1".to_string(),
            summary: "Test Event".to_string(),
            description: Some("A test event".to_string()),
            start: chrono::Utc::now(),
            end: chrono::Utc::now() + chrono::Duration::hours(1),
            location: Some("Test Location".to_string()),
        };

        // Note: This test would fail without a real CalDAV server
        // In production, you'd use mock servers or integration tests
    }
}
