# Sol Calendar Refactoring Plan

This document tracks the refactoring tasks identified from codebase analysis. Tasks are organized into phases by priority and effort.

---

## Phase 1: Quick Wins (1-2 days)

Low-effort, high-impact changes that reduce code duplication with minimal risk.

### 1.1 Extract Span Position Utilities ✅
- [x] Create `span_border_radius()` function in `components/event_chip.rs`
- [x] Create `span_border_radius_from_flags()` function in `components/event_chip.rs`
- [x] Create `span_padding()` function in `components/event_chip.rs`
- [x] Add `SpanPosition::from_start_end()` method
- [x] Replace duplicated logic in:
  - [x] `event_chip.rs` (render_all_day_chip)
  - [x] `event_chip.rs` (render_all_day_chip_selectable)
  - [x] `month.rs` (render_date_event_chip)
  - [x] `month.rs` (render_compact_date_event_chip)

**Current pattern (repeated 3x):**
```rust
let border_radius: [f32; 4] = match span_position {
    SpanPosition::Single => [radius, radius, radius, radius],
    SpanPosition::First => [radius, 0.0, 0.0, radius],
    SpanPosition::Middle => [0.0, 0.0, 0.0, 0.0],
    SpanPosition::Last => [0.0, radius, radius, 0.0],
};
```

**Target:**
```rust
pub fn span_position_border_radius(span_position: SpanPosition, radius: f32) -> [f32; 4]
pub fn span_position_padding(span_position: SpanPosition) -> [u16; 4]
```

---

### 1.2 Create Spacer Helper Functions (In Progress)
- [x] Add spacer utilities - create `components/spacer.rs`
- [x] Replace `container(widget::text(""))` pattern in:
  - [x] `views/month.rs` (13 occurrences replaced)
  - [x] `views/week.rs` (7 occurrences replaced)
- [ ] Replace remaining occurrences in:
  - [ ] `components/event_chip.rs`
  - [ ] `components/time_grid.rs`
  - [ ] `components/event_dialog.rs`
  - [ ] `components/mini_calendar.rs`
  - [ ] `components/color_picker.rs`
  - [ ] `views/year.rs`

**Current pattern (repeated 13+ times):**
```rust
container(widget::text(""))
    .width(Length::Fill)
    .height(Length::Fixed(height))
```

**Target:**
```rust
pub fn spacer(width: Length, height: Length) -> Element<'static, Message>
pub fn fixed_spacer(width: f32, height: f32) -> Element<'static, Message>
pub fn vertical_spacer(height: f32) -> Element<'static, Message>
pub fn horizontal_spacer(width: f32) -> Element<'static, Message>
```

---

### 1.3 Safe Hex Color Parsing ✅
- [x] Add `parse_color_safe()` to `components/color_picker.rs`
- [x] Replace `parse_hex_color().unwrap_or()` pattern in:
  - [x] `views/month.rs` (2 occurrences)
  - [x] `views/week.rs` (3 occurrences)
- [ ] Remaining files to update:
  - [ ] `components/event_chip.rs` (6 calls)
  - [ ] `components/color_picker.rs` (3 calls)
  - [ ] `components/calendar_dialog.rs` (1 call)

**Current pattern (repeated 20+ times):**
```rust
parse_hex_color(&event.color).unwrap_or(cosmic::iced::Color::from_rgb(0.5, 0.5, 0.5))
```

**Target:**
```rust
pub const COLOR_DEFAULT_EVENT: Color = Color::from_rgb(0.5, 0.5, 0.5);

pub fn parse_color_safe(hex: &str) -> Color {
    parse_hex_color(hex).unwrap_or(COLOR_DEFAULT_EVENT)
}
```

---

### 1.4 Weekend Background Helper ✅
- [x] Add `weekend_background()` to `styles.rs`
- [x] Replace weekend background pattern:
  - [x] `components/time_grid.rs` (2 occurrences)
  - [x] `views/week.rs` (3 occurrences)
  - [x] `styles.rs` (2 occurrences - internal use within existing style functions)

