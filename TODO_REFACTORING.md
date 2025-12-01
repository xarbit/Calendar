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

### 2.5 Reduce Clone Operations in Month View (Deferred)
**Status:** Deferred - Low priority optimization. The clones in responsive closures are required
by iced's widget API and the current implementation performs well.

- [ ] Identify all `.clone()` calls in closure captures in `month.rs`
- [ ] Replace heavy clones with `Rc<>` or `Arc<>` where appropriate
- [ ] Use references instead of owned values in callbacks where possible
- [ ] Cache computed values instead of cloning in closures

**Note:** Requires API changes to `MonthViewEvents` struct and would add complexity without
significant performance benefit.

---

### 2.6 Share Day Column Rendering Logic (Deferred)
**Status:** Deferred - Medium priority. Would reduce code duplication between week.rs and day.rs
but requires careful API design.

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

### 3.1 Standardize Border Radius Usage ✅
- [x] Audit all border radius usages across codebase
- [x] Add `BORDER_RADIUS_VALUE` (4.0) and `BORDER_RADIUS_SMALL` (2.0) constants
- [x] Replace hardcoded values with constant references
- [x] Ensure consistent use of `BORDER_RADIUS.into()` pattern

**Completed:**
- Added `BORDER_RADIUS_VALUE` for single-value usage (span functions)
- Added `BORDER_RADIUS_SMALL` for compact event indicators
- Updated `layout.rs` - drag preview border
- Updated `views/week.rs` - current time indicator dot
- Updated `views/year.rs` - today highlight in year view
- Updated `views/month/events.rs` - date event chips (full and compact)

---

### 3.2 Consistent Container Styling Pattern ✅
- [x] Document preferred closure style pattern
- [x] Audit `move` keyword usage (only when capturing variables)
- [x] Standardize `_theme` vs `theme` parameter naming
- [x] Create style helper functions that accept theme parameter

**Status:** Already consistent across codebase after refactoring. Pattern is:
```rust
// When not capturing variables:
.style(|_theme: &cosmic::Theme| container::Style { ... })

// When capturing variables:
.style(move |_theme: &cosmic::Theme| container::Style { ... })

// When using theme:
.style(|theme: &cosmic::Theme| some_style_function(theme))
```

Style helper functions in `styles.rs` accept theme parameter when needed (e.g., `today_filled_style`, `adjacent_month_selected_style`).

---

### 3.3 Organize Layout Constants ✅
- [x] Group related constants with clear section headers
- [x] Add documentation comments to all constants
- [x] Add module-level documentation with category list

**Completed:** Added comprehensive documentation and visual section headers:
- Module-level doc with category overview
- Clear `// ===` section dividers for each category
- Doc comments for every constant explaining its purpose and value
- Categories: Layout, Calendar Views, Menu, Spacing, Padding, Border, Color Picker, Shadow, Icons, Fonts, Events, Thresholds, Day Cell

**Note:** Sub-modules were not used (would require import changes throughout codebase). Section headers provide sufficient organization.

---

### 3.4 Document Update Handler Helpers ✅
- [x] Add documentation comments to helper functions in `update/mod.rs`
- [x] Add module-level documentation listing all helpers
- [x] Document the purpose and usage of each helper

**Completed:** All helpers were already documented with doc comments. Enhanced module-level
documentation to include a "Helper Functions" section listing all helpers with descriptions.

**Helpers documented:**
- `dismiss_on_focus_loss()` - Clear transient UI state when user navigates away
- `focus_quick_event_input()` - Focus the quick event text input
- `scroll_week_to_current_time()` - Scroll week view to show current time
- `scroll_week_to_hour()` - Scroll week view to a specific hour
- `close_legacy_event_dialog()` - Close deprecated event dialog field
- `schedule_deferred_scroll_restore()` - Schedule scroll position restoration
- `close_quick_event_with_scroll_restore()` - Close quick event and restore scroll

**Note:** Helpers kept in `mod.rs` rather than separate file - they're small, closely related
to the message handler, and used only within the update module.

---

### 3.5 Remove Dead Code (Selective) ✅
- [x] Review compiler warnings for unused code
- [x] Remove truly dead code that won't be used
- [x] Keep callback structures (needed for future implementations)
- [x] Keep validation module (foundation for future features)
- [x] Add `#[allow(dead_code)]` with comments for intentionally unused code

**Completed:** Added `#[allow(dead_code)]` annotations to ~40 functions/methods/types across:
- `src/components/spacer.rs` - Shrink spacer variant
- `src/styles.rs` - Grid cell style helper
- `src/components/day_header.rs` - Week view config
- `src/components/day_cell.rs` - Event slots field
- `src/components/time_grid.rs` - Day column constructor
- `src/cache.rs` - Period text getter
- `src/layout_constants.rs` - Border opacity and palette constants
- `src/calendars/` - Calendar source trait methods and manager methods
- `src/database/schema.rs` - Encryption and update methods
- `src/logging.rs` - Debug level checks
- `src/locale.rs` - Weekday calculation
- `src/protocols/` - Protocol trait and implementations
- `src/dialogs/manager.rs` - Dialog helpers and action enum
- `src/services/` - Export, settings, calendar, and event handlers
- `src/models/` - State navigation methods
- `src/selection.rs` - Time-based selection methods
- `src/message.rs` - Message enum variants

