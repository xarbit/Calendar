//! Protocol layer for calendar event storage and synchronization.
//!
//! This module provides protocol implementations for different storage backends.
//! Each protocol handles the actual read/write operations for events.
//!
//! # Architecture
//!
//! ```text
//! EventHandler (middleware)
//!       │
//!       ▼
//! Protocol Trait
//!       │
//!       ├── LocalProtocol (SQLite database)
//!       └── CalDavProtocol (HTTP/CalDAV server)
//! ```
//!
//! # Adding a New Protocol
//!
//! 1. Create a new file in `src/protocols/` (e.g., `google.rs`)
//! 2. Implement the `Protocol` trait
//! 3. Re-export from this module

mod local;
mod caldav;

// Internal use only - LocalProtocol used in tests, CalDavProtocol for future remote calendar support
#[allow(unused_imports)]
pub(crate) use local::LocalProtocol;

use crate::caldav::CalendarEvent;
use std::error::Error;

/// Result type for protocol operations
pub type ProtocolResult<T> = Result<T, Box<dyn Error>>;

/// Protocol trait for calendar event storage backends.
///
/// This trait abstracts the storage mechanism, allowing the EventHandler
/// to work with different backends (local SQLite, CalDAV, Google Calendar, etc.)
/// without knowing the implementation details.
pub trait Protocol: std::fmt::Debug + Send {
    /// Fetch all events from this protocol/storage
    fn fetch_events(&self, calendar_id: &str) -> ProtocolResult<Vec<CalendarEvent>>;

    /// Add a new event
    fn add_event(&mut self, calendar_id: &str, event: &CalendarEvent) -> ProtocolResult<()>;

    /// Update an existing event
    fn update_event(&mut self, calendar_id: &str, event: &CalendarEvent) -> ProtocolResult<()>;

    /// Delete an event by UID
    fn delete_event(&mut self, calendar_id: &str, uid: &str) -> ProtocolResult<bool>;

    /// Sync with remote (no-op for local, fetches latest for remote)
    fn sync(&mut self, calendar_id: &str) -> ProtocolResult<()>;

    /// Check if this protocol supports write operations
    fn supports_write(&self) -> bool {
        true
    }

    /// Check if this protocol requires network connectivity
    fn requires_network(&self) -> bool {
        false
    }

    /// Get the protocol type name (for debugging/logging)
    fn protocol_type(&self) -> &'static str;
}
