mod app;
mod cache;
mod caldav;
mod calendars;
mod color_constants;
mod components;
mod database;
#[cfg(debug_assertions)]
mod demo_data;
mod dialogs;
mod keyboard;
mod layout;
mod layout_constants;
mod locale;
mod localize;
mod localized_names;
mod logging;
mod menu_action;
mod message;
mod models;
mod protocols;
mod selection;
mod services;
mod settings;
mod storage;
mod styles;
mod ui_constants;
mod update;
mod validation;
mod views;

use app::CosmicCalendar;
use cosmic::app::Settings;
#[cfg(debug_assertions)]
use database::Database;
use log::info;
#[cfg(debug_assertions)]
use std::env;

/// Application entry point
pub fn main() -> cosmic::iced::Result {
    // Initialize centralized logging
    logging::init();

    info!("Sol Calendar starting...");

    // Development-only CLI flags (only available in debug builds)
    #[cfg(debug_assertions)]
    {
        let args: Vec<String> = env::args().collect();

        // Check for --dev-reset-db flag
        if args.contains(&"--dev-reset-db".to_string()) {
            info!("[DEV] Clearing all events from database");
            match Database::open() {
                Ok(db) => match db.clear_all_events() {
                    Ok(count) => {
                        info!("[DEV] Cleared {} events from database", count);
                        println!("[DEV] Cleared {} events from database", count);
                    }
                    Err(e) => {
                        log::error!("[DEV] Failed to clear events: {}", e);
                        eprintln!("[DEV] Failed to clear events: {}", e);
                    }
                },
                Err(e) => {
                    log::error!("[DEV] Failed to open database: {}", e);
                    eprintln!("[DEV] Failed to open database: {}", e);
                }
            }
        }

        // Check for --dev-seed-data flag
        if args.contains(&"--dev-seed-data".to_string()) {
            info!("[DEV] Generating demo events for a full year");
            println!("[DEV] Seeding database with demo data...");
            match Database::open() {
                Ok(db) => match demo_data::populate_demo_data(&db) {
                    Ok(count) => {
                        info!("[DEV] Generated {} demo events", count);
                        println!("[DEV] Successfully generated {} demo events", count);
                    }
                    Err(e) => {
                        log::error!("[DEV] Failed to generate demo data: {}", e);
                        eprintln!("[DEV] Failed to generate demo data: {}", e);
                    }
                },
                Err(e) => {
                    log::error!("[DEV] Failed to open database: {}", e);
                    eprintln!("[DEV] Failed to open database: {}", e);
                }
            }
        }
    }

    // Initialize localization system
    localize::init();

    info!("Localization initialized, launching application");
    cosmic::app::run::<CosmicCalendar>(Settings::default(), ())
}
