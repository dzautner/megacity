use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::time_of_day::GameClock;
use crate::weather::Weather;

/// Baseline temperature for HDD/CDD calculation (18.3C = 65F).
const BASELINE_TEMP_C: f32 = 18.3;

/// HDD coefficient: each heating degree day increases HVAC demand by 2%.
const HDD_COEFFICIENT: f32 = 0.02;

/// CDD coefficient: each cooling degree day increases HVAC demand by 3%.
const CDD_COEFFICIENT: f32 = 0.03;

/// Number of months in a game year (360-day year / 30 days per month = 12 months).
const MONTHS_PER_YEAR: usize = 12;

/// Tracks Heating Degree Days (HDD) and Cooling Degree Days (CDD) for
/// HVAC energy demand calculations.
///
/// - `HDD = max(0, 18.3 - T_avg_C)` computed daily
/// - `CDD = max(0, T_avg_C - 18.3)` computed daily
/// - `daily_hvac_modifier = 1.0 + HDD * 0.02 + CDD * 0.03`
#[derive(Resource, Debug, Clone, Serialize, Deserialize)]
pub struct DegreeDays {
    /// Today's heating degree days.
    pub daily_hdd: f32,
    /// Today's cooling degree days.
    pub daily_cdd: f32,

    /// Monthly HDD accumulator (index 0 = month 1, etc.).
    pub monthly_hdd: [f32; MONTHS_PER_YEAR],
    /// Monthly CDD accumulator (index 0 = month 1, etc.).
    pub monthly_cdd: [f32; MONTHS_PER_YEAR],

    /// Cumulative annual HDD (resets each year).
    pub annual_hdd: f32,
    /// Cumulative annual CDD (resets each year).
    pub annual_cdd: f32,

    /// Last day that was processed (to avoid double-counting).
    pub last_update_day: u32,
}

impl Default for DegreeDays {
    fn default() -> Self {
        Self {
            daily_hdd: 0.0,
            daily_cdd: 0.0,
            monthly_hdd: [0.0; MONTHS_PER_YEAR],
            monthly_cdd: [0.0; MONTHS_PER_YEAR],
            annual_hdd: 0.0,
            annual_cdd: 0.0,
            last_update_day: 0,
        }
    }
}

impl DegreeDays {
    /// Compute the HVAC energy demand modifier from today's degree days.
    ///
    /// `modifier = 1.0 + daily_hdd * 0.02 + daily_cdd * 0.03`
    ///
    /// A mild day (18.3C) returns 1.0x. Cold or hot days return > 1.0x.
    pub fn hvac_modifier(&self) -> f32 {
        1.0 + self.daily_hdd * HDD_COEFFICIENT + self.daily_cdd * CDD_COEFFICIENT
    }

    /// Returns the month index (0-based) for a given game day.
    /// 360-day year, 30 days per month.
    fn month_index(day: u32) -> usize {
        let day_of_year = ((day.saturating_sub(1)) % 360) + 1;
        ((day_of_year - 1) / 30) as usize
    }

    /// Returns the year number (0-based) for a given game day.
    fn year_number(day: u32) -> u32 {
        (day.saturating_sub(1)) / 360
    }
}

