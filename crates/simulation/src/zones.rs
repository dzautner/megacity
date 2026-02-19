use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::buildings::{Building, MixedUseBuilding};
use crate::grid::{CellType, WorldGrid, ZoneType};

// ---------------------------------------------------------------------------
// Natural vacancy rates: when vacancy is below these thresholds, demand rises;
// when above, demand falls. The midpoint of each range is the equilibrium.
// ---------------------------------------------------------------------------

/// Natural vacancy rate range for residential zones (5-7%, midpoint 6%).
const NATURAL_VACANCY_RES: (f32, f32) = (0.05, 0.07);
/// Natural vacancy rate range for commercial zones (5-8%, midpoint 6.5%).
const NATURAL_VACANCY_COM: (f32, f32) = (0.05, 0.08);
/// Natural vacancy rate range for industrial zones (5-8%, midpoint 6.5%).
const NATURAL_VACANCY_IND: (f32, f32) = (0.05, 0.08);
/// Natural vacancy rate range for office zones (8-12%, midpoint 10%).
const NATURAL_VACANCY_OFF: (f32, f32) = (0.08, 0.12);

/// Damping factor applied to demand changes each tick to smooth oscillation.
/// A value of 0.15 means demand moves at most 15% of the way toward the target.
const DAMPING: f32 = 0.15;

/// Bootstrap demand for the initial state when no buildings exist yet.
const BOOTSTRAP_DEMAND: f32 = 0.5;

#[derive(Resource, Debug, Clone, Serialize, Deserialize)]
pub struct ZoneDemand {
    pub residential: f32,
    pub commercial: f32,
    pub industrial: f32,
    pub office: f32,
    /// Tracked vacancy rates (built capacity vs occupied) per zone category.
    #[serde(default)]
    pub vacancy_residential: f32,
    #[serde(default)]
    pub vacancy_commercial: f32,
    #[serde(default)]
    pub vacancy_industrial: f32,
    #[serde(default)]
    pub vacancy_office: f32,
}

impl Default for ZoneDemand {
    fn default() -> Self {
        Self {
            residential: 0.0,
            commercial: 0.0,
            industrial: 0.0,
            office: 0.0,
            vacancy_residential: 0.0,
            vacancy_commercial: 0.0,
            vacancy_industrial: 0.0,
            vacancy_office: 0.0,
        }
    }
}

impl ZoneDemand {
    pub fn demand_for(&self, zone: ZoneType) -> f32 {
        match zone {
            ZoneType::ResidentialLow | ZoneType::ResidentialMedium | ZoneType::ResidentialHigh => {
                self.residential
            }
            ZoneType::CommercialLow | ZoneType::CommercialHigh => self.commercial,
            ZoneType::Industrial => self.industrial,
            ZoneType::Office => self.office,
            // MixedUse responds to the higher of residential and commercial demand
            ZoneType::MixedUse => self.residential.max(self.commercial),
            ZoneType::None => 0.0,
        }
    }
}

// ---------------------------------------------------------------------------
// Intermediate struct for tallying building stats from the grid + ECS.
// ---------------------------------------------------------------------------

struct ZoneStats {
    /// Total population living in residential buildings.
    population: u32,
    /// Total capacity of residential buildings.
    residential_capacity: u32,
    /// Total occupants of residential buildings.
    residential_occupants: u32,
    /// Total capacity of commercial buildings.
    commercial_capacity: u32,
    /// Total occupants of commercial buildings.
    commercial_occupants: u32,
    /// Total capacity of industrial buildings.
    industrial_capacity: u32,
    /// Total occupants of industrial buildings.
    industrial_occupants: u32,
    /// Total capacity of office buildings.
    office_capacity: u32,
    /// Total occupants of office buildings.
    office_occupants: u32,
    /// Total job capacity (commercial + industrial + office).
    total_job_capacity: u32,
    /// Total job occupants (commercial + industrial + office).
    total_job_occupants: u32,
    /// Whether any roads exist (needed for bootstrapping).
    has_roads: bool,
}