**Warning reduction:** 51 → 5 (remaining 5 are expected `deprecated` warnings for `event_dialog` field)

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
- [~] 2.5 Reduce Clone Operations (Deferred)
- [~] 2.6 Share Day Column Logic (Deferred)

### Phase 3 Status
- [x] 3.1 Border Radius Standardization
- [x] 3.2 Container Styling Pattern
- [x] 3.3 Layout Constants Organization
- [x] 3.4 Update Handler Documentation
- [x] 3.5 Dead Code Cleanup

---

## Phase 4: Additional Submodule Extractions

Large files that would benefit from submodule organization, following the patterns established in Phase 2.

### 4.1 Extract Week View Submodules ✅
**File:** `views/week.rs` (951 lines) → `views/week/` directory
**Priority:** HIGH

- [x] Create `src/views/week/` directory
- [x] Extract header rendering to `src/views/week/header.rs`
  - [x] `render_header_section()` function
  - [x] `render_day_header()` function
  - [x] `render_all_day_section()` function
  - [x] `render_all_day_events_for_day()` function
- [x] Extract time grid rendering to `src/views/week/time_grid.rs`
  - [x] `render_time_labels_column()` function
  - [x] `render_hour_grid_background()` function
  - [x] `render_clickable_hour_cell()` function
- [x] Extract events overlay to `src/views/week/events.rs`
  - [x] `render_events_overlay_layer()` function
  - [x] `render_column_events()` function
  - [x] `render_positioned_event_block()` function
- [x] Extract time indicator to `src/views/week/time_indicator.rs`
  - [x] `render_time_indicator_layer()` function
- [x] Extract quick event input to `src/views/week/quick_event.rs`
  - [x] `render_quick_event_input_layer()` function
- [x] Extract utilities to `src/views/week/utils.rs`
  - [x] `PositionedEvent` struct
  - [x] `separate_events()`, `calculate_max_all_day_slots()` functions
  - [x] `event_time_range()`, `events_overlap()`, `calculate_event_columns()` functions
  - [x] Week view constants (`DAY_HEADER_HEIGHT`, `ALL_DAY_*`)
- [x] Create `src/views/week/mod.rs` as entry point
- [x] Delete old `src/views/week.rs`

**Completed structure:**
```
views/week/
├── mod.rs           (245 lines) - Main render_week_view, exports
├── header.rs        (186 lines) - Day headers + all-day events section
├── time_grid.rs     (103 lines) - Time grid background and hour markers
├── events.rs        (146 lines) - Event positioning & rendering
├── time_indicator.rs (90 lines) - Current time indicator
├── quick_event.rs   (71 lines) - Quick event input overlay
└── utils.rs         (161 lines) - Helper functions and constants
```

**Total:** 951 lines organized into 7 logical submodules

---

### 4.2 Extract Selection Submodules ✅
**File:** `selection.rs` (874 lines) → `selection/` directory
**Priority:** MEDIUM

- [x] Create `src/selection/` directory
- [x] Extract selection point to `src/selection/point.rs`
  - [x] `SelectionPoint` struct and methods
- [x] Extract selection state to `src/selection/state.rs`
  - [x] `SelectionState` struct and methods
- [x] Extract selection range to `src/selection/range.rs`
  - [x] `SelectionRange` struct and methods
- [x] Extract drag state to `src/selection/drag.rs`
  - [x] `EventDragState` struct
  - [x] `DragPreviewInfo` struct
  - [x] `DragTarget` struct
- [x] Create `src/selection/mod.rs` as entry point with tests
- [x] Delete old `src/selection.rs`

**Completed structure:**
```
selection/
├── mod.rs    (300 lines) - Entry point, re-exports, tests
├── point.rs  (42 lines) - SelectionPoint struct
├── state.rs  (167 lines) - SelectionState for drag selection
├── range.rs  (88 lines) - SelectionRange (normalized)
└── drag.rs   (234 lines) - EventDragState, DragPreviewInfo, DragTarget
```

**Total:** 874 lines organized into 5 logical submodules (21 tests preserved)

---

### 4.3 Extract Event Dialog Submodules (Deferred)
**File:** `components/event_dialog.rs` (581 lines)
**Priority:** LOW (Deferred)

**Status:** Deferred - The event dialog is a single render function with inline section builders.
The code is tightly coupled and doesn't naturally decompose into submodules without adding
complexity. The sections are built inline and passed to a settings::view_column, making
extraction awkward. The file is already well-organized with clear section comments.

**Potential future improvements:**
- Extract label helper functions if they grow (travel_time_label, repeat_label, alert_label)
- Create section builder functions if the dialog gets more complex
- Consider extraction if the file grows beyond 800+ lines

---

## Progress Tracking

### Phase 4 Status
- [x] 4.1 Extract Week View Submodules
- [x] 4.2 Extract Selection Submodules
- [~] 4.3 Extract Event Dialog Submodules (Deferred)

---

## Notes

- Always run `cargo build --release` and `cargo test` after changes
- Commit after each completed task for easy rollback
- Update this file as tasks are completed
- Consider creating feature branches for larger Phase 2 changes
