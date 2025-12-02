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
mod url_handler;
mod validation;
mod views;

use app::{AppFlags, CosmicCalendar};
use clap::Parser;
use cosmic::app::Settings;
#[cfg(debug_assertions)]
use database::Database;
use log::info;
#[cfg(debug_assertions)]
use std::env;
use std::path::PathBuf;

/// Sol Calendar - A calendar application for the COSMIC Desktop
#[derive(Parser, Debug)]
#[command(name = "sol-calendar")]
#[command(about = "A calendar application for the COSMIC Desktop", long_about = None)]
struct Cli {
    /// Calendar files to import (.ics files) or URLs to open (webcal://, ics://, calendar://)
    #[arg(value_name = "FILE_OR_URL")]
    inputs: Vec<String>,

    /// Reset database (development only, debug builds only)
    #[cfg(debug_assertions)]
    #[arg(long = "dev-reset-db")]
    dev_reset_db: bool,

    /// Seed database with demo data (development only, debug builds only)
    #[cfg(debug_assertions)]
    #[arg(long = "dev-seed-data")]
    dev_seed_data: bool,
}

/// Application entry point
pub fn main() -> cosmic::iced::Result {
    // Initialize centralized logging
    logging::init();

    info!("Sol Calendar starting...");

    // Parse command-line arguments
    let cli = Cli::parse();

    // Development-only CLI flags (only available in debug builds)
    #[cfg(debug_assertions)]
    {
        // Check for --dev-reset-db flag
        if cli.dev_reset_db {
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
        if cli.dev_seed_data {
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

    // Separate files and URLs
    let mut files_to_open = Vec::new();
    let mut urls_to_open = Vec::new();

    // Debug: Log all received arguments
    info!("main: Received {} input arguments", cli.inputs.len());
    for (i, input) in cli.inputs.iter().enumerate() {
        info!("main: Argument[{}]: {}", i, input);
    }

    for input in cli.inputs {
        // Check if input is a URL scheme (webcal://, ics://, calendar://)
        if input.starts_with("webcal://")
            || input.starts_with("ics://")
            || input.starts_with("calendar://")
        {
            urls_to_open.push(input);
        } else {
            // Treat as file path
            files_to_open.push(PathBuf::from(input));
        }
    }

    // Prepare application flags
    let app_flags = AppFlags {
        files_to_open: files_to_open.clone(),
        urls_to_open: urls_to_open.clone(),
    };

    if !files_to_open.is_empty() {
        info!("Launching with {} file(s) to open", files_to_open.len());
    }

    if !urls_to_open.is_empty() {
        info!("Launching with {} URL(s) to open", urls_to_open.len());
    }

    // Initialize localization system
    localize::init();

    info!("Localization initialized, launching application");

    // Configure application settings
    let settings = Settings::default()
        .exit_on_close(true);  // exit_on_close(true) prevents D-Bus blocking hang

    #[cfg(feature = "single-instance")]
    {
        info!("Launching with single-instance support (use COSMIC_SINGLE_INSTANCE=false to disable)");
        cosmic::app::run_single_instance::<CosmicCalendar>(settings, app_flags)
    }

    #[cfg(not(feature = "single-instance"))]
    {
        info!("Launching without single-instance");
        cosmic::app::run::<CosmicCalendar>(settings, app_flags)
    }
}