**Current pattern (repeated 14x):**
```rust
background: if is_weekend {
    Some(Background::Color(COLOR_WEEKEND_BACKGROUND))
} else {
    None
}
```

**Target:**
```rust
pub fn weekend_background(is_weekend: bool) -> Option<Background> {
    if is_weekend {
        Some(Background::Color(COLOR_WEEKEND_BACKGROUND))
    } else {
        None
    }
}
```

---

### 1.5 Opacity Calculation Helper ✅
- [x] Create `ChipOpacity` struct with `from_state()` and `dot_opacity()` methods in `event_chip.rs`
- [x] Replace opacity calculation pattern:
  - [x] `event_chip.rs` (render_all_day_chip_selectable, render_timed_event_chip_selectable)
  - [x] `month.rs` (render_date_event_chip)

**Current pattern (repeated 3+ times):**
```rust
let base_opacity = if is_being_dragged { 0.15 } else if is_selected { 0.5 } else { 0.3 };
let text_opacity = if is_being_dragged { 0.4 } else { 1.0 };
```

**Target:**
```rust
pub struct ChipOpacity {
    pub background: f32,
    pub text: f32,
    pub border: f32,
}

pub fn calculate_chip_opacity(is_selected: bool, is_being_dragged: bool) -> ChipOpacity
```

---

## Phase 2: Structural Improvements (3-5 days)

Medium-effort changes that improve code organization and maintainability.

### 2.1 Consolidate Event Chip Rendering Functions ✅
- [x] Create `ChipSelectionState` struct to bundle selection parameters
- [x] Merge `render_all_day_chip()` and `render_all_day_chip_selectable()` into single function
- [x] Merge `render_timed_event_chip()` and `render_timed_event_chip_selectable()` into single function
- [x] Update all call sites to use new unified functions

**Completed:** 4 functions consolidated into 2 generic functions with `Option<ChipSelectionState>` parameter

```rust
pub struct ChipSelectionState {
    pub is_selected: bool,
    pub is_being_dragged: bool,
    pub is_drag_active: bool,
}

pub fn render_all_day_chip(
    summary: String,
    color: Color,
    span_position: SpanPosition,
    selection: Option<ChipSelectionState>,
) -> Element<'static, Message>
```

---

### 2.2 Consolidate Style Functions ✅
- [x] Move `grid_cell_style()` from `time_grid.rs` to `styles.rs`
- [x] Move `bordered_cell_style()` from `time_grid.rs` to `styles.rs`
- [x] `apply_day_cell_style()` kept in `day_cell.rs` (it builds containers, not just styles; uses styles from `styles.rs`)
- [x] Update imports in all files that use these functions
- [x] Remove duplicate style definitions from `time_grid.rs`

**Completed:** Style functions consolidated in `styles.rs`, imports updated in `views/day.rs`

---

### 2.3 Extract Month View Submodules ✅
- [x] Create `src/views/month/` directory
- [x] Extract header rendering to `src/views/month/header.rs`
  - [x] `render_weekday_header()` function (~45 lines)
- [x] Extract overlay logic to `src/views/month/overlay.rs`
  - [x] `compute_week_date_event_slots()` function
  - [x] `collect_date_event_segments()` function
  - [x] `render_date_events_overlay()` function
  - [x] `WeekSlotInfo` and `DateEventSegment` structs (~420 lines)
- [x] Extract event rendering to `src/views/month/events.rs`
  - [x] `render_date_event_chip()` function
  - [x] `render_compact_date_event_chip()` function (~115 lines)
- [x] Extract selection overlay to `src/views/month/selection.rs`
  - [x] `render_spanning_overlay()` function (~120 lines)
- [x] Create `src/views/month/mod.rs` as entry point (~315 lines)
- [x] Delete old `src/views/month.rs`
- [x] Update exports (existing `mod.rs` already compatible)