fn gather_zone_stats(
    grid: &WorldGrid,
    buildings: &Query<&Building>,
    mixed_use_buildings: &Query<&MixedUseBuilding>,
) -> ZoneStats {
    let mut stats = ZoneStats {
        population: 0,
        residential_capacity: 0,
        residential_occupants: 0,
        commercial_capacity: 0,
        commercial_occupants: 0,
        industrial_capacity: 0,
        industrial_occupants: 0,
        office_capacity: 0,
        office_occupants: 0,
        total_job_capacity: 0,
        total_job_occupants: 0,
        has_roads: false,
    };

    for cell in &grid.cells {
        if cell.cell_type == CellType::Road {
            stats.has_roads = true;
        }

        if let Some(entity) = cell.building_id {
            if let Ok(b) = buildings.get(entity) {
                match cell.zone {
                    ZoneType::ResidentialLow
                    | ZoneType::ResidentialMedium
                    | ZoneType::ResidentialHigh => {
                        stats.residential_capacity += b.capacity;
                        stats.residential_occupants += b.occupants;
                        stats.population += b.occupants;
                    }
                    ZoneType::CommercialLow | ZoneType::CommercialHigh => {
                        stats.commercial_capacity += b.capacity;
                        stats.commercial_occupants += b.occupants;
                        stats.total_job_capacity += b.capacity;
                        stats.total_job_occupants += b.occupants;
                    }
                    ZoneType::Industrial => {
                        stats.industrial_capacity += b.capacity;
                        stats.industrial_occupants += b.occupants;
                        stats.total_job_capacity += b.capacity;
                        stats.total_job_occupants += b.occupants;
                    }
                    ZoneType::Office => {
                        stats.office_capacity += b.capacity;
                        stats.office_occupants += b.occupants;
                        stats.total_job_capacity += b.capacity;
                        stats.total_job_occupants += b.occupants;
                    }
                    ZoneType::MixedUse => {
                        // MixedUse counts toward both residential and commercial
                        if let Ok(mu) = mixed_use_buildings.get(entity) {
                            stats.residential_capacity += mu.residential_capacity;
                            stats.residential_occupants += mu.residential_occupants;
                            stats.population += mu.residential_occupants;
                            stats.commercial_capacity += mu.commercial_capacity;
                            stats.commercial_occupants += mu.commercial_occupants;
                            stats.total_job_capacity += mu.commercial_capacity;
                            stats.total_job_occupants += mu.commercial_occupants;
                        }
                    }
                    ZoneType::None => {}
                }
            }
        }
    }

    stats
}

// ---------------------------------------------------------------------------
// Vacancy rate helpers
// ---------------------------------------------------------------------------

/// Compute vacancy rate: fraction of capacity that is unoccupied.
/// Returns 0.0 when capacity is 0 (no buildings).
fn vacancy_rate(capacity: u32, occupants: u32) -> f32 {
    if capacity == 0 {
        return 0.0;
    }
    let occupied = occupants.min(capacity) as f32;
    (capacity as f32 - occupied) / capacity as f32
}

/// Compute a raw demand signal from a vacancy rate relative to a natural vacancy range.
/// Returns a value roughly in [-1.0, 1.0]:
///   - Large positive when vacancy is well below the natural range (market is tight).
///   - Large negative when vacancy is well above the natural range (market is oversupplied).
///   - Near zero when vacancy is within the natural range.
fn vacancy_demand_signal(vacancy: f32, natural: (f32, f32)) -> f32 {
    let midpoint = (natural.0 + natural.1) * 0.5;
    // Scale factor: how strongly the signal responds to deviation.
    // At the edges of the natural range the signal should be ~0.3,
    // so we want signal_strength * half_width ≈ 0.3.
    let half_width = (natural.1 - natural.0) * 0.5;
    let sensitivity = if half_width > 0.0 {
        0.3 / half_width
    } else {
        10.0
    };
    // Negative because high vacancy means low demand (invert).
    let raw = (midpoint - vacancy) * sensitivity;
    raw.clamp(-1.0, 1.0)
}

// ---------------------------------------------------------------------------
// Market factor helpers
// ---------------------------------------------------------------------------

