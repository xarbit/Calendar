//! Local protocol implementation using SQLite database.
//!
//! This protocol stores events in a local SQLite database.
//! It's the default protocol for local calendars.

use std::sync::{Arc, Mutex};

use crate::caldav::CalendarEvent;
use crate::database::Database;
use super::{Protocol, ProtocolResult};

/// Local protocol using SQLite database for event storage.
#[derive(Debug)]
pub struct LocalProtocol {
    /// Shared database connection
    db: Arc<Mutex<Database>>,
}

impl LocalProtocol {
    /// Create a new LocalProtocol with a shared database connection
    pub fn new(db: Arc<Mutex<Database>>) -> Self {
        LocalProtocol { db }
    }
}

impl Protocol for LocalProtocol {
    fn fetch_events(&self, calendar_id: &str) -> ProtocolResult<Vec<CalendarEvent>> {
        let db = self.db.lock().map_err(|e| format!("Database lock error: {}", e))?;
        db.get_events_for_calendar(calendar_id)
    }

    fn add_event(&mut self, calendar_id: &str, event: &CalendarEvent) -> ProtocolResult<()> {
        let db = self.db.lock().map_err(|e| format!("Database lock error: {}", e))?;
        db.insert_event(calendar_id, event)
    }

    fn update_event(&mut self, _calendar_id: &str, event: &CalendarEvent) -> ProtocolResult<()> {
        let db = self.db.lock().map_err(|e| format!("Database lock error: {}", e))?;
        db.update_event(event)
    }

    fn delete_event(&mut self, _calendar_id: &str, uid: &str) -> ProtocolResult<bool> {
        let db = self.db.lock().map_err(|e| format!("Database lock error: {}", e))?;
        db.delete_event(uid)
    }

    fn sync(&mut self, _calendar_id: &str) -> ProtocolResult<()> {
        // Local protocol doesn't need to sync - data is always fresh from DB
        Ok(())
    }

    fn requires_network(&self) -> bool {
        false
    }

    fn protocol_type(&self) -> &'static str {
        "local"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::caldav::{AlertTime, RepeatFrequency, TravelTime};
    use chrono::{TimeZone, Utc};

    fn create_test_event(uid: &str, summary: &str) -> CalendarEvent {
        CalendarEvent {
            uid: uid.to_string(),
            summary: summary.to_string(),
            location: None,
            all_day: false,
            start: Utc.with_ymd_and_hms(2025, 11, 30, 10, 0, 0).unwrap(),
            end: Utc.with_ymd_and_hms(2025, 11, 30, 11, 0, 0).unwrap(),
            travel_time: TravelTime::None,
            repeat: RepeatFrequency::Never,
            invitees: vec![],
            alert: AlertTime::None,
            alert_second: None,
            attachments: vec![],
            url: None,
            notes: None,
        }
    }

    fn setup_protocol() -> (LocalProtocol, std::path::PathBuf) {
        let temp_dir = std::env::temp_dir();
        let db_path = temp_dir.join(format!("sol_protocol_test_{}.db", std::process::id()));
        let _ = std::fs::remove_file(&db_path);

        let db = Database::open_at(db_path.clone()).unwrap();
        let db = Arc::new(Mutex::new(db));
        (LocalProtocol::new(db), db_path)
    }

    // ==================== Basic Protocol Tests ====================

