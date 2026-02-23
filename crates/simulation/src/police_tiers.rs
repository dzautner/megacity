//! SVC-005: Police Service Multi-Tier System
//!
//! Implements a three-tier police system with varying coverage, crime
//! reduction, response time, and maintenance cost:
//!
//! - **Small Police Station (PoliceKiosk):** Local patrol, low cost, small
//!   radius, basic crime reduction.
//! - **Police Station:** Standard coverage, medium cost, moderate crime
//!   reduction and response time.
//! - **Police HQ:** City-wide coordination bonus, large radius, high crime
//!   reduction, fast response, highest maintenance.
//!
//! The system tracks per-tier statistics and applies differentiated crime
//! reduction to the `CrimeGrid`. A coordination bonus is applied when a
//! Police HQ exists, boosting all lower-tier stations in the city.

use bevy::prelude::*;
use bitcode::{Decode, Encode};
use serde::{Deserialize, Serialize};

use crate::config::{GRID_HEIGHT, GRID_WIDTH};
use crate::crime::CrimeGrid;
use crate::services::{ServiceBuilding, ServiceType};
use crate::Saveable;

// ---------------------------------------------------------------------------
// Tier definitions
// ---------------------------------------------------------------------------

/// Police tier with associated gameplay parameters.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Encode, Decode, Serialize, Deserialize)]
pub enum PoliceTier {
    /// Small Police Station (kiosk). Local patrol, low cost.
    Kiosk,
    /// Standard Police Station. Balanced coverage.
    Station,
    /// Police HQ. City-wide coordination, large radius.
    Headquarters,
}

impl PoliceTier {
    /// Coverage radius in grid cells.
    pub fn coverage_radius(self) -> i32 {
        match self {
            Self::Kiosk => 10,
            Self::Station => 20,
            Self::Headquarters => 35,
        }
    }

    /// Base crime reduction applied within coverage (higher = stronger).
    pub fn crime_reduction(self) -> u8 {
        match self {
            Self::Kiosk => 8,
            Self::Station => 18,
            Self::Headquarters => 30,
        }
    }

    /// Response time in slow ticks (lower = faster response).
    pub fn response_time(self) -> u32 {
        match self {
            Self::Kiosk => 3,
            Self::Station => 2,
            Self::Headquarters => 1,
        }
    }

    /// Monthly maintenance cost (currency units).
    pub fn maintenance_cost(self) -> f64 {
        match self {
            Self::Kiosk => 8.0,
            Self::Station => 20.0,
            Self::Headquarters => 60.0,
        }
    }

    /// Map from `ServiceType` to police tier, if applicable.
    pub fn from_service_type(st: ServiceType) -> Option<Self> {
        match st {
            ServiceType::PoliceKiosk => Some(Self::Kiosk),
            ServiceType::PoliceStation => Some(Self::Station),
            ServiceType::PoliceHQ => Some(Self::Headquarters),
            _ => None,
        }
    }
}

// ---------------------------------------------------------------------------
// Per-tier stats
// ---------------------------------------------------------------------------

/// Statistics tracked per police tier.
#[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
pub struct TierStats {
    /// Number of active buildings of this tier.
    pub building_count: u32,
    /// Total grid cells covered by this tier's buildings.
    pub cells_covered: u32,
    /// Cumulative crime points reduced this tick.
    pub crime_reduced: u32,
    /// Total monthly maintenance cost for this tier.
    pub total_maintenance: f64,
}

// ---------------------------------------------------------------------------
// Main resource
// ---------------------------------------------------------------------------

/// Coordination bonus multiplier when at least one Police HQ exists.
const HQ_COORDINATION_BONUS: f32 = 1.25;

/// Main state resource for the police tier system.
#[derive(Resource, Clone, Debug, Encode, Decode, Serialize, Deserialize)]
pub struct PoliceTiersState {
    /// Stats for the Kiosk (small) tier.
    pub kiosk_stats: TierStats,
    /// Stats for the Station (standard) tier.
    pub station_stats: TierStats,
    /// Stats for the HQ tier.
    pub hq_stats: TierStats,
    /// Whether the HQ coordination bonus is currently active.
    pub coordination_active: bool,
    /// Effective coordination multiplier (1.0 = no bonus).
    pub coordination_multiplier: f32,
    /// Overall police coverage ratio (0.0 to 1.0).
    pub city_coverage: f32,
}

impl Default for PoliceTiersState {
    fn default() -> Self {
        Self {
            kiosk_stats: TierStats::default(),
            station_stats: TierStats::default(),
            hq_stats: TierStats::default(),
            coordination_active: false,
            coordination_multiplier: 1.0,
            city_coverage: 0.0,
        }
    }
}

