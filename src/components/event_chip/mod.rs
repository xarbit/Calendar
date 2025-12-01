//! Event chip rendering module
//!
//! This module contains all event chip rendering logic, organized into submodules:
//! - `types`: Core types (SpanPosition, ChipOpacity, ChipSelectionState, DisplayEvent)
//! - `all_day`: All-day event chip rendering
//! - `timed`: Timed event chip rendering
//! - `clickable`: Clickable event chip wrapper
//! - `quick_event`: Quick event input fields
//! - `unified`: Unified events column rendering
//! - `compact`: Compact events rendering

mod all_day;
mod clickable;
mod compact;
mod quick_event;
mod timed;
mod types;
mod unified;

// Re-export public types (only what's actually used externally)
pub use types::{ChipOpacity, DisplayEvent, span_border_radius_from_flags};

// Re-export rendering functions (only what's actually used externally)
pub use compact::render_compact_events;
pub use quick_event::{
    quick_event_input_id, render_quick_event_input, render_spanning_quick_event_input,
};
pub use unified::render_unified_events_with_selection;