/// Employment availability: fraction of total job slots that are unfilled.
/// High values mean lots of available jobs, pulling residential demand up.
fn employment_availability(zs: &ZoneStats) -> f32 {
    if zs.total_job_capacity == 0 {
        return 0.0;
    }
    let unfilled = zs.total_job_capacity.saturating_sub(zs.total_job_occupants);
    (unfilled as f32 / zs.total_job_capacity as f32).clamp(0.0, 1.0)
}

/// Population spending power: ratio of population to commercial capacity.
/// High ratio means lots of shoppers relative to commercial space, driving demand.
fn population_spending_pressure(zs: &ZoneStats) -> f32 {
    if zs.commercial_capacity == 0 {
        // No commercial at all: high pressure to build some.
        return if zs.population > 0 { 1.0 } else { 0.0 };
    }
    let ratio = zs.population as f32 / zs.commercial_capacity as f32;
    // Normalize: ratio of 1.0 is balanced, >1 means under-served.
    (ratio - 0.5).clamp(0.0, 1.0)
}

/// Labor supply factor for industrial demand: how many workers exist relative
/// to industrial capacity. High ratio means ample labor, good for industry.
fn labor_supply_factor(zs: &ZoneStats) -> f32 {
    if zs.industrial_capacity == 0 {
        return if zs.population > 0 { 0.8 } else { 0.0 };
    }
    let ratio = zs.population as f32 / (zs.industrial_capacity as f32 * 2.0);
    ratio.clamp(0.0, 1.0)
}

/// Educated workforce fraction for office demand.
/// Approximated by looking at office occupancy rate as a proxy.
fn office_workforce_factor(zs: &ZoneStats) -> f32 {
    if zs.office_capacity == 0 {
        // No offices yet: moderate pull if population exists.
        return if zs.population > 100 { 0.5 } else { 0.0 };
    }
    // How well-staffed are offices? High occupancy = educated workforce available.
    let occ_rate = zs.office_occupants as f32 / zs.office_capacity as f32;
    // Also consider population scale: bigger city, more demand for offices.
    let pop_scale = (zs.population as f32 / 5000.0).min(1.0);
    (occ_rate * 0.5 + pop_scale * 0.5).clamp(0.0, 1.0)
}

// ---------------------------------------------------------------------------
// Core demand computation (pure function, testable without ECS)
// ---------------------------------------------------------------------------

/// Compute raw (un-damped) demand targets given zone stats.
/// Returns (residential, commercial, industrial, office) demands in [0, 1].
pub fn compute_market_demand(zs: &ZoneStats) -> (f32, f32, f32, f32) {
    // --- Bootstrap: no buildings at all ---
    if !zs.has_roads {
        return (0.0, 0.0, 0.0, 0.0);
    }

    let total_capacity = zs.residential_capacity
        + zs.commercial_capacity
        + zs.industrial_capacity
        + zs.office_capacity;

    if total_capacity == 0 {
        // Roads exist but no buildings: initial bootstrap demand.
        return (
            BOOTSTRAP_DEMAND,
            BOOTSTRAP_DEMAND * 0.4,
            BOOTSTRAP_DEMAND * 0.6,
            BOOTSTRAP_DEMAND * 0.2,
        );
    }

    // --- Vacancy rates ---
    let vr = vacancy_rate(zs.residential_capacity, zs.residential_occupants);
    let vc = vacancy_rate(zs.commercial_capacity, zs.commercial_occupants);
    let vi = vacancy_rate(zs.industrial_capacity, zs.industrial_occupants);
    let vo = vacancy_rate(zs.office_capacity, zs.office_occupants);

    // --- Vacancy signals: positive = need more, negative = oversupplied ---
    let r_vacancy_sig = vacancy_demand_signal(vr, NATURAL_VACANCY_RES);
    let c_vacancy_sig = vacancy_demand_signal(vc, NATURAL_VACANCY_COM);
    let i_vacancy_sig = vacancy_demand_signal(vi, NATURAL_VACANCY_IND);
    let o_vacancy_sig = vacancy_demand_signal(vo, NATURAL_VACANCY_OFF);

    // --- Market factor signals ---
    let emp_avail = employment_availability(zs);
    let spending = population_spending_pressure(zs);
    let labor = labor_supply_factor(zs);
    let office_wf = office_workforce_factor(zs);

    // --- Residential demand = f(employment availability, vacancy) ---
    // Weight: 50% vacancy signal, 35% employment availability, 15% base immigration pressure.
    let r_base_pressure = 0.15; // constant immigration pressure
    let r_raw = r_vacancy_sig * 0.50 + emp_avail * 0.35 + r_base_pressure;

    // --- Commercial demand = f(population spending power, vacancy) ---
    // Weight: 45% vacancy signal, 40% population spending, 15% base.
    let c_raw = c_vacancy_sig * 0.45 + spending * 0.40 + 0.05;

    // --- Industrial demand = f(labor supply, vacancy) ---
    // Weight: 50% vacancy signal, 35% labor supply, 15% base.
    let i_raw = i_vacancy_sig * 0.50 + labor * 0.35 + 0.05;

    // --- Office demand = f(educated workforce, vacancy) ---
    // Weight: 45% vacancy signal, 40% workforce factor, 15% base.
    let o_raw = o_vacancy_sig * 0.45 + office_wf * 0.40 + 0.02;

    (
        r_raw.clamp(0.0, 1.0),
        c_raw.clamp(0.0, 1.0),
        i_raw.clamp(0.0, 1.0),
        o_raw.clamp(0.0, 1.0),
    )
}

