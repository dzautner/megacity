//! SERV-010: Daycare and Eldercare Services
//!
//! Daycare buildings enable parents to work (increases workforce participation).
//! Eldercare buildings keep seniors healthy longer (reduces deathcare load).
//! Both increase happiness for citizens within their coverage radius.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::citizen::{Citizen, CitizenDetails, HomeLocation};
use crate::config::{CELL_SIZE, GRID_HEIGHT, GRID_WIDTH};
use crate::services::{ServiceBuilding, ServiceType};

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Happiness bonus for citizens within daycare coverage.
pub const DAYCARE_HAPPINESS_BONUS: f32 = 4.0;

/// Happiness bonus for citizens within eldercare coverage.
pub const ELDERCARE_HAPPINESS_BONUS: f32 = 3.0;

/// Health bonus per slow tick for elderly citizens within eldercare coverage.
pub const ELDERCARE_HEALTH_BONUS: f32 = 2.0;

/// Monthly maintenance cost multiplier (already defined per-type in ServiceBuilding).
pub const DAYCARE_MONTHLY_MAINTENANCE: f64 = 15.0;
pub const ELDERCARE_MONTHLY_MAINTENANCE: f64 = 20.0;

/// Age threshold for elderly citizen health bonus.
pub const ELDERLY_AGE_THRESHOLD: u8 = 65;

// ---------------------------------------------------------------------------
// Coverage Grid
// ---------------------------------------------------------------------------

/// Per-cell coverage grid for daycare and eldercare services.
/// Bit 0 = daycare coverage, Bit 1 = eldercare coverage.
#[derive(Resource)]
pub struct DaycareEldercareCoverage {
    pub flags: Vec<u8>,
    pub dirty: bool,
}

const COVERAGE_DAYCARE: u8 = 0b01;
const COVERAGE_ELDERCARE: u8 = 0b10;

impl Default for DaycareEldercareCoverage {
    fn default() -> Self {
        Self {
            flags: vec![0; GRID_WIDTH * GRID_HEIGHT],
            dirty: true,
        }
    }
}

impl DaycareEldercareCoverage {
    #[inline]
    pub fn idx(x: usize, y: usize) -> usize {
        y * GRID_WIDTH + x
    }

    pub fn clear(&mut self) {
        self.flags.fill(0);
    }

    #[inline]
    pub fn has_daycare(&self, x: usize, y: usize) -> bool {
        self.flags[Self::idx(x, y)] & COVERAGE_DAYCARE != 0
    }

    #[inline]
    pub fn has_eldercare(&self, x: usize, y: usize) -> bool {
        self.flags[Self::idx(x, y)] & COVERAGE_ELDERCARE != 0
    }

    /// Count of cells with daycare coverage.
    pub fn daycare_covered_cells(&self) -> usize {
        self.flags
            .iter()
            .filter(|&&f| f & COVERAGE_DAYCARE != 0)
            .count()
    }

    /// Count of cells with eldercare coverage.
    pub fn eldercare_covered_cells(&self) -> usize {
        self.flags
            .iter()
            .filter(|&&f| f & COVERAGE_ELDERCARE != 0)
            .count()
    }
}

// ---------------------------------------------------------------------------
// Saveable State (city-wide stats)
// ---------------------------------------------------------------------------

#[derive(Resource, Debug, Clone, Default, Serialize, Deserialize)]
pub struct DaycareEldercareState {
    /// Number of daycare buildings in the city.
    pub daycare_count: u32,
    /// Number of eldercare buildings in the city.
    pub eldercare_count: u32,
    /// Citizens currently covered by daycare.
    pub daycare_covered_citizens: u32,
    /// Citizens currently covered by eldercare.
    pub eldercare_covered_citizens: u32,
    /// Total monthly maintenance cost for care services.
    pub monthly_maintenance: f64,
}

impl crate::Saveable for DaycareEldercareState {
    const SAVE_KEY: &'static str = "daycare_eldercare";

    fn save_to_bytes(&self) -> Option<Vec<u8>> {
        Some(bitcode::encode(self))
    }

    fn load_from_bytes(bytes: &[u8]) -> Self {
        crate::decode_or_warn(Self::SAVE_KEY, bytes)
    }
}

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