**Completed structure:**
```
views/
├── month/
│   ├── mod.rs (315 lines) - Main entry point + MonthViewEvents
│   ├── header.rs (45 lines) - Weekday header rendering
│   ├── overlay.rs (420 lines) - Slot computation + date event overlay
│   ├── events.rs (115 lines) - Date event chip rendering
│   └── selection.rs (120 lines) - Quick event selection overlay
```

**Total:** ~1015 lines extracted into organized submodules

---

### 2.4 Extract Event Chip Submodules ✅
- [x] Create `src/components/event_chip/` directory
- [x] Extract all-day chip rendering to `src/components/event_chip/all_day.rs` (~62 lines)
- [x] Extract timed chip rendering to `src/components/event_chip/timed.rs` (~96 lines)
- [x] Extract quick event input to `src/components/event_chip/quick_event.rs` (~89 lines)
- [x] Extract unified rendering to `src/components/event_chip/unified.rs` (~153 lines)
- [x] Extract compact rendering to `src/components/event_chip/compact.rs` (~143 lines)
- [x] Extract clickable chip wrapper to `src/components/event_chip/clickable.rs` (~84 lines)
- [x] Extract core types to `src/components/event_chip/types.rs` (~155 lines)
- [x] Create `mod.rs` with public API re-exports (~28 lines)
- [x] Delete old `src/components/event_chip.rs`

**Completed structure:**
```
components/
├── event_chip/
│   ├── mod.rs (28 lines) - Public API re-exports
│   ├── types.rs (155 lines) - SpanPosition, ChipOpacity, DisplayEvent
│   ├── all_day.rs (62 lines) - All-day event chip
│   ├── timed.rs (96 lines) - Timed event chip with dot
│   ├── clickable.rs (84 lines) - Clickable chip wrapper
│   ├── quick_event.rs (89 lines) - Quick event input
│   ├── unified.rs (153 lines) - Unified events column
│   └── compact.rs (143 lines) - Compact events indicators
```

**Total:** ~810 lines organized into 8 logical submodules (from original 708 lines)

---

### 2.5 Reduce Clone Operations in Month View
- [ ] Identify all `.clone()` calls in closure captures in `month.rs`
- [ ] Replace heavy clones with `Rc<>` or `Arc<>` where appropriate
- [ ] Use references instead of owned values in callbacks where possible
- [ ] Cache computed values instead of cloning in closures

**Focus areas:**
- Lines 807-812: `weeks`, `events_by_date` cloning
- Responsive callback closures
- Event iteration with cloning

---

### 2.6 Share Day Column Rendering Logic
- [ ] Create generic `DayColumnContent` struct
- [ ] Create `render_day_column_generic()` function in `components/`
- [ ] Refactor `week.rs` to use generic day column renderer
- [ ] Refactor `day.rs` to use generic day column renderer

```rust
pub struct DayColumnConfig {
    pub date: NaiveDate,
    pub events: Vec<DisplayEvent>,
    pub is_weekend: bool,
    pub is_today: bool,
    pub show_time_indicator: bool,
    pub current_hour: Option<u32>,
    pub current_minute: Option<u32>,
}

pub fn render_day_column(config: DayColumnConfig) -> Element<'static, Message>
```

---

## Phase 3: Polish (Ongoing)

Low-priority improvements for consistency and cleanliness.

### 3.1 Standardize Border Radius Usage
- [ ] Audit all border radius usages across codebase
- [ ] Create `default_border_radius()` helper function
- [ ] Replace hardcoded values (e.g., `4.0`) with constant references
- [ ] Ensure consistent use of `BORDER_RADIUS.into()` pattern

**Files to check:**
- `components/event_chip.rs` - Uses `BORDER_RADIUS[0]`
- `components/time_grid.rs` - Hardcoded `4.0`
- `styles.rs` - Uses `BORDER_RADIUS.into()`

---