impl PoliceTiersState {
    /// Get stats for a specific tier.
    pub fn stats_for_tier(&self, tier: PoliceTier) -> &TierStats {
        match tier {
            PoliceTier::Kiosk => &self.kiosk_stats,
            PoliceTier::Station => &self.station_stats,
            PoliceTier::Headquarters => &self.hq_stats,
        }
    }

    fn stats_for_tier_mut(&mut self, tier: PoliceTier) -> &mut TierStats {
        match tier {
            PoliceTier::Kiosk => &mut self.kiosk_stats,
            PoliceTier::Station => &mut self.station_stats,
            PoliceTier::Headquarters => &mut self.hq_stats,
        }
    }

    /// Total building count across all tiers.
    pub fn total_buildings(&self) -> u32 {
        self.kiosk_stats.building_count
            + self.station_stats.building_count
            + self.hq_stats.building_count
    }

    /// Total monthly maintenance across all tiers.
    pub fn total_maintenance(&self) -> f64 {
        self.kiosk_stats.total_maintenance
            + self.station_stats.total_maintenance
            + self.hq_stats.total_maintenance
    }
}

impl Saveable for PoliceTiersState {
    const SAVE_KEY: &'static str = "police_tiers";
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

/// Gather police service buildings, compute per-tier stats, and apply
/// differentiated crime reduction to the `CrimeGrid`.
#[allow(clippy::too_many_arguments)]
pub fn update_police_tiers(
    slow_timer: Res<crate::SlowTickTimer>,
    services: Query<&ServiceBuilding>,
    ext_budget: Res<crate::budget::ExtendedBudget>,
    mut state: ResMut<PoliceTiersState>,
    mut crime_grid: ResMut<CrimeGrid>,
) {
    if !slow_timer.should_run() {
        return;
    }

    // Reset per-tick stats.
    state.kiosk_stats = TierStats::default();
    state.station_stats = TierStats::default();
    state.hq_stats = TierStats::default();

    let police_budget = ext_budget.service_budgets.police;

    // Collect police buildings by tier.
    struct PoliceUnit {
        tier: PoliceTier,
        grid_x: usize,
        grid_y: usize,
    }

    let mut units: Vec<PoliceUnit> = Vec::new();
    for service in &services {
        if let Some(tier) = PoliceTier::from_service_type(service.service_type) {
            let ts = state.stats_for_tier_mut(tier);
            ts.building_count += 1;
            ts.total_maintenance += tier.maintenance_cost();
            units.push(PoliceUnit {
                tier,
                grid_x: service.grid_x,
                grid_y: service.grid_y,
            });
        }
    }

    // Determine coordination bonus.
    let has_hq = state.hq_stats.building_count > 0;
    state.coordination_active = has_hq;
    state.coordination_multiplier = if has_hq { HQ_COORDINATION_BONUS } else { 1.0 };

    let coord_mult = state.coordination_multiplier;

    // Track which cells are covered (for city_coverage stat).
    let total_cells = GRID_WIDTH * GRID_HEIGHT;
    let mut covered = vec![false; total_cells];
    let mut total_crime_reduced: u32 = 0;

    // Apply crime reduction per unit.
    for unit in &units {
        let radius = unit.tier.coverage_radius();
        let base_reduction = unit.tier.crime_reduction();
        // HQ coordination bonus applies to non-HQ tiers when HQ exists.
        let tier_multiplier = if has_hq && unit.tier != PoliceTier::Headquarters {
            coord_mult
        } else {
            1.0
        };
        let effective_reduction =
            (base_reduction as f32 * police_budget * tier_multiplier) as u8;

        let mut tier_reduced: u32 = 0;
        let mut tier_cells: u32 = 0;

        let gx = unit.grid_x as i32;
        let gy = unit.grid_y as i32;
        for dy in -radius..=radius {
            for dx in -radius..=radius {
                let nx = gx + dx;
                let ny = gy + dy;
                if nx < 0
                    || ny < 0
                    || (nx as usize) >= GRID_WIDTH
                    || (ny as usize) >= GRID_HEIGHT
                {
                    continue;
                }
                let ux = nx as usize;
                let uy = ny as usize;
                let dist = dx.abs() + dy.abs();
                // Falloff: reduction decreases with Manhattan distance.
                let falloff_reduction = effective_reduction.saturating_sub(
                    (dist as f32 / radius as f32 * effective_reduction as f32) as u8,
                );
                if falloff_reduction == 0 {
                    continue;
                }

                let idx = uy * GRID_WIDTH + ux;
                let old = crime_grid.levels[idx];
                crime_grid.levels[idx] = old.saturating_sub(falloff_reduction);
                let actual = old - crime_grid.levels[idx];
                tier_reduced += actual as u32;
                tier_cells += 1;
                covered[idx] = true;
            }
        }

        let ts = state.stats_for_tier_mut(unit.tier);
        ts.cells_covered += tier_cells;
        ts.crime_reduced += tier_reduced;
        total_crime_reduced += tier_reduced;
    }

    let _ = total_crime_reduced; // used implicitly through tier stats

    // Compute city coverage ratio.
    let covered_count = covered.iter().filter(|&&c| c).count();
    state.city_coverage = covered_count as f32 / total_cells as f32;
}

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

pub struct PoliceTiersPlugin;

impl Plugin for PoliceTiersPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PoliceTiersState>();

