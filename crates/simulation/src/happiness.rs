use bevy::prelude::*;

use crate::citizen::{Citizen, CitizenDetails, HomeLocation, Needs, WorkLocation};
use crate::config::{CELL_SIZE, GRID_HEIGHT, GRID_WIDTH};
use crate::economy::CityBudget;
use crate::grid::WorldGrid;
use crate::policies::Policies;
use crate::services::{ServiceBuilding, ServiceType};
use crate::traffic::TrafficGrid;
use crate::weather::Weather;

/// Bitflags for service coverage packed into a single byte per cell.
pub const COVERAGE_HEALTH: u8        = 0b0000_0001;
pub const COVERAGE_EDUCATION: u8     = 0b0000_0010;
pub const COVERAGE_POLICE: u8        = 0b0000_0100;
pub const COVERAGE_PARK: u8          = 0b0000_1000;
pub const COVERAGE_ENTERTAINMENT: u8 = 0b0001_0000;

/// Per-cell service coverage flags, precomputed when service buildings change.
/// Uses bitflags packed into a single Vec<u8> — 5x less memory than 5 separate Vec<bool>.
#[derive(Resource)]
pub struct ServiceCoverageGrid {
    /// One byte per cell, with bits for each service type.
    pub flags: Vec<u8>,
    pub dirty: bool,
}

impl Default for ServiceCoverageGrid {
    fn default() -> Self {
        let n = GRID_WIDTH * GRID_HEIGHT;
        Self {
            flags: vec![0; n],
            dirty: true,
        }
    }
}

impl ServiceCoverageGrid {
    pub fn clear(&mut self) {
        self.flags.fill(0);
    }

    pub fn idx(x: usize, y: usize) -> usize {
        y * GRID_WIDTH + x
    }

    #[inline]
    pub fn has_health(&self, idx: usize) -> bool {
        self.flags[idx] & COVERAGE_HEALTH != 0
    }
    #[inline]
    pub fn has_education(&self, idx: usize) -> bool {
        self.flags[idx] & COVERAGE_EDUCATION != 0
    }
    #[inline]
    pub fn has_police(&self, idx: usize) -> bool {
        self.flags[idx] & COVERAGE_POLICE != 0
    }
    #[inline]
    pub fn has_park(&self, idx: usize) -> bool {
        self.flags[idx] & COVERAGE_PARK != 0
    }
    #[inline]
    pub fn has_entertainment(&self, idx: usize) -> bool {
        self.flags[idx] & COVERAGE_ENTERTAINMENT != 0
    }
}

pub fn update_service_coverage(
    services: Query<&ServiceBuilding>,
    added_services: Query<Entity, Added<ServiceBuilding>>,
    mut coverage: ResMut<ServiceCoverageGrid>,
) {
    if !added_services.is_empty() {
        coverage.dirty = true;
    }
    if !coverage.dirty {
        return;
    }
    coverage.dirty = false;
    coverage.clear();

    for service in &services {
        let radius_cells = (service.radius / CELL_SIZE).ceil() as i32;
        let sx = service.grid_x as i32;
        let sy = service.grid_y as i32;
        let r2 = service.radius * service.radius;

        // Determine which coverage bits this service sets
        let bits = match service.service_type {
            ServiceType::Hospital | ServiceType::MedicalClinic | ServiceType::MedicalCenter => COVERAGE_HEALTH,
            ServiceType::ElementarySchool
            | ServiceType::HighSchool
            | ServiceType::University
            | ServiceType::Library
            | ServiceType::Kindergarten => COVERAGE_EDUCATION,
            ServiceType::PoliceStation | ServiceType::PoliceKiosk | ServiceType::PoliceHQ | ServiceType::Prison => COVERAGE_POLICE,
            ServiceType::SmallPark | ServiceType::LargePark | ServiceType::Playground => COVERAGE_PARK,
            ServiceType::Stadium | ServiceType::Plaza | ServiceType::SportsField => COVERAGE_ENTERTAINMENT,
            _ => continue,
        };

        for dy in -radius_cells..=radius_cells {
            for dx in -radius_cells..=radius_cells {
                let cx = sx + dx;
                let cy = sy + dy;
                if cx < 0 || cy < 0 || cx >= GRID_WIDTH as i32 || cy >= GRID_HEIGHT as i32 {
                    continue;
                }
                let wx_diff = dx as f32 * CELL_SIZE;
                let wy_diff = dy as f32 * CELL_SIZE;
                if wx_diff * wx_diff + wy_diff * wy_diff > r2 {
                    continue;
                }
                let idx = ServiceCoverageGrid::idx(cx as usize, cy as usize);
                coverage.flags[idx] |= bits;
            }
        }
    }
}