    #[test]
    fn test_local_protocol() {
        let temp_dir = std::env::temp_dir();
        let db_path = temp_dir.join("sol_protocol_test.db");
        let _ = std::fs::remove_file(&db_path);

        let db = Database::open_at(db_path.clone()).unwrap();
        let db = Arc::new(Mutex::new(db));
        let mut protocol = LocalProtocol::new(db);

        // Create an event
        let event = CalendarEvent {
            uid: "protocol-test-1".to_string(),
            summary: "Protocol Test".to_string(),
            location: None,
            all_day: false,
            start: Utc.with_ymd_and_hms(2025, 11, 30, 10, 0, 0).unwrap(),
            end: Utc.with_ymd_and_hms(2025, 11, 30, 11, 0, 0).unwrap(),
            travel_time: TravelTime::None,
            repeat: RepeatFrequency::Never,
            invitees: vec![],
            alert: AlertTime::None,
            alert_second: None,
            attachments: vec![],
            url: None,
            notes: None,
        };

        // Add event
        protocol.add_event("test-cal", &event).unwrap();

        // Fetch events
        let events = protocol.fetch_events("test-cal").unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].summary, "Protocol Test");

        // Delete event
        let deleted = protocol.delete_event("test-cal", "protocol-test-1").unwrap();
        assert!(deleted);

        // Verify deletion
        let events = protocol.fetch_events("test-cal").unwrap();
        assert_eq!(events.len(), 0);

        // Clean up
        let _ = std::fs::remove_file(&db_path);
    }

    // ==================== Protocol Trait Tests ====================

    #[test]
    fn test_protocol_type() {
        let (protocol, db_path) = setup_protocol();
        assert_eq!(protocol.protocol_type(), "local");
        let _ = std::fs::remove_file(&db_path);
    }

    #[test]
    fn test_requires_network() {
        let (protocol, db_path) = setup_protocol();
        assert!(!protocol.requires_network());
        let _ = std::fs::remove_file(&db_path);
    }

    #[test]
    fn test_sync_local_protocol() {
        let (mut protocol, db_path) = setup_protocol();
        // Sync should be a no-op for local protocol
        let result = protocol.sync("test-cal");
        assert!(result.is_ok());
        let _ = std::fs::remove_file(&db_path);
    }

    // ==================== Update Event Tests ====================

    #[test]
    fn test_update_event() {
        let (mut protocol, db_path) = setup_protocol();

        let mut event = create_test_event("update-1", "Original");
        protocol.add_event("cal1", &event).unwrap();

        // Update
        event.summary = "Updated".to_string();
        event.location = Some("New Location".to_string());
        protocol.update_event("cal1", &event).unwrap();

        let events = protocol.fetch_events("cal1").unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].summary, "Updated");
        assert_eq!(events[0].location, Some("New Location".to_string()));

        let _ = std::fs::remove_file(&db_path);
    }

    // ==================== Multiple Events Tests ====================

    #[test]
    fn test_multiple_events() {
        let (mut protocol, db_path) = setup_protocol();

        for i in 1..=5 {
            let event = create_test_event(&format!("event-{}", i), &format!("Event {}", i));
            protocol.add_event("cal1", &event).unwrap();
        }

        let events = protocol.fetch_events("cal1").unwrap();
        assert_eq!(events.len(), 5);

        let _ = std::fs::remove_file(&db_path);
    }

    #[test]
    fn test_multiple_calendars() {
        let (mut protocol, db_path) = setup_protocol();

        protocol.add_event("work", &create_test_event("w1", "Work 1")).unwrap();
        protocol.add_event("work", &create_test_event("w2", "Work 2")).unwrap();
        protocol.add_event("personal", &create_test_event("p1", "Personal")).unwrap();

        assert_eq!(protocol.fetch_events("work").unwrap().len(), 2);
        assert_eq!(protocol.fetch_events("personal").unwrap().len(), 1);
        assert_eq!(protocol.fetch_events("other").unwrap().len(), 0);

        let _ = std::fs::remove_file(&db_path);
    }

    // ==================== Delete Tests ====================

    #[test]
    fn test_delete_nonexistent() {
        let (mut protocol, db_path) = setup_protocol();

        let deleted = protocol.delete_event("cal1", "nonexistent").unwrap();
        assert!(!deleted);

        let _ = std::fs::remove_file(&db_path);
    }

    // ==================== Debug Tests ====================

    #[test]
    fn test_local_protocol_debug() {
        let (protocol, db_path) = setup_protocol();
        let debug_str = format!("{:?}", protocol);
        assert!(debug_str.contains("LocalProtocol"));
        let _ = std::fs::remove_file(&db_path);
    }

    // ==================== Unicode Tests ====================

    #[test]
    fn test_unicode_events() {
        let (mut protocol, db_path) = setup_protocol();

        let event = CalendarEvent {
            uid: "unicode-1".to_string(),
            summary: "会议 📅".to_string(),
            location: Some("北京".to_string()),
            all_day: false,
            start: Utc.with_ymd_and_hms(2025, 12, 1, 10, 0, 0).unwrap(),
            end: Utc.with_ymd_and_hms(2025, 12, 1, 11, 0, 0).unwrap(),
            travel_time: TravelTime::None,
            repeat: RepeatFrequency::Never,
            invitees: vec![],
            alert: AlertTime::None,
            alert_second: None,
            attachments: vec![],
            url: None,
            notes: Some("备注".to_string()),
        };

        protocol.add_event("cal1", &event).unwrap();

        let events = protocol.fetch_events("cal1").unwrap();
        assert_eq!(events[0].summary, "会议 📅");
        assert_eq!(events[0].location, Some("北京".to_string()));

        let _ = std::fs::remove_file(&db_path);
    }
}