### 3.2 Consistent Container Styling Pattern
- [ ] Document preferred closure style pattern
- [ ] Audit `move` keyword usage (only when capturing variables)
- [ ] Standardize `_theme` vs `theme` parameter naming
- [ ] Create style helper functions that accept theme parameter

**Pattern to enforce:**
```rust
// When not capturing variables:
.style(|_theme: &cosmic::Theme| container::Style { ... })

// When capturing variables:
.style(move |_theme: &cosmic::Theme| container::Style { ... })

// When using theme:
.style(|theme: &cosmic::Theme| some_style_function(theme))
```

---

### 3.3 Organize Layout Constants
- [ ] Group related constants into sub-modules
- [ ] Add documentation comments to constant groups
- [ ] Consider creating type aliases for clarity

**Target structure in `layout_constants.rs`:**
```rust
/// Calendar grid dimensions
pub mod grid {
    pub const HOUR_ROW_HEIGHT: f32 = 60.0;
    pub const TIME_LABEL_WIDTH: f32 = 60.0;
    pub const WEEK_NUMBER_WIDTH: f32 = 32.0;
}

/// Day cell dimensions
pub mod day_cell {
    pub const HEADER_HEIGHT: f32 = 28.0;
    pub const HEADER_OFFSET: f32 = 32.0;
    pub const TOP_PADDING: f32 = 4.0;
}

/// Event display dimensions
pub mod events {
    pub const DATE_EVENT_HEIGHT: f32 = 19.0;
    pub const COMPACT_EVENT_HEIGHT: f32 = 6.0;
    pub const DATE_EVENT_SPACING: f32 = 2.0;
}
```

---

### 3.4 Document Update Handler Helpers
- [ ] Add documentation comments to helper functions in `update/mod.rs`
- [ ] Consider extracting helpers to separate `update/helpers.rs` module
- [ ] Document the purpose and usage of each helper

**Helpers to document (lines 30-108):**
- `focus_quick_event_input()`
- `scroll_week_to_hour()`
- `dismiss_on_focus_loss()`
- `close_legacy_event_dialog()`
- `schedule_deferred_scroll_restore()`
- `close_quick_event_with_scroll_restore()`

---

### 3.5 Remove Dead Code (Selective)
- [ ] Review compiler warnings for unused code
- [ ] Remove truly dead code that won't be used
- [ ] Keep callback structures (needed for future implementations)
- [ ] Keep validation module (foundation for future features)
- [ ] Add `#[allow(dead_code)]` with comments for intentionally unused code

**Keep (with documentation):**
- `components/calendar_dialog_callbacks.rs` - Future dialog refactoring
- `components/event_dialog_callbacks.rs` - Future dialog refactoring
- `validation.rs` - Foundation for input validation

**Review for removal:**
- Unused functions flagged by compiler
- Commented-out code blocks
- Unused imports

---

## Progress Tracking

### Phase 1 Status
- [x] 1.1 Span Position Utilities
- [x] 1.2 Spacer Helpers (main views done, remaining components TBD)
- [x] 1.3 Safe Color Parsing (main views done, remaining components TBD)
- [x] 1.4 Weekend Background Helper
- [x] 1.5 Opacity Calculation Helper

### Phase 2 Status
- [x] 2.1 Consolidate Event Chip Functions
- [x] 2.2 Consolidate Style Functions
- [x] 2.3 Extract Month View Submodules
- [x] 2.4 Extract Event Chip Submodules
- [ ] 2.5 Reduce Clone Operations
- [ ] 2.6 Share Day Column Logic

### Phase 3 Status
- [ ] 3.1 Border Radius Standardization
- [ ] 3.2 Container Styling Pattern
- [ ] 3.3 Layout Constants Organization
- [ ] 3.4 Update Handler Documentation
- [ ] 3.5 Dead Code Cleanup

---

## Notes

- Always run `cargo build --release` and `cargo test` after changes
- Commit after each completed task for easy rollback
- Update this file as tasks are completed
- Consider creating feature branches for larger Phase 2 changes
