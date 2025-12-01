use crate::models::CalendarState;
use std::collections::HashMap;

/// Manages all calendar caching including state, formatted text, and pre-cached months
pub struct CalendarCache {
    /// Cache of calendar states by (year, month)
    states: HashMap<(i32, u32), CalendarState>,
    /// Cache of formatted period text by (year, month)
    period_texts: HashMap<(i32, u32), String>,
    /// Current active month
    current: (i32, u32),
}

impl CalendarCache {
    /// Create a new cache for the given year and month
    pub fn new(year: i32, month: u32) -> Self {
        let mut cache = CalendarCache {
            states: HashMap::new(),
            period_texts: HashMap::new(),
            current: (year, month),
        };

        // Cache the initial month
        cache.ensure_cached(year, month);

        cache
    }

    /// Update the current month and ensure it's cached
    pub fn set_current(&mut self, year: i32, month: u32) {
        let changed = self.current != (year, month);
        self.current = (year, month);

        if changed {
            self.ensure_cached(year, month);
        }
    }

    /// Get the current calendar state
    pub fn current_state(&self) -> &CalendarState {
        self.states.get(&self.current)
            .expect("Current month should always be cached")
    }

    /// Get the current formatted period text
    pub fn current_period_text(&self) -> &str {
        self.period_texts.get(&self.current)
            .map(|s| s.as_str())
            .expect("Current month period text should always be cached")
    }

    /// Get the current month name
    pub fn current_month_text(&self) -> String {
        let date = chrono::NaiveDate::from_ymd_opt(self.current.0, self.current.1, 1).unwrap();
        format!("{}", date.format("%B"))
    }

    /// Get the current year as text
    pub fn current_year_text(&self) -> String {
        self.current.0.to_string()
    }

    /// Ensure a specific month is cached
    fn ensure_cached(&mut self, year: i32, month: u32) {
        let key = (year, month);

        // Cache calendar state if not already present
        if !self.states.contains_key(&key) {
            self.states.insert(key, CalendarState::new(year, month));
        }

        // Cache period text if not already present
        if !self.period_texts.contains_key(&key) {
            let date = chrono::NaiveDate::from_ymd_opt(year, month, 1).unwrap();
            let period_text = format!("{}", date.format("%B %Y"));
            self.period_texts.insert(key, period_text);
        }
    }

    /// Pre-cache surrounding months (previous and next N months)
    pub fn precache_surrounding(&mut self, months_before: u32, months_after: u32) {
        let (year, month) = self.current;

        // Pre-cache previous months
        for i in 1..=months_before {
            let (prev_year, prev_month) = self.subtract_months(year, month, i);
            self.ensure_cached(prev_year, prev_month);
        }

        // Pre-cache next months
        for i in 1..=months_after {
            let (next_year, next_month) = self.add_months(year, month, i);
            self.ensure_cached(next_year, next_month);
        }
    }

    /// Add N months to a given year/month
    fn add_months(&self, year: i32, month: u32, n: u32) -> (i32, u32) {
        let total_months = (year * 12 + month as i32 - 1) + n as i32;
        let new_year = total_months / 12;
        let new_month = (total_months % 12) + 1;
        (new_year, new_month as u32)
    }

    /// Subtract N months from a given year/month
    fn subtract_months(&self, year: i32, month: u32, n: u32) -> (i32, u32) {
        let total_months = (year * 12 + month as i32 - 1) - n as i32;
        let new_year = total_months / 12;
        let new_month = (total_months % 12) + 1;
        (new_year, new_month as u32)
    }

    /// Clear old cached entries to prevent unbounded growth
    /// Keeps only the current month and surrounding months
    #[allow(dead_code)] // For future use when implementing cache cleanup strategies
    pub fn cleanup(&mut self, keep_radius: u32) {
        let (year, month) = self.current;

        // Collect keys to keep
        let mut keys_to_keep = vec![(year, month)];

        for i in 1..=keep_radius {
            keys_to_keep.push(self.subtract_months(year, month, i));
            keys_to_keep.push(self.add_months(year, month, i));
        }

        // Remove entries not in the keep list
        self.states.retain(|k, _| keys_to_keep.contains(k));
        self.period_texts.retain(|k, _| keys_to_keep.contains(k));
    }