/// System: compute daily HDD/CDD from the current weather temperature.
///
/// Runs once per game day (when `clock.day` changes). Uses the weather
/// temperature at the time of the day transition as the daily average
/// approximation.
pub fn update_degree_days(
    clock: Res<GameClock>,
    weather: Res<Weather>,
    mut degree_days: ResMut<DegreeDays>,
) {
    // Only update once per day
    if clock.day == degree_days.last_update_day {
        return;
    }

    // Detect year rollover: reset annual accumulators
    let prev_year = DegreeDays::year_number(degree_days.last_update_day);
    let curr_year = DegreeDays::year_number(clock.day);
    if curr_year != prev_year && degree_days.last_update_day > 0 {
        degree_days.annual_hdd = 0.0;
        degree_days.annual_cdd = 0.0;
        degree_days.monthly_hdd = [0.0; MONTHS_PER_YEAR];
        degree_days.monthly_cdd = [0.0; MONTHS_PER_YEAR];
    }

    degree_days.last_update_day = clock.day;

    let temp = weather.temperature;

    // HDD = max(0, 18.3 - T_avg)
    let hdd = (BASELINE_TEMP_C - temp).max(0.0);
    // CDD = max(0, T_avg - 18.3)
    let cdd = (temp - BASELINE_TEMP_C).max(0.0);

    degree_days.daily_hdd = hdd;
    degree_days.daily_cdd = cdd;

    // Accumulate into monthly buckets
    let month_idx = DegreeDays::month_index(clock.day);
    degree_days.monthly_hdd[month_idx] += hdd;
    degree_days.monthly_cdd[month_idx] += cdd;

    // Accumulate annual totals
    degree_days.annual_hdd += hdd;
    degree_days.annual_cdd += cdd;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hdd_30_gives_1_6x_modifier() {
        // 30 HDD day = 1.0 + 30 * 0.02 = 1.6x
        let dd = DegreeDays {
            daily_hdd: 30.0,
            daily_cdd: 0.0,
            ..Default::default()
        };
        let modifier = dd.hvac_modifier();
        assert!(
            (modifier - 1.6).abs() < 0.001,
            "Expected 1.6, got {}",
            modifier
        );
    }

    #[test]
    fn test_cdd_20_gives_1_6x_modifier() {
        // 20 CDD day = 1.0 + 20 * 0.03 = 1.6x
        let dd = DegreeDays {
            daily_hdd: 0.0,
            daily_cdd: 20.0,
            ..Default::default()
        };
        let modifier = dd.hvac_modifier();
        assert!(
            (modifier - 1.6).abs() < 0.001,
            "Expected 1.6, got {}",
            modifier
        );
    }

    #[test]
    fn test_zero_hdd_cdd_gives_1x_modifier() {
        // 0 HDD and 0 CDD = 1.0x
        let dd = DegreeDays {
            daily_hdd: 0.0,
            daily_cdd: 0.0,
            ..Default::default()
        };
        let modifier = dd.hvac_modifier();
        assert!(
            (modifier - 1.0).abs() < 0.001,
            "Expected 1.0, got {}",
            modifier
        );
    }

    #[test]
    fn test_hdd_computation_cold_day() {
        // Temperature = -11.7C => HDD = 18.3 - (-11.7) = 30.0
        let temp = -11.7_f32;
        let hdd = (BASELINE_TEMP_C - temp).max(0.0);
        let cdd = (temp - BASELINE_TEMP_C).max(0.0);
        assert!((hdd - 30.0).abs() < 0.01, "HDD should be 30.0, got {}", hdd);
        assert!(cdd.abs() < 0.001, "CDD should be 0.0, got {}", cdd);
    }

    #[test]
    fn test_cdd_computation_hot_day() {
        // Temperature = 38.3C => CDD = 38.3 - 18.3 = 20.0
        let temp = 38.3_f32;
        let hdd = (BASELINE_TEMP_C - temp).max(0.0);
        let cdd = (temp - BASELINE_TEMP_C).max(0.0);
        assert!(hdd.abs() < 0.001, "HDD should be 0.0, got {}", hdd);
        assert!((cdd - 20.0).abs() < 0.01, "CDD should be 20.0, got {}", cdd);
    }

    #[test]
    fn test_baseline_temp_gives_zero_hdd_cdd() {
        // At exactly 18.3C, both HDD and CDD should be 0
        let temp = BASELINE_TEMP_C;
        let hdd = (BASELINE_TEMP_C - temp).max(0.0);
        let cdd = (temp - BASELINE_TEMP_C).max(0.0);
        assert!(hdd.abs() < 0.001);
        assert!(cdd.abs() < 0.001);
    }

    #[test]
    fn test_month_index() {
        // Day 1-30 => month 0
        assert_eq!(DegreeDays::month_index(1), 0);
        assert_eq!(DegreeDays::month_index(30), 0);
        // Day 31-60 => month 1
        assert_eq!(DegreeDays::month_index(31), 1);
        assert_eq!(DegreeDays::month_index(60), 1);
        // Day 301-330 => month 10
        assert_eq!(DegreeDays::month_index(301), 10);
        assert_eq!(DegreeDays::month_index(330), 10);
        // Day 331-360 => month 11
        assert_eq!(DegreeDays::month_index(331), 11);
        assert_eq!(DegreeDays::month_index(360), 11);
        // Day 361 wraps => month 0 of next year
        assert_eq!(DegreeDays::month_index(361), 0);
    }

    #[test]
    fn test_year_number() {
        assert_eq!(DegreeDays::year_number(1), 0);
        assert_eq!(DegreeDays::year_number(360), 0);
        assert_eq!(DegreeDays::year_number(361), 1);
        assert_eq!(DegreeDays::year_number(720), 1);
        assert_eq!(DegreeDays::year_number(721), 2);
    }

    #[test]
    fn test_combined_hdd_cdd_modifier() {
        // Both HDD and CDD contribute (shouldn't happen in practice, but the math works)
        let dd = DegreeDays {
            daily_hdd: 10.0,
            daily_cdd: 5.0,
            ..Default::default()
        };
        // 1.0 + 10*0.02 + 5*0.03 = 1.0 + 0.2 + 0.15 = 1.35
        let modifier = dd.hvac_modifier();
        assert!(
            (modifier - 1.35).abs() < 0.001,
            "Expected 1.35, got {}",
            modifier
        );
    }

    #[test]
    fn test_monthly_accumulation() {
        let mut dd = DegreeDays::default();

        // Simulate day 1 (month 0) with cold temp
        dd.last_update_day = 1;
        dd.daily_hdd = 10.0;
        dd.daily_cdd = 0.0;
        let month_idx = DegreeDays::month_index(1);
        dd.monthly_hdd[month_idx] += dd.daily_hdd;
        dd.monthly_cdd[month_idx] += dd.daily_cdd;
        dd.annual_hdd += dd.daily_hdd;

        // Simulate day 2 (still month 0)
        dd.last_update_day = 2;
        dd.daily_hdd = 15.0;
        let month_idx = DegreeDays::month_index(2);
        dd.monthly_hdd[month_idx] += dd.daily_hdd;
        dd.annual_hdd += dd.daily_hdd;

        assert!((dd.monthly_hdd[0] - 25.0).abs() < 0.001);
        assert!((dd.annual_hdd - 25.0).abs() < 0.001);
    }

    #[test]
    fn test_default_values() {
        let dd = DegreeDays::default();
        assert_eq!(dd.daily_hdd, 0.0);
        assert_eq!(dd.daily_cdd, 0.0);
        assert_eq!(dd.annual_hdd, 0.0);
        assert_eq!(dd.annual_cdd, 0.0);
        assert_eq!(dd.last_update_day, 0);
        for i in 0..12 {
            assert_eq!(dd.monthly_hdd[i], 0.0);
            assert_eq!(dd.monthly_cdd[i], 0.0);
        }
    }
}

pub struct DegreeDaysPlugin;

impl Plugin for DegreeDaysPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DegreeDays>().add_systems(
            FixedUpdate,
            update_degree_days
                .after(crate::imports_exports::process_trade)
                .in_set(crate::SimulationSet::Simulation),
        );
    }
}