const BASE_HAPPINESS: f32 = 50.0;
const EMPLOYED_BONUS: f32 = 15.0;
const SHORT_COMMUTE_BONUS: f32 = 10.0;
const POWER_BONUS: f32 = 5.0;
const WATER_BONUS: f32 = 5.0;
const HEALTH_COVERAGE_BONUS: f32 = 5.0;
const EDUCATION_BONUS: f32 = 3.0;
const POLICE_BONUS: f32 = 5.0;
const PARK_BONUS: f32 = 8.0;
const ENTERTAINMENT_BONUS: f32 = 5.0;
const HIGH_TAX_PENALTY: f32 = 8.0;
const CONGESTION_PENALTY: f32 = 5.0;
const GARBAGE_PENALTY: f32 = 5.0;

#[allow(clippy::too_many_arguments)]
pub fn update_happiness(
    tick: Res<crate::TickCounter>,
    grid: Res<WorldGrid>,
    budget: Res<CityBudget>,
    traffic: Res<TrafficGrid>,
    pollution_grid: Res<crate::pollution::PollutionGrid>,
    garbage_grid: Res<crate::garbage::GarbageGrid>,
    land_value_grid: Res<crate::land_value::LandValueGrid>,
    policies: Res<Policies>,
    weather: Res<Weather>,
    coverage: Res<ServiceCoverageGrid>,
    mut citizens: Query<
        (&mut CitizenDetails, &HomeLocation, Option<&WorkLocation>, Option<&Needs>),
        With<Citizen>,
    >,
) {
    if !tick.0.is_multiple_of(10) {
        return;
    }
    let tax_penalty = if budget.tax_rate > 0.15 {
        HIGH_TAX_PENALTY * ((budget.tax_rate - 0.15) / 0.10)
    } else {
        0.0
    };

    // Pre-compute shared values to avoid redundant reads per citizen
    let policy_bonus = policies.happiness_bonus();
    let weather_mod = weather.happiness_modifier();

    citizens.par_iter_mut().for_each(|(mut details, home, work, needs)| {
        let mut happiness = BASE_HAPPINESS;

        // Employment
        if work.is_some() {
            happiness += EMPLOYED_BONUS;
        }

        // Commute distance (short = close home to work)
        if let Some(work_loc) = work {
            let dx = (home.grid_x as i32 - work_loc.grid_x as i32).abs();
            let dy = (home.grid_y as i32 - work_loc.grid_y as i32).abs();
            let dist = dx + dy;
            if dist < 20 {
                happiness += SHORT_COMMUTE_BONUS;
            }
        }

        // Utilities at home
        let home_cell = grid.get(home.grid_x, home.grid_y);
        if home_cell.has_power {
            happiness += POWER_BONUS;
        }
        if home_cell.has_water {
            happiness += WATER_BONUS;
        }

        // Service coverage (O(1) bitflag lookup from precomputed grid)
        let idx = ServiceCoverageGrid::idx(home.grid_x, home.grid_y);
        let cov = coverage.flags[idx];
        if cov & COVERAGE_HEALTH != 0 {
            happiness += HEALTH_COVERAGE_BONUS;
        }
        if cov & COVERAGE_EDUCATION != 0 {
            happiness += EDUCATION_BONUS;
        }
        if cov & COVERAGE_POLICE != 0 {
            happiness += POLICE_BONUS;
        }
        if cov & COVERAGE_PARK != 0 {
            happiness += PARK_BONUS;
        }
        if cov & COVERAGE_ENTERTAINMENT != 0 {
            happiness += ENTERTAINMENT_BONUS;
        }

        // Pollution penalty
        let pollution = pollution_grid.get(home.grid_x, home.grid_y) as f32;
        happiness -= pollution / 25.0;

        // Garbage penalty
        if garbage_grid.get(home.grid_x, home.grid_y) > 10 {
            happiness -= GARBAGE_PENALTY;
        }

        // Land value bonus
        let land_value = land_value_grid.get(home.grid_x, home.grid_y) as f32;
        happiness += land_value / 50.0;

        // Traffic congestion near home
        let congestion = traffic.congestion_level(home.grid_x, home.grid_y);
        happiness -= congestion * CONGESTION_PENALTY;

        // Tax penalty
        happiness -= tax_penalty;

        // Policy and weather bonuses (pre-computed)
        happiness += policy_bonus;
        happiness += weather_mod;

        // Needs satisfaction (if citizen has needs component)
        if let Some(needs) = needs {
            let satisfaction = needs.overall_satisfaction();
            happiness += (satisfaction - 0.5) * 35.0;
        }

        // Health affects happiness
        if details.health < 50.0 {
            happiness -= (50.0 - details.health) * 0.3;
        }
        if details.health > 80.0 {
            happiness += 3.0;
        }

        details.happiness = happiness.clamp(0.0, 100.0);
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_happiness_bounds() {
        // Base happiness should be in range
        assert!(BASE_HAPPINESS >= 0.0 && BASE_HAPPINESS <= 100.0);
    }

    #[test]
    fn test_all_factors_affect_output() {
        // Verify all bonuses/penalties are non-zero
        assert!(EMPLOYED_BONUS > 0.0);
        assert!(SHORT_COMMUTE_BONUS > 0.0);
        assert!(POWER_BONUS > 0.0);
        assert!(WATER_BONUS > 0.0);
        assert!(HEALTH_COVERAGE_BONUS > 0.0);
        assert!(EDUCATION_BONUS > 0.0);
        assert!(POLICE_BONUS > 0.0);
        assert!(PARK_BONUS > 0.0);
        assert!(ENTERTAINMENT_BONUS > 0.0);
        assert!(HIGH_TAX_PENALTY > 0.0);
        assert!(CONGESTION_PENALTY > 0.0);
        assert!(GARBAGE_PENALTY > 0.0);
    }

    #[test]
    fn test_max_happiness_reachable() {
        // Max theoretical happiness: all bonuses, no penalties, max land value (255/50 = 5.1)
        let max_land_bonus: f32 = 255.0 / 50.0;
        let max = BASE_HAPPINESS + EMPLOYED_BONUS + SHORT_COMMUTE_BONUS
            + POWER_BONUS + WATER_BONUS + HEALTH_COVERAGE_BONUS + EDUCATION_BONUS
            + POLICE_BONUS + PARK_BONUS + ENTERTAINMENT_BONUS + max_land_bonus;
        // With all bonuses the raw sum exceeds 100, but clamp caps at 100
        assert!(max > 100.0, "max happiness {} should exceed 100 before clamping", max);
        // Verify it is meaningful without land value
        let max_no_land = BASE_HAPPINESS + EMPLOYED_BONUS + SHORT_COMMUTE_BONUS
            + POWER_BONUS + WATER_BONUS + HEALTH_COVERAGE_BONUS + EDUCATION_BONUS
            + POLICE_BONUS + PARK_BONUS + ENTERTAINMENT_BONUS;
        assert!(max_no_land > 80.0, "max happiness {} (no land) should be > 80 to be meaningful", max_no_land);
    }

    #[test]
    fn test_service_coverage_dirty_flag_default() {
        let grid = ServiceCoverageGrid::default();
        assert!(grid.dirty, "should start dirty so first update runs");
    }

    #[test]
    fn test_service_coverage_clear_resets_all() {
        let mut grid = ServiceCoverageGrid::default();
        let idx = ServiceCoverageGrid::idx(10, 10);
        grid.flags[idx] = COVERAGE_HEALTH | COVERAGE_EDUCATION | COVERAGE_POLICE | COVERAGE_PARK | COVERAGE_ENTERTAINMENT;
        grid.clear();
        assert!(!grid.has_health(idx));
        assert!(!grid.has_education(idx));
        assert!(!grid.has_police(idx));
        assert!(!grid.has_park(idx));
        assert!(!grid.has_entertainment(idx));
    }
}