// ---------------------------------------------------------------------------
// ECS system
// ---------------------------------------------------------------------------

pub fn update_zone_demand(
    slow_tick: Res<crate::SlowTickTimer>,
    grid: Res<WorldGrid>,
    buildings: Query<&Building>,
    mixed_use_buildings: Query<&MixedUseBuilding>,
    mut demand: ResMut<ZoneDemand>,
) {
    if !slow_tick.should_run() {
        return;
    }

    let zs = gather_zone_stats(&grid, &buildings, &mixed_use_buildings);

    // Update tracked vacancy rates.
    demand.vacancy_residential =
        vacancy_rate(zs.residential_capacity, zs.residential_occupants);
    demand.vacancy_commercial =
        vacancy_rate(zs.commercial_capacity, zs.commercial_occupants);
    demand.vacancy_industrial =
        vacancy_rate(zs.industrial_capacity, zs.industrial_occupants);
    demand.vacancy_office = vacancy_rate(zs.office_capacity, zs.office_occupants);

    // Compute raw target demand values.
    let (r_target, c_target, i_target, o_target) = compute_market_demand(&zs);

    // Apply damping: smoothly interpolate toward target to avoid oscillation.
    demand.residential += (r_target - demand.residential) * DAMPING;
    demand.commercial += (c_target - demand.commercial) * DAMPING;
    demand.industrial += (i_target - demand.industrial) * DAMPING;
    demand.office += (o_target - demand.office) * DAMPING;

    // Ensure final values stay clamped.
    demand.residential = demand.residential.clamp(0.0, 1.0);
    demand.commercial = demand.commercial.clamp(0.0, 1.0);
    demand.industrial = demand.industrial.clamp(0.0, 1.0);
    demand.office = demand.office.clamp(0.0, 1.0);
}