/// Recompute coverage grid when service buildings change.
pub fn update_daycare_eldercare_coverage(
    services: Query<&ServiceBuilding>,
    added_services: Query<Entity, Added<ServiceBuilding>>,
    mut coverage: ResMut<DaycareEldercareCoverage>,
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
        let bits = match service.service_type {
            ServiceType::Daycare => COVERAGE_DAYCARE,
            ServiceType::Eldercare => COVERAGE_ELDERCARE,
            _ => continue,
        };

        let radius_cells = (service.radius / CELL_SIZE).ceil() as i32;
        let sx = service.grid_x as i32;
        let sy = service.grid_y as i32;
        let r2 = service.radius * service.radius;

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
                let idx = DaycareEldercareCoverage::idx(cx as usize, cy as usize);
                coverage.flags[idx] |= bits;
            }
        }
    }
}

/// Apply eldercare health bonus to elderly citizens within coverage.
/// Also update daycare/eldercare stats and apply happiness bonuses.
#[allow(clippy::too_many_arguments)]
pub fn apply_daycare_eldercare_effects(
    slow_timer: Res<crate::SlowTickTimer>,
    coverage: Res<DaycareEldercareCoverage>,
    services: Query<&ServiceBuilding>,
    mut state: ResMut<DaycareEldercareState>,
    mut citizens: Query<(&mut CitizenDetails, &HomeLocation), With<Citizen>>,
) {
    if !slow_timer.should_run() {
        return;
    }

    // Count service buildings
    let mut daycare_count: u32 = 0;
    let mut eldercare_count: u32 = 0;
    let mut total_maintenance: f64 = 0.0;

    for service in &services {
        match service.service_type {
            ServiceType::Daycare => {
                daycare_count += 1;
                total_maintenance += DAYCARE_MONTHLY_MAINTENANCE;
            }
            ServiceType::Eldercare => {
                eldercare_count += 1;
                total_maintenance += ELDERCARE_MONTHLY_MAINTENANCE;
            }
            _ => {}
        }
    }

    state.daycare_count = daycare_count;
    state.eldercare_count = eldercare_count;
    state.monthly_maintenance = total_maintenance;

    // Apply effects to citizens
    let mut daycare_covered: u32 = 0;
    let mut eldercare_covered: u32 = 0;

    for (mut details, home) in &mut citizens {
        let has_daycare = coverage.has_daycare(home.grid_x, home.grid_y);
        let has_eldercare = coverage.has_eldercare(home.grid_x, home.grid_y);

        if has_daycare {
            daycare_covered += 1;
            // Happiness bonus for daycare coverage (working parents benefit)
            details.happiness = (details.happiness + DAYCARE_HAPPINESS_BONUS).min(100.0);
        }

        if has_eldercare {
            eldercare_covered += 1;
            // Happiness bonus for eldercare coverage
            details.happiness = (details.happiness + ELDERCARE_HAPPINESS_BONUS).min(100.0);

            // Health bonus for elderly citizens — slows aging/health decline
            if details.age >= ELDERLY_AGE_THRESHOLD {
                details.health = (details.health + ELDERCARE_HEALTH_BONUS).min(100.0);
            }
        }
    }

    state.daycare_covered_citizens = daycare_covered;
    state.eldercare_covered_citizens = eldercare_covered;
}

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

pub struct DaycareEldercarePlugin;