    /// Get cache statistics for debugging
    #[allow(dead_code)]
    pub fn stats(&self) -> CacheStats {
        CacheStats {
            states_cached: self.states.len(),
            period_texts_cached: self.period_texts.len(),
            current_month: self.current,
        }
    }
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct CacheStats {
    pub states_cached: usize,
    pub period_texts_cached: usize,
    pub current_month: (i32, u32),
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== Cache Creation Tests ====================

    #[test]
    fn test_cache_new() {
        let cache = CalendarCache::new(2024, 6);
        assert_eq!(cache.current, (2024, 6));
        assert!(cache.states.contains_key(&(2024, 6)));
        assert!(cache.period_texts.contains_key(&(2024, 6)));
    }

    #[test]
    fn test_cache_new_january() {
        let cache = CalendarCache::new(2024, 1);
        assert_eq!(cache.current, (2024, 1));
    }

    #[test]
    fn test_cache_new_december() {
        let cache = CalendarCache::new(2024, 12);
        assert_eq!(cache.current, (2024, 12));
    }

    // ==================== set_current Tests ====================

    #[test]
    fn test_set_current_new_month() {
        let mut cache = CalendarCache::new(2024, 6);
        cache.set_current(2024, 7);
        assert_eq!(cache.current, (2024, 7));
        assert!(cache.states.contains_key(&(2024, 7)));
    }

    #[test]
    fn test_set_current_same_month() {
        let mut cache = CalendarCache::new(2024, 6);
        let initial_states_count = cache.states.len();
        cache.set_current(2024, 6);
        assert_eq!(cache.states.len(), initial_states_count);
    }

    #[test]
    fn test_set_current_cross_year_forward() {
        let mut cache = CalendarCache::new(2024, 12);
        cache.set_current(2025, 1);
        assert_eq!(cache.current, (2025, 1));
        assert!(cache.states.contains_key(&(2025, 1)));
    }

    #[test]
    fn test_set_current_cross_year_backward() {
        let mut cache = CalendarCache::new(2024, 1);
        cache.set_current(2023, 12);
        assert_eq!(cache.current, (2023, 12));
        assert!(cache.states.contains_key(&(2023, 12)));
    }

    // ==================== current_state Tests ====================

    #[test]
    fn test_current_state() {
        let cache = CalendarCache::new(2024, 6);
        let state = cache.current_state();
        assert_eq!(state.year, 2024);
        assert_eq!(state.month, 6);
    }

    // ==================== current_period_text Tests ====================

    #[test]
    fn test_current_period_text() {
        let cache = CalendarCache::new(2024, 6);
        let text = cache.current_period_text();
        assert!(text.contains("June"));
        assert!(text.contains("2024"));
    }

    #[test]
    fn test_current_period_text_january() {
        let cache = CalendarCache::new(2024, 1);
        let text = cache.current_period_text();
        assert!(text.contains("January"));
    }

    #[test]
    fn test_current_period_text_december() {
        let cache = CalendarCache::new(2024, 12);
        let text = cache.current_period_text();
        assert!(text.contains("December"));
    }

    // ==================== current_month_text Tests ====================

    #[test]
    fn test_current_month_text() {
        let cache = CalendarCache::new(2024, 6);
        assert_eq!(cache.current_month_text(), "June");
    }

    #[test]
    fn test_current_month_text_all_months() {
        let months = [
            "January", "February", "March", "April", "May", "June",
            "July", "August", "September", "October", "November", "December"
        ];
        for (i, month_name) in months.iter().enumerate() {
            let cache = CalendarCache::new(2024, (i + 1) as u32);
            assert_eq!(cache.current_month_text(), *month_name);
        }
    }

    // ==================== current_year_text Tests ====================

    #[test]
    fn test_current_year_text() {
        let cache = CalendarCache::new(2024, 6);
        assert_eq!(cache.current_year_text(), "2024");
    }

    #[test]
    fn test_current_year_text_various_years() {
        for year in [1999, 2000, 2024, 2050, 2100] {
            let cache = CalendarCache::new(year, 1);
            assert_eq!(cache.current_year_text(), year.to_string());
        }
    }

    // ==================== add_months Tests ====================

    #[test]
    fn test_add_months_same_year() {
        let cache = CalendarCache::new(2024, 1);
        assert_eq!(cache.add_months(2024, 1, 1), (2024, 2));
        assert_eq!(cache.add_months(2024, 1, 6), (2024, 7));
        assert_eq!(cache.add_months(2024, 1, 11), (2024, 12));
    }

    #[test]
    fn test_add_months_cross_year() {
        let cache = CalendarCache::new(2024, 1);
        assert_eq!(cache.add_months(2024, 1, 12), (2025, 1));
        assert_eq!(cache.add_months(2024, 6, 12), (2025, 6));
        assert_eq!(cache.add_months(2024, 12, 1), (2025, 1));
    }

    #[test]
    fn test_add_months_multiple_years() {
        let cache = CalendarCache::new(2024, 1);
        assert_eq!(cache.add_months(2024, 1, 24), (2026, 1));
        assert_eq!(cache.add_months(2024, 1, 36), (2027, 1));
    }

    // ==================== subtract_months Tests ====================

    #[test]
    fn test_subtract_months_same_year() {
        let cache = CalendarCache::new(2024, 12);
        assert_eq!(cache.subtract_months(2024, 12, 1), (2024, 11));
        assert_eq!(cache.subtract_months(2024, 12, 6), (2024, 6));
        assert_eq!(cache.subtract_months(2024, 12, 11), (2024, 1));
    }

    #[test]
    fn test_subtract_months_cross_year() {
        let cache = CalendarCache::new(2024, 1);
        assert_eq!(cache.subtract_months(2024, 1, 1), (2023, 12));
        assert_eq!(cache.subtract_months(2024, 6, 12), (2023, 6));
    }

    #[test]
    fn test_subtract_months_multiple_years() {
        let cache = CalendarCache::new(2024, 1);
        assert_eq!(cache.subtract_months(2024, 1, 24), (2022, 1));
        assert_eq!(cache.subtract_months(2024, 1, 36), (2021, 1));
    }

    // ==================== precache_surrounding Tests ====================

    #[test]
    fn test_precache_surrounding() {
        let mut cache = CalendarCache::new(2024, 6);
        cache.precache_surrounding(2, 2);

        // Should have current + 2 before + 2 after = 5 months cached
        assert!(cache.states.contains_key(&(2024, 4)));
        assert!(cache.states.contains_key(&(2024, 5)));
        assert!(cache.states.contains_key(&(2024, 6)));
        assert!(cache.states.contains_key(&(2024, 7)));
        assert!(cache.states.contains_key(&(2024, 8)));
    }

    #[test]
    fn test_precache_surrounding_cross_year() {
        let mut cache = CalendarCache::new(2024, 1);
        cache.precache_surrounding(2, 2);

        // Should cache Nov 2023, Dec 2023, Jan 2024, Feb 2024, Mar 2024
        assert!(cache.states.contains_key(&(2023, 11)));
        assert!(cache.states.contains_key(&(2023, 12)));
        assert!(cache.states.contains_key(&(2024, 1)));
        assert!(cache.states.contains_key(&(2024, 2)));
        assert!(cache.states.contains_key(&(2024, 3)));
    }

    // ==================== cleanup Tests ====================

    #[test]
    fn test_cleanup() {
        let mut cache = CalendarCache::new(2024, 6);
        // Pre-cache many months
        cache.precache_surrounding(6, 6);

        let initial_count = cache.states.len();
        assert!(initial_count > 3);

        // Cleanup to keep only radius of 1
        cache.cleanup(1);

        // Should only keep current month and ±1 month = 3 months
        assert_eq!(cache.states.len(), 3);
        assert!(cache.states.contains_key(&(2024, 5)));
        assert!(cache.states.contains_key(&(2024, 6)));
        assert!(cache.states.contains_key(&(2024, 7)));
    }

    #[test]
    fn test_cleanup_cross_year() {
        let mut cache = CalendarCache::new(2024, 1);
        cache.precache_surrounding(3, 3);
        cache.cleanup(1);

        // Should keep Dec 2023, Jan 2024, Feb 2024
        assert_eq!(cache.states.len(), 3);
        assert!(cache.states.contains_key(&(2023, 12)));
        assert!(cache.states.contains_key(&(2024, 1)));
        assert!(cache.states.contains_key(&(2024, 2)));
    }

    // ==================== stats Tests ====================

    #[test]
    fn test_stats() {
        let mut cache = CalendarCache::new(2024, 6);
        cache.precache_surrounding(1, 1);

        let stats = cache.stats();
        assert_eq!(stats.states_cached, 3);
        assert_eq!(stats.period_texts_cached, 3);
        assert_eq!(stats.current_month, (2024, 6));
    }

    // ==================== CacheStats Debug Tests ====================

    #[test]
    fn test_cache_stats_debug() {
        let stats = CacheStats {
            states_cached: 5,
            period_texts_cached: 5,
            current_month: (2024, 6),
        };
        let debug_str = format!("{:?}", stats);
        assert!(debug_str.contains("states_cached"));
        assert!(debug_str.contains("5"));
    }
}
