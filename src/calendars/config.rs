use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::PathBuf;

/// Configuration for a calendar source
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalendarConfig {
    pub id: String,
    pub name: String,
    pub color: String,
    pub enabled: bool,
    pub calendar_type: String,
}

/// Manager configuration that stores all calendar settings
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CalendarManagerConfig {
    pub calendars: Vec<CalendarConfig>,
}

impl CalendarManagerConfig {
    /// Load configuration from disk
    pub fn load() -> Result<Self, io::Error> {
        let path = Self::config_path();

        if !path.exists() {
            return Ok(Self::default());
        }

        let contents = fs::read_to_string(&path)?;
        let config: CalendarManagerConfig = serde_json::from_str(&contents)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        Ok(config)
    }

    /// Save configuration to disk
    pub fn save(&self) -> Result<(), io::Error> {
        let path = Self::config_path();

        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let json = serde_json::to_string_pretty(self)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        fs::write(&path, json)?;
        Ok(())
    }

    /// Get the configuration file path
    fn config_path() -> PathBuf {
        let mut path = dirs::config_local_dir().unwrap_or_else(|| PathBuf::from("."));
        path.push("sol-calendar");
        path.push("calendars.json");
        path
    }

    /// Update or add a calendar configuration
    pub fn update_calendar(&mut self, config: CalendarConfig) {
        if let Some(existing) = self.calendars.iter_mut().find(|c| c.id == config.id) {
            *existing = config;
        } else {
            self.calendars.push(config);
        }
    }

    /// Get a calendar configuration by ID
    pub fn get_calendar(&self, id: &str) -> Option<&CalendarConfig> {
        self.calendars.iter().find(|c| c.id == id)
    }

    /// Remove a calendar configuration
    pub fn remove_calendar(&mut self, id: &str) -> bool {
        if let Some(index) = self.calendars.iter().position(|c| c.id == id) {
            self.calendars.remove(index);
            true
        } else {
            false
        }
    }
}