impl Plugin for DaycareEldercarePlugin {
    fn build(&self, app: &mut App) {
        use save::SaveableAppExt;

        app.init_resource::<DaycareEldercareCoverage>()
            .init_resource::<DaycareEldercareState>()
            .register_saveable::<DaycareEldercareState>()
            .add_systems(
                FixedUpdate,
                (
                    update_daycare_eldercare_coverage,
                    apply_daycare_eldercare_effects
                        .after(update_daycare_eldercare_coverage),
                )
                    .in_set(crate::SimulationSet::Simulation),
            );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_coverage_grid_default_is_clear() {
        let cov = DaycareEldercareCoverage::default();
        assert_eq!(cov.flags.len(), GRID_WIDTH * GRID_HEIGHT);
        assert!(cov.flags.iter().all(|&f| f == 0));
    }

    #[test]
    fn test_coverage_has_daycare() {
        let mut cov = DaycareEldercareCoverage::default();
        assert!(!cov.has_daycare(10, 10));
        let idx = DaycareEldercareCoverage::idx(10, 10);
        cov.flags[idx] |= COVERAGE_DAYCARE;
        assert!(cov.has_daycare(10, 10));
        assert!(!cov.has_eldercare(10, 10));
    }

    #[test]
    fn test_coverage_has_eldercare() {
        let mut cov = DaycareEldercareCoverage::default();
        assert!(!cov.has_eldercare(10, 10));
        let idx = DaycareEldercareCoverage::idx(10, 10);
        cov.flags[idx] |= COVERAGE_ELDERCARE;
        assert!(cov.has_eldercare(10, 10));
        assert!(!cov.has_daycare(10, 10));
    }

    #[test]
    fn test_coverage_clear() {
        let mut cov = DaycareEldercareCoverage::default();
        let idx = DaycareEldercareCoverage::idx(5, 5);
        cov.flags[idx] = COVERAGE_DAYCARE | COVERAGE_ELDERCARE;
        cov.clear();
        assert!(!cov.has_daycare(5, 5));
        assert!(!cov.has_eldercare(5, 5));
    }

    #[test]
    fn test_covered_cells_count() {
        let mut cov = DaycareEldercareCoverage::default();
        assert_eq!(cov.daycare_covered_cells(), 0);
        assert_eq!(cov.eldercare_covered_cells(), 0);

        let idx1 = DaycareEldercareCoverage::idx(1, 1);
        let idx2 = DaycareEldercareCoverage::idx(2, 2);
        cov.flags[idx1] = COVERAGE_DAYCARE;
        cov.flags[idx2] = COVERAGE_DAYCARE | COVERAGE_ELDERCARE;
        assert_eq!(cov.daycare_covered_cells(), 2);
        assert_eq!(cov.eldercare_covered_cells(), 1);
    }

    #[test]
    fn test_state_default() {
        let state = DaycareEldercareState::default();
        assert_eq!(state.daycare_count, 0);
        assert_eq!(state.eldercare_count, 0);
        assert_eq!(state.daycare_covered_citizens, 0);
        assert_eq!(state.eldercare_covered_citizens, 0);
        assert_eq!(state.monthly_maintenance, 0.0);
    }

    #[test]
    fn test_saveable_roundtrip() {
        use crate::Saveable;
        let mut state = DaycareEldercareState::default();
        state.daycare_count = 3;
        state.eldercare_count = 2;
        state.daycare_covered_citizens = 150;
        state.eldercare_covered_citizens = 80;
        state.monthly_maintenance = 105.0;
        let bytes = state.save_to_bytes().expect("should serialize");
        let restored = DaycareEldercareState::load_from_bytes(&bytes);
        assert_eq!(restored.daycare_count, 3);
        assert_eq!(restored.eldercare_count, 2);
        assert_eq!(restored.daycare_covered_citizens, 150);
        assert_eq!(restored.eldercare_covered_citizens, 80);
        assert!((restored.monthly_maintenance - 105.0).abs() < 0.001);
    }

    #[test]
    fn test_service_type_names() {
        assert_eq!(ServiceType::Daycare.name(), "Daycare");
        assert_eq!(ServiceType::Eldercare.name(), "Eldercare");
    }

    #[test]
    fn test_coverage_radius() {
        let daycare_radius = ServiceBuilding::coverage_radius(ServiceType::Daycare);
        let eldercare_radius = ServiceBuilding::coverage_radius(ServiceType::Eldercare);
        assert!(daycare_radius > 0.0);
        assert!(eldercare_radius > 0.0);
        assert!(daycare_radius > eldercare_radius); // daycare has larger radius
    }

    #[test]
    fn test_monthly_maintenance_costs() {
        let daycare_maint = ServiceBuilding::monthly_maintenance(ServiceType::Daycare);
        let eldercare_maint = ServiceBuilding::monthly_maintenance(ServiceType::Eldercare);
        assert!(daycare_maint > 0.0);
        assert!(eldercare_maint > 0.0);
    }

    #[test]
    fn test_building_cost() {
        let daycare_cost = ServiceBuilding::cost(ServiceType::Daycare);
        let eldercare_cost = ServiceBuilding::cost(ServiceType::Eldercare);
        assert!(daycare_cost > 0.0);
        assert!(eldercare_cost > 0.0);
    }
}