        app.init_resource::<crate::SaveableRegistry>();
        app.world_mut()
            .resource_mut::<crate::SaveableRegistry>()
            .register::<PoliceTiersState>();

        app.add_systems(
            FixedUpdate,
            update_police_tiers
                .after(crate::crime::update_crime)
                .in_set(crate::SimulationSet::Simulation),
        );
    }
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tier_coverage_radius_ordering() {
        assert!(PoliceTier::Kiosk.coverage_radius() < PoliceTier::Station.coverage_radius());
        assert!(
            PoliceTier::Station.coverage_radius() < PoliceTier::Headquarters.coverage_radius()
        );
    }

    #[test]
    fn test_tier_crime_reduction_ordering() {
        assert!(PoliceTier::Kiosk.crime_reduction() < PoliceTier::Station.crime_reduction());
        assert!(
            PoliceTier::Station.crime_reduction() < PoliceTier::Headquarters.crime_reduction()
        );
    }

    #[test]
    fn test_tier_response_time_ordering() {
        // Lower is faster — HQ should be fastest.
        assert!(PoliceTier::Headquarters.response_time() < PoliceTier::Station.response_time());
        assert!(PoliceTier::Station.response_time() < PoliceTier::Kiosk.response_time());
    }

    #[test]
    fn test_tier_maintenance_cost_ordering() {
        assert!(PoliceTier::Kiosk.maintenance_cost() < PoliceTier::Station.maintenance_cost());
        assert!(
            PoliceTier::Station.maintenance_cost() < PoliceTier::Headquarters.maintenance_cost()
        );
    }

    #[test]
    fn test_from_service_type() {
        assert_eq!(
            PoliceTier::from_service_type(ServiceType::PoliceKiosk),
            Some(PoliceTier::Kiosk)
        );
        assert_eq!(
            PoliceTier::from_service_type(ServiceType::PoliceStation),
            Some(PoliceTier::Station)
        );
        assert_eq!(
            PoliceTier::from_service_type(ServiceType::PoliceHQ),
            Some(PoliceTier::Headquarters)
        );
        assert_eq!(
            PoliceTier::from_service_type(ServiceType::Hospital),
            None
        );
    }

    #[test]
    fn test_default_state() {
        let s = PoliceTiersState::default();
        assert_eq!(s.total_buildings(), 0);
        assert!(!s.coordination_active);
        assert!((s.coordination_multiplier - 1.0).abs() < 0.001);
        assert!((s.city_coverage - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_saveable_roundtrip() {
        let mut s = PoliceTiersState::default();
        s.kiosk_stats.building_count = 5;
        s.station_stats.building_count = 3;
        s.hq_stats.building_count = 1;
        s.coordination_active = true;
        s.coordination_multiplier = HQ_COORDINATION_BONUS;
        s.city_coverage = 0.42;
        let bytes = s.save_to_bytes().unwrap();
        let r = PoliceTiersState::load_from_bytes(&bytes);
        assert_eq!(r.kiosk_stats.building_count, 5);
        assert_eq!(r.station_stats.building_count, 3);
        assert_eq!(r.hq_stats.building_count, 1);
        assert!(r.coordination_active);
        assert!((r.coordination_multiplier - HQ_COORDINATION_BONUS).abs() < 0.001);
        assert!((r.city_coverage - 0.42).abs() < 0.001);
    }

    #[test]
    fn test_total_maintenance() {
        let mut s = PoliceTiersState::default();
        s.kiosk_stats.total_maintenance = 16.0;
        s.station_stats.total_maintenance = 40.0;
        s.hq_stats.total_maintenance = 60.0;
        assert!((s.total_maintenance() - 116.0).abs() < 0.001);
    }
}