pub fn is_adjacent_to_road(grid: &WorldGrid, x: usize, y: usize) -> bool {
    // Check within 2-cell radius so interior block cells can also have buildings
    for dy in -2i32..=2 {
        for dx in -2i32..=2 {
            if dx == 0 && dy == 0 {
                continue;
            }
            let nx = x as i32 + dx;
            let ny = y as i32 + dy;
            if nx >= 0
                && ny >= 0
                && (nx as usize) < grid.width
                && (ny as usize) < grid.height
                && grid.get(nx as usize, ny as usize).cell_type == CellType::Road
            {
                return true;
            }
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{GRID_HEIGHT, GRID_WIDTH};

    // Helper to create a ZoneStats for testing the pure demand function.
    fn make_stats(
        has_roads: bool,
        r_cap: u32,
        r_occ: u32,
        c_cap: u32,
        c_occ: u32,
        i_cap: u32,
        i_occ: u32,
        o_cap: u32,
        o_occ: u32,
    ) -> ZoneStats {
        ZoneStats {
            population: r_occ,
            residential_capacity: r_cap,
            residential_occupants: r_occ,
            commercial_capacity: c_cap,
            commercial_occupants: c_occ,
            industrial_capacity: i_cap,
            industrial_occupants: i_occ,
            office_capacity: o_cap,
            office_occupants: o_occ,
            total_job_capacity: c_cap + i_cap + o_cap,
            total_job_occupants: c_occ + i_occ + o_occ,
            has_roads,
        }
    }

    #[test]
    fn test_zoning_requires_road_adjacency() {
        let grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        // No roads placed, no cell is adjacent to a road
        assert!(!is_adjacent_to_road(&grid, 10, 10));
    }

    #[test]
    fn test_demand_increases_with_roads() {
        // No roads: demand should be zero.
        let zs_no_roads = make_stats(false, 0, 0, 0, 0, 0, 0, 0, 0);
        let (r0, _, _, _) = compute_market_demand(&zs_no_roads);
        assert_eq!(r0, 0.0);

        // Roads but no buildings: bootstrap demand should be positive.
        let zs_roads = make_stats(true, 0, 0, 0, 0, 0, 0, 0, 0);
        let (r1, _, _, _) = compute_market_demand(&zs_roads);
        assert!(r1 > 0.0, "Residential demand should be positive with roads");
    }

    #[test]
    fn test_demand_formula_bounds() {
        let demand = ZoneDemand {
            residential: 0.8,
            commercial: 0.5,
            industrial: 0.3,
            office: 0.2,
            ..Default::default()
        };
        assert!(demand.residential >= 0.0 && demand.residential <= 1.0);
        assert!(demand.commercial >= 0.0 && demand.commercial <= 1.0);
        assert!(demand.industrial >= 0.0 && demand.industrial <= 1.0);
        assert!(demand.office >= 0.0 && demand.office <= 1.0);
    }

    #[test]
    fn test_demand_for_zones() {
        let demand = ZoneDemand {
            residential: 0.8,
            commercial: 0.5,
            industrial: 0.3,
            office: 0.2,
            ..Default::default()
        };
        assert_eq!(demand.demand_for(ZoneType::ResidentialLow), 0.8);
        assert_eq!(demand.demand_for(ZoneType::ResidentialMedium), 0.8);
        assert_eq!(demand.demand_for(ZoneType::ResidentialHigh), 0.8);
        assert_eq!(demand.demand_for(ZoneType::CommercialLow), 0.5);
        assert_eq!(demand.demand_for(ZoneType::CommercialHigh), 0.5);
        assert_eq!(demand.demand_for(ZoneType::Industrial), 0.3);
        assert_eq!(demand.demand_for(ZoneType::Office), 0.2);
        assert_eq!(demand.demand_for(ZoneType::None), 0.0);
    }

    #[test]
    fn test_mixed_use_demand_uses_max() {
        // MixedUse should respond to the higher of residential and commercial demand
        let demand = ZoneDemand {
            residential: 0.8,
            commercial: 0.5,
            industrial: 0.3,
            office: 0.2,
            ..Default::default()
        };
        assert_eq!(demand.demand_for(ZoneType::MixedUse), 0.8);

        let demand2 = ZoneDemand {
            residential: 0.3,
            commercial: 0.9,
            industrial: 0.3,
            office: 0.2,
            ..Default::default()
        };
        assert_eq!(demand2.demand_for(ZoneType::MixedUse), 0.9);
    }

    // -----------------------------------------------------------------------
    // Vacancy rate tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_vacancy_rate_zero_capacity() {
        assert_eq!(vacancy_rate(0, 0), 0.0);
    }

    #[test]
    fn test_vacancy_rate_full_occupancy() {
        assert!((vacancy_rate(100, 100)).abs() < 0.001);
    }

    #[test]
    fn test_vacancy_rate_half_empty() {
        let vr = vacancy_rate(200, 100);
        assert!((vr - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_vacancy_rate_empty_building() {
        let vr = vacancy_rate(100, 0);
        assert!((vr - 1.0).abs() < 0.001);
    }

    // -----------------------------------------------------------------------
    // Vacancy demand signal tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_vacancy_signal_at_midpoint_is_near_zero() {
        // At midpoint of natural vacancy range, signal should be ~0.
        let mid = (NATURAL_VACANCY_RES.0 + NATURAL_VACANCY_RES.1) * 0.5;
        let sig = vacancy_demand_signal(mid, NATURAL_VACANCY_RES);
        assert!(
            sig.abs() < 0.05,
            "Signal at midpoint should be near zero, got {}",
            sig
        );
    }

    #[test]
    fn test_vacancy_signal_zero_vacancy_is_positive() {
        // 0% vacancy = extremely tight market = high positive demand signal.
        let sig = vacancy_demand_signal(0.0, NATURAL_VACANCY_RES);
        assert!(sig > 0.0, "Zero vacancy should give positive signal");
    }

    #[test]
    fn test_vacancy_signal_high_vacancy_is_negative() {
        // 50% vacancy = hugely oversupplied = negative demand signal.
        let sig = vacancy_demand_signal(0.50, NATURAL_VACANCY_RES);
        assert!(sig < 0.0, "High vacancy should give negative signal");
    }

    // -----------------------------------------------------------------------
    // Market demand integration tests (pure function)
    // -----------------------------------------------------------------------

    #[test]
    fn test_zero_vacancy_demand_high() {
        // 0% vacancy across all zones: everything is fully occupied.
        // Jobs exist (meaning employment is available IF vacancy is 0 the jobs
        // are full, so employment_availability is 0 -- but vacancy signal is strong).
        let zs = make_stats(
            true,
            1000, 1000, // residential: 100% occupied
            500, 500, // commercial: 100% occupied
            300, 300, // industrial: 100% occupied
            200, 200, // office: 100% occupied
        );
        let (r, c, i, o) = compute_market_demand(&zs);
        // Residential should be high because vacancy signal is strongly positive.
        assert!(
            r > 0.3,
            "0% vacancy should produce high residential demand, got {}",
            r
        );
        // Commercial should also be elevated.
        assert!(
            c > 0.2,
            "0% vacancy should produce elevated commercial demand, got {}",
            c
        );
        // Industrial should be elevated.
        assert!(
            i > 0.2,
            "0% vacancy should produce elevated industrial demand, got {}",
            i
        );
        // Office should be elevated.
        assert!(
            o > 0.2,
            "0% vacancy should produce elevated office demand, got {}",
            o
        );
    }

    #[test]
    fn test_high_vacancy_demand_low() {
        // 80% vacancy (only 20% occupied): massive oversupply.
        let zs = make_stats(
            true,
            1000, 200, // residential: 80% vacant
            500, 100, // commercial: 80% vacant
            300, 60, // industrial: 80% vacant
            200, 40, // office: 80% vacant
        );
        let (r, c, i, o) = compute_market_demand(&zs);
        // All demands should be very low with 80% vacancy.
        assert!(
            r < 0.2,
            "80% vacancy should produce low residential demand, got {}",
            r
        );
        assert!(
            c < 0.2,
            "80% vacancy should produce low commercial demand, got {}",
            c
        );
        assert!(
            i < 0.2,
            "80% vacancy should produce low industrial demand, got {}",
            i
        );
        assert!(
            o < 0.2,
            "80% vacancy should produce low office demand, got {}",
            o
        );
    }

    #[test]
    fn test_bootstrap_demand_no_buildings() {
        // Roads exist, no buildings: bootstrap demand should be moderate.
        let zs = make_stats(true, 0, 0, 0, 0, 0, 0, 0, 0);
        let (r, c, i, o) = compute_market_demand(&zs);
        assert!(
            r > 0.3,
            "Bootstrap residential demand should be moderate, got {}",
            r
        );
        assert!(c > 0.0, "Bootstrap commercial demand should be positive");
        assert!(i > 0.0, "Bootstrap industrial demand should be positive");
        assert!(o > 0.0, "Bootstrap office demand should be positive");
    }

    #[test]
    fn test_no_roads_no_demand() {
        let zs = make_stats(false, 0, 0, 0, 0, 0, 0, 0, 0);
        let (r, c, i, o) = compute_market_demand(&zs);
        assert_eq!(r, 0.0);
        assert_eq!(c, 0.0);
        assert_eq!(i, 0.0);
        assert_eq!(o, 0.0);
    }

    #[test]
    fn test_adding_jobs_raises_residential_demand() {
        // Scenario A: few jobs, moderate residential occupancy.
        let zs_few_jobs = make_stats(
            true,
            500, 400, // residential: 80% occupied
            50, 50, // commercial: full
            50, 50, // industrial: full
            50, 50, // office: full
        );
        let (r_few, _, _, _) = compute_market_demand(&zs_few_jobs);

        // Scenario B: many unfilled jobs, same residential occupancy.
        let zs_many_jobs = make_stats(
            true,
            500, 400, // residential: 80% occupied
            500, 50, // commercial: mostly empty (= lots of job openings)
            500, 50, // industrial: mostly empty
            500, 50, // office: mostly empty
        );
        let (r_many, _, _, _) = compute_market_demand(&zs_many_jobs);

        // More available jobs should increase residential demand (people want to move in).
        assert!(
            r_many > r_few,
            "More job availability should raise residential demand: {} vs {}",
            r_many,
            r_few
        );
    }

    #[test]
    fn test_excess_residential_lowers_demand() {
        // Lots of residential capacity, few occupants (high vacancy).
        let zs_excess = make_stats(
            true,
            2000, 200, // residential: 90% vacant
            200, 180, // commercial: near full
            200, 180, // industrial: near full
            100, 90, // office: near full
        );
        let (r, _, _, _) = compute_market_demand(&zs_excess);
        assert!(
            r < 0.2,
            "Excess residential should lower demand below 0.2, got {}",
            r
        );
    }

    #[test]
    fn test_demand_values_always_in_bounds() {
        // Test with a variety of extreme parameters.
        let cases = [
            make_stats(true, 0, 0, 0, 0, 0, 0, 0, 0),
            make_stats(true, 100, 100, 100, 100, 100, 100, 100, 100),
            make_stats(true, 100, 0, 100, 0, 100, 0, 100, 0),
            make_stats(true, 1, 1, 1, 1, 1, 1, 1, 1),
            make_stats(true, 100000, 1, 100000, 1, 100000, 1, 100000, 1),
            make_stats(true, 1, 100000, 1, 100000, 1, 100000, 1, 100000),
            make_stats(false, 100, 50, 100, 50, 100, 50, 100, 50),
        ];
        for zs in &cases {
            let (r, c, i, o) = compute_market_demand(zs);
            assert!(r >= 0.0 && r <= 1.0, "Residential out of bounds: {}", r);
            assert!(c >= 0.0 && c <= 1.0, "Commercial out of bounds: {}", c);
            assert!(i >= 0.0 && i <= 1.0, "Industrial out of bounds: {}", i);
            assert!(o >= 0.0 && o <= 1.0, "Office out of bounds: {}", o);
        }
    }

    #[test]
    fn test_damping_smooths_demand_changes() {
        // Simulate starting from zero demand and computing target.
        let mut demand = ZoneDemand::default();
        let zs = make_stats(true, 0, 0, 0, 0, 0, 0, 0, 0);
        let (r_target, c_target, i_target, o_target) = compute_market_demand(&zs);

        // Apply damping once.
        demand.residential += (r_target - demand.residential) * DAMPING;
        demand.commercial += (c_target - demand.commercial) * DAMPING;
        demand.industrial += (i_target - demand.industrial) * DAMPING;
        demand.office += (o_target - demand.office) * DAMPING;

        // After one step, demand should be between 0 and target (not at target yet).
        assert!(
            demand.residential < r_target,
            "Damped residential {} should be below target {}",
            demand.residential,
            r_target
        );
        assert!(
            demand.residential > 0.0,
            "Damped residential should be above 0.0"
        );
    }
}
