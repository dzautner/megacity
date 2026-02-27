//! Formatting helpers for save slot display.
//!
//! Pure functions for formatting save slot metadata into human-readable
//! strings. Used by both `save_slot_ui` and `main_menu` to consistently
//! display slot information.

use simulation::save_slots::SaveSlotInfo;

/// Format slot details for display (population, day, play time, timestamp).
pub fn format_slot_details(slot: &SaveSlotInfo) -> String {
    let play_hours = (slot.play_time_seconds / 3600.0) as u32;
    let play_mins = ((slot.play_time_seconds % 3600.0) / 60.0) as u32;

    let time_str = format_timestamp(slot.timestamp);

    format!(
        "Pop: {} | Day {} | {}h{}m played | {}",
        format_population(slot.population),
        slot.day,
        play_hours,
        play_mins,
        time_str,
    )
}

/// Format a population number with thousands separator.
pub fn format_population(pop: u32) -> String {
    if pop >= 1_000_000 {
        format!("{:.1}M", pop as f64 / 1_000_000.0)
    } else if pop >= 1_000 {
        format!("{:.1}K", pop as f64 / 1_000.0)
    } else {
        pop.to_string()
    }
}

/// Format a Unix timestamp into a human-readable date/time string.
pub fn format_timestamp(timestamp: u64) -> String {
    if timestamp == 0 {
        return "Unknown".to_string();
    }

    // Simple UTC-based formatting without external dependencies.
    let secs = timestamp;
    let days_since_epoch = secs / 86400;
    let time_of_day = secs % 86400;
    let hours = time_of_day / 3600;
    let minutes = (time_of_day % 3600) / 60;

    let (year, month, day) = days_to_ymd(days_since_epoch);

    format!(
        "{:04}-{:02}-{:02} {:02}:{:02}",
        year, month, day, hours, minutes
    )
}

/// Convert days since Unix epoch to (year, month, day).
fn days_to_ymd(days: u64) -> (u64, u64, u64) {
    // Algorithm adapted from Howard Hinnant's chrono-compatible date algo.
    let z = days + 719468;
    let era = z / 146097;
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    (y, m, d)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_population_small() {
        assert_eq!(format_population(0), "0");
        assert_eq!(format_population(999), "999");
    }

    #[test]
    fn test_format_population_thousands() {
        assert_eq!(format_population(1000), "1.0K");
        assert_eq!(format_population(50_000), "50.0K");
    }

    #[test]
    fn test_format_population_millions() {
        assert_eq!(format_population(1_000_000), "1.0M");
    }

    #[test]
    fn test_format_timestamp_zero() {
        assert_eq!(format_timestamp(0), "Unknown");
    }

    #[test]
    fn test_format_timestamp_known() {
        // 2024-01-01 00:00:00 UTC = 1704067200
        let result = format_timestamp(1704067200);
        assert_eq!(result, "2024-01-01 00:00");
    }

    #[test]
    fn test_days_to_ymd_epoch() {
        let (y, m, d) = days_to_ymd(0);
        assert_eq!((y, m, d), (1970, 1, 1));
    }

    #[test]
    fn test_days_to_ymd_2024() {
        // 2024-01-01 is day 19723
        let (y, m, d) = days_to_ymd(19723);
        assert_eq!((y, m, d), (2024, 1, 1));
    }
}
