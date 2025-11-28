use crate::caldav::CalendarEvent;
use chrono::Datelike;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalStorage {
    events: Vec<CalendarEvent>,
}

impl LocalStorage {
    pub fn new() -> Self {
        LocalStorage {
            events: Vec::new(),
        }
    }

    pub fn load_from_file(path: &PathBuf) -> Result<Self, io::Error> {
        let contents = fs::read_to_string(path)?;
        let storage: LocalStorage = serde_json::from_str(&contents)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        Ok(storage)
    }

    pub fn save_to_file(&self, path: &PathBuf) -> Result<(), io::Error> {
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        fs::write(path, json)?;
        Ok(())
    }

    pub fn add_event(&mut self, event: CalendarEvent) {
        self.events.push(event);
    }

    pub fn remove_event(&mut self, uid: &str) {
        self.events.retain(|e| e.uid != uid);
    }

    pub fn update_event(&mut self, event: CalendarEvent) {
        if let Some(existing) = self.events.iter_mut().find(|e| e.uid == event.uid) {
            *existing = event;
        } else {
            self.add_event(event);
        }
    }

    pub fn get_events(&self) -> &[CalendarEvent] {
        &self.events
    }

    pub fn get_events_for_date(&self, date: chrono::NaiveDate) -> Vec<&CalendarEvent> {
        self.events
            .iter()
            .filter(|e| {
                let event_date = e.start.date_naive();
                event_date == date
            })
            .collect()
    }

    pub fn get_events_for_month(&self, year: i32, month: u32) -> Vec<&CalendarEvent> {
        self.events
            .iter()
            .filter(|e| {
                let event_date = e.start.date_naive();
                event_date.year() == year && event_date.month() == month
            })
            .collect()
    }

    pub fn sync_with_caldav(&mut self, caldav_events: Vec<CalendarEvent>) {
        // Simple sync: replace all local events with CalDAV events
        // In a production app, you'd want more sophisticated merging logic
        self.events = caldav_events;
    }

    pub fn get_storage_path() -> PathBuf {
        let mut path = dirs::data_local_dir().unwrap_or_else(|| PathBuf::from("."));
        path.push("sol-calendar");
        fs::create_dir_all(&path).ok();
        path.push("events.json");
        path
    }
}

impl Default for LocalStorage {
    fn default() -> Self {
        Self::new()
    }
}
