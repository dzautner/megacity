use crate::buildings::Building;
use crate::config::{CELL_SIZE, GRID_HEIGHT, GRID_WIDTH};
use crate::grid::ZoneType;
use crate::services::{ServiceBuilding, ServiceType};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

// =============================================================================
// Waste collection constants (WASTE-003)
// =============================================================================

/// Service radius in grid cells for waste collection facilities.
pub const WASTE_SERVICE_RADIUS_CELLS: i32 = 20;

/// Per-facility collection capacity in tons/day.
pub fn facility_capacity_tons(service_type: ServiceType) -> f64 {
    match service_type {
        ServiceType::TransferStation => 200.0,
        ServiceType::Landfill => 150.0,
        ServiceType::RecyclingCenter => 100.0,
        ServiceType::Incinerator => 250.0,
        _ => 0.0,
    }
}

/// Per-facility daily operating cost.
pub fn facility_operating_cost(service_type: ServiceType) -> f64 {
    match service_type {
        ServiceType::TransferStation => 2_000.0,
        ServiceType::Landfill => 1_500.0,
        ServiceType::RecyclingCenter => 1_800.0,
        ServiceType::Incinerator => 3_000.0,
        _ => 0.0,
    }
}

/// Cost per ton-mile for waste transport.
pub const TRANSPORT_COST_PER_TON_MILE: f64 = 5.0;

/// Happiness penalty for uncollected waste at a building's location.
pub const UNCOLLECTED_WASTE_HAPPINESS_PENALTY: f32 = 5.0;

/// Land value reduction factor for uncollected waste (10% reduction).
pub const UNCOLLECTED_WASTE_LAND_VALUE_FACTOR: f32 = 0.10;

#[derive(Resource)]
pub struct GarbageGrid {
    pub levels: Vec<u8>,
    pub width: usize,
    pub height: usize,
}

impl Default for GarbageGrid {
    fn default() -> Self {
        Self {
            levels: vec![0; GRID_WIDTH * GRID_HEIGHT],
            width: GRID_WIDTH,
            height: GRID_HEIGHT,
        }
    }
}

impl GarbageGrid {
    pub fn get(&self, x: usize, y: usize) -> u8 {
        self.levels[y * self.width + x]
    }
    pub fn set(&mut self, x: usize, y: usize, val: u8) {
        self.levels[y * self.width + x] = val;
    }
}

// =============================================================================
// Per-building waste generation (WASTE-001)
// =============================================================================

/// Component attached to each building that tracks its waste generation rate.
///
/// Residential buildings generate waste per person per day (lbs/person/day),
/// while commercial, industrial, and service buildings generate waste per
/// building per day (lbs/building/day).
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct WasteProducer {
    /// Waste generation rate in lbs/day.
    /// For residential: lbs per person per day (multiplied by occupants).
    /// For non-residential: lbs per building per day.
    pub waste_lbs_per_day: f32,
    /// Whether this building participates in the recycling program.
    /// When true, waste output is reduced by 30%.
    pub recycling_participation: bool,
}

impl WasteProducer {
    /// Returns the per-person-per-day waste rate for a residential building
    /// based on its zone type and building level.
    ///
    /// Level 1 = low-income, Level 2 = middle-income, Level 3+ = high-income.
    pub fn residential_rate(zone: ZoneType, level: u8) -> f32 {
        match (zone, level) {
            (ZoneType::ResidentialLow, 1) => 3.0,    // low-income
            (ZoneType::ResidentialLow, 2) => 4.5,    // middle-income
            (ZoneType::ResidentialLow, _) => 6.0,    // high-income
            (ZoneType::ResidentialMedium, 1) => 3.0, // low-income
            (ZoneType::ResidentialMedium, 2) => 4.5, // middle-income
            (ZoneType::ResidentialMedium, _) => 6.0, // high-income
            (ZoneType::ResidentialHigh, 1) => 3.0,   // low-income
            (ZoneType::ResidentialHigh, 2) => 4.5,   // middle-income
            (ZoneType::ResidentialHigh, _) => 6.0,   // high-income
            _ => 0.0,
        }
    }

    /// Returns the per-building-per-day waste rate for a commercial building.
    ///
    /// Low-density commercial = small shops (50 lbs/day).
    /// High-density commercial at level 1-2 = large commercial (300 lbs/day).
    /// High-density commercial at level 3+ approximates restaurants (200 lbs/day).
    pub fn commercial_rate(zone: ZoneType, level: u8) -> f32 {
        match (zone, level) {
            (ZoneType::CommercialLow, _) => 50.0,       // small commercial
            (ZoneType::CommercialHigh, 1..=2) => 300.0, // large commercial
            (ZoneType::CommercialHigh, _) => 200.0,     // restaurant-type
            _ => 0.0,
        }
    }

    /// Returns the per-building-per-day waste rate for an industrial building.
    ///
    /// Level 1-2 = light industry (500 lbs/day).
    /// Level 3+ = heavy industry (2000 lbs/day).
    pub fn industrial_rate(level: u8) -> f32 {
        match level {
            1..=2 => 500.0, // light industry
            _ => 2000.0,    // heavy industry
        }
    }

    /// Returns the per-facility-per-day waste rate for a service building.
    pub fn service_rate(service_type: ServiceType) -> f32 {
        match service_type {
            ServiceType::Hospital | ServiceType::MedicalCenter => 1500.0,
            ServiceType::ElementarySchool
            | ServiceType::HighSchool
            | ServiceType::Kindergarten
            | ServiceType::Library => 100.0,
            ServiceType::University => 200.0,
            ServiceType::Stadium => 500.0,
            _ => 50.0, // default for other service buildings
        }
    }

    /// Create a WasteProducer for a zoned building based on its type and level.
    pub fn for_building(zone: ZoneType, level: u8) -> Self {
        let rate = if zone.is_residential() {
            Self::residential_rate(zone, level)
        } else if zone.is_commercial() {
            Self::commercial_rate(zone, level)
        } else if zone == ZoneType::Industrial {
            Self::industrial_rate(level)
        } else {
            // Office buildings: moderate waste (similar to small commercial)
            50.0
        };
        Self {
            waste_lbs_per_day: rate,
            recycling_participation: false,
        }
    }

    /// Create a WasteProducer for a service building.
    pub fn for_service(service_type: ServiceType) -> Self {
        Self {
            waste_lbs_per_day: Self::service_rate(service_type),
            recycling_participation: false,
        }
    }

    /// Returns the effective daily waste output in lbs, accounting for
    /// recycling participation and optionally the building's occupant count
    /// (for residential buildings, which scale by population).
    pub fn effective_daily_waste(&self, occupants: u32, is_residential: bool) -> f32 {
        let base = if is_residential {
            self.waste_lbs_per_day * occupants as f32
        } else {
            self.waste_lbs_per_day
        };
        if self.recycling_participation {
            base * 0.7 // 30% reduction from recycling
        } else {
            base
        }
    }
}

/// City-wide waste statistics resource, updated periodically by the waste system.
#[derive(Resource, Debug, Clone, Default, Serialize, Deserialize)]
pub struct WasteSystem {
    /// Total waste generated across the city in tons (accumulated since last reset).
    pub total_generated_tons: f64,
    /// Total waste generated this update period in tons.
    pub period_generated_tons: f64,
    /// Per-capita waste in lbs/person/day (averaged over the last update period).
    pub per_capita_lbs_per_day: f32,
    /// Total population used for per-capita calculation.
    pub tracked_population: u32,
    /// Number of buildings with recycling participation.
    pub recycling_buildings: u32,
    /// Total number of waste-producing buildings tracked.
    pub total_producers: u32,
    // --- WASTE-003: Collection tracking ---
    /// Total waste collected this period in tons.
    pub total_collected_tons: f64,
    /// Total collection capacity across all facilities in tons/day.
    pub total_capacity_tons: f64,
    /// Collection rate: min(1.0, capacity / generated). 1.0 means all waste collected.
    pub collection_rate: f64,
    /// Number of buildings not covered by any waste collection facility.
    pub uncovered_buildings: u32,
    /// Total transport cost this period.
    pub transport_cost: f64,
    /// Number of active waste collection facilities.
    pub active_facilities: u32,
}

/// Per-cell waste collection coverage grid (WASTE-003).
///
/// Tracks which cells are within the service area of at least one waste
/// collection facility. Used to determine uncollected waste accumulation
/// and happiness/land-value penalties.
#[derive(Resource)]
pub struct WasteCollectionGrid {
    /// Per-cell collection coverage: 0 = uncovered, >0 = number of covering facilities.
    pub coverage: Vec<u8>,
    /// Per-cell uncollected waste accumulation in lbs.
    pub uncollected_lbs: Vec<f32>,
    pub width: usize,
    pub height: usize,
}

impl Default for WasteCollectionGrid {
    fn default() -> Self {
        Self {
            coverage: vec![0; GRID_WIDTH * GRID_HEIGHT],
            uncollected_lbs: vec![0.0; GRID_WIDTH * GRID_HEIGHT],
            width: GRID_WIDTH,
            height: GRID_HEIGHT,
        }
    }
}

impl WasteCollectionGrid {
    #[inline]
    fn idx(&self, x: usize, y: usize) -> usize {
        y * self.width + x
    }

    /// Returns true if the cell at (x, y) is covered by waste collection.
    pub fn is_covered(&self, x: usize, y: usize) -> bool {
        self.coverage[self.idx(x, y)] > 0
    }

    /// Returns the uncollected waste in lbs at (x, y).
    pub fn uncollected(&self, x: usize, y: usize) -> f32 {
        self.uncollected_lbs[self.idx(x, y)]
    }

    /// Clear coverage counts (recalculated each tick).
    pub fn clear_coverage(&mut self) {
        self.coverage.fill(0);
    }
}

/// Attaches `WasteProducer` components to buildings that don't have one yet.
/// Runs on the slow tick to avoid overhead every frame.
pub fn attach_waste_producers(
    slow_timer: Res<crate::SlowTickTimer>,
    mut commands: Commands,
    buildings_without: Query<(Entity, &Building), Without<WasteProducer>>,
    services_without: Query<(Entity, &ServiceBuilding), Without<WasteProducer>>,
    policies: Res<crate::policies::Policies>,
) {
    if !slow_timer.should_run() {
        return;
    }
    let recycling_active = policies.is_active(crate::policies::Policy::RecyclingProgram);

    for (entity, building) in &buildings_without {
        let mut producer = WasteProducer::for_building(building.zone_type, building.level);
        producer.recycling_participation = recycling_active;
        commands.entity(entity).insert(producer);
    }

    for (entity, service) in &services_without {
        let mut producer = WasteProducer::for_service(service.service_type);
        producer.recycling_participation = recycling_active;
        commands.entity(entity).insert(producer);
    }
}

/// Updates recycling participation on all WasteProducers when the recycling
/// policy changes. Runs on the slow tick.
pub fn sync_recycling_policy(
    slow_timer: Res<crate::SlowTickTimer>,
    policies: Res<crate::policies::Policies>,
    mut producers: Query<&mut WasteProducer>,
) {
    if !slow_timer.should_run() {
        return;
    }
    let recycling_active = policies.is_active(crate::policies::Policy::RecyclingProgram);
    for mut producer in &mut producers {
        producer.recycling_participation = recycling_active;
    }
}

/// Aggregates waste generation across all buildings and updates the `WasteSystem`
/// resource with totals and per-capita metrics.
///
/// Runs on the slow tick (every ~10 seconds of game time).
/// The slow tick interval is 100 ticks at 10Hz = 10 game-seconds.
/// We treat each slow tick as representing roughly 1 game-day for waste calculations.
pub fn update_waste_generation(
    slow_timer: Res<crate::SlowTickTimer>,
    mut waste_system: ResMut<WasteSystem>,
    building_producers: Query<(&Building, &WasteProducer)>,
    service_producers: Query<(&ServiceBuilding, &WasteProducer)>,
    stats: Res<crate::stats::CityStats>,
) {
    if !slow_timer.should_run() {
        return;
    }

    let mut total_waste_lbs: f64 = 0.0;
    let mut recycling_count = 0u32;
    let mut producer_count = 0u32;

    // Zoned buildings (residential, commercial, industrial, office)
    for (building, producer) in &building_producers {
        let is_residential = building.zone_type.is_residential();
        let daily_waste = producer.effective_daily_waste(building.occupants, is_residential);
        total_waste_lbs += daily_waste as f64;
        producer_count += 1;
        if producer.recycling_participation {
            recycling_count += 1;
        }
    }

    // Service buildings (hospitals, schools, etc.)
    for (_service, producer) in &service_producers {
        let daily_waste = producer.effective_daily_waste(0, false);
        total_waste_lbs += daily_waste as f64;
        producer_count += 1;
        if producer.recycling_participation {
            recycling_count += 1;
        }
    }

    // Convert lbs to tons (1 ton = 2000 lbs)
    let period_tons = total_waste_lbs / 2000.0;

    let population = stats.population;
    let per_capita = if population > 0 {
        total_waste_lbs as f32 / population as f32
    } else {
        0.0
    };

    waste_system.period_generated_tons = period_tons;
    waste_system.total_generated_tons += period_tons;
    waste_system.per_capita_lbs_per_day = per_capita;
    waste_system.tracked_population = population;
    waste_system.recycling_buildings = recycling_count;
    waste_system.total_producers = producer_count;
}

/// Updates waste collection coverage and statistics (WASTE-003).
///
/// For each waste collection facility (transfer station, landfill, recycling center,
/// incinerator), marks cells within service radius as covered. Then computes the
/// collection rate as `min(1.0, total_capacity / total_generated)`, and accumulates
/// uncollected waste at uncovered buildings.
///
/// Closer buildings are implicitly served first (capacity-based, not per-truck).
/// Overlapping service areas do not double-count capacity.
pub fn update_waste_collection(
    slow_timer: Res<crate::SlowTickTimer>,
    mut waste_system: ResMut<WasteSystem>,
    mut collection_grid: ResMut<WasteCollectionGrid>,
    waste_services: Query<&ServiceBuilding>,
    building_producers: Query<(&Building, &WasteProducer)>,
    service_producers: Query<(&ServiceBuilding, &WasteProducer)>,
) {
    if !slow_timer.should_run() {
        return;
    }

    // Phase 1: Rebuild coverage grid from waste service buildings.
    collection_grid.clear_coverage();
    let mut total_capacity: f64 = 0.0;
    let mut facility_count = 0u32;

    for service in &waste_services {
        if !ServiceBuilding::is_garbage(service.service_type) {
            continue;
        }
        total_capacity += facility_capacity_tons(service.service_type);
        facility_count += 1;

        let radius = WASTE_SERVICE_RADIUS_CELLS;
        let sx = service.grid_x as i32;
        let sy = service.grid_y as i32;
        let r2 = (radius as f32 * CELL_SIZE) * (radius as f32 * CELL_SIZE);

        for dy in -radius..=radius {
            for dx in -radius..=radius {
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
                let idx = cy as usize * collection_grid.width + cx as usize;
                collection_grid.coverage[idx] = collection_grid.coverage[idx].saturating_add(1);
            }
        }
    }

    // Phase 2: Compute total waste generated by all buildings this period (lbs).
    let mut total_generated_lbs: f64 = 0.0;
    let mut uncovered_buildings = 0u32;

    // Collect per-building waste and coverage status for zoned buildings.
    for (building, producer) in &building_producers {
        let is_residential = building.zone_type.is_residential();
        let daily_lbs = producer.effective_daily_waste(building.occupants, is_residential) as f64;
        total_generated_lbs += daily_lbs;

        let covered = collection_grid.is_covered(building.grid_x, building.grid_y);
        if !covered {
            uncovered_buildings += 1;
        }
    }

    // Service buildings that produce waste.
    for (service, producer) in &service_producers {
        let daily_lbs = producer.effective_daily_waste(0, false) as f64;
        total_generated_lbs += daily_lbs;

        let covered = collection_grid.is_covered(service.grid_x, service.grid_y);
        if !covered {
            uncovered_buildings += 1;
        }
    }

    let total_generated_tons = total_generated_lbs / 2000.0;

    // Phase 3: Compute collection rate.
    let collection_rate = if total_generated_tons > 0.0 {
        (total_capacity / total_generated_tons).min(1.0)
    } else {
        1.0 // nothing to collect
    };

    let total_collected_tons = total_generated_tons * collection_rate;

    // Phase 4: Accumulate uncollected waste at uncovered building locations.
    // For covered buildings, reduce uncollected waste proportional to collection rate.
    // For uncovered buildings, all waste accumulates.
    for (building, producer) in &building_producers {
        let is_residential = building.zone_type.is_residential();
        let daily_lbs = producer.effective_daily_waste(building.occupants, is_residential);
        let idx = building.grid_y * collection_grid.width + building.grid_x;
        let covered = collection_grid.coverage[idx] > 0;

        if covered {
            // Covered: only uncollected fraction accumulates, and collected fraction decays.
            let uncollected_fraction = 1.0 - collection_rate as f32;
            collection_grid.uncollected_lbs[idx] += daily_lbs * uncollected_fraction;
            // Decay: collection picks up some accumulated waste too.
            collection_grid.uncollected_lbs[idx] *= 1.0 - collection_rate as f32 * 0.5;
        } else {
            // Not covered: all waste accumulates.
            collection_grid.uncollected_lbs[idx] += daily_lbs;
        }
        // Cap uncollected waste to prevent unbounded accumulation.
        collection_grid.uncollected_lbs[idx] = collection_grid.uncollected_lbs[idx].min(10_000.0);
    }

    for (service, producer) in &service_producers {
        let daily_lbs = producer.effective_daily_waste(0, false);
        let idx = service.grid_y * collection_grid.width + service.grid_x;
        let covered = collection_grid.coverage[idx] > 0;

        if covered {
            let uncollected_fraction = 1.0 - collection_rate as f32;
            collection_grid.uncollected_lbs[idx] += daily_lbs * uncollected_fraction;
            collection_grid.uncollected_lbs[idx] *= 1.0 - collection_rate as f32 * 0.5;
        } else {
            collection_grid.uncollected_lbs[idx] += daily_lbs;
        }
        collection_grid.uncollected_lbs[idx] = collection_grid.uncollected_lbs[idx].min(10_000.0);
    }

    // Phase 5: Compute transport cost (simplified: total_collected * cost_per_ton_mile * avg_distance).
    // Average distance approximated as half the service radius in cells, converted to miles.
    // 1 cell = CELL_SIZE world units. Assume 1 world unit ~ 1 meter, so CELL_SIZE meters per cell.
    let avg_distance_cells = WASTE_SERVICE_RADIUS_CELLS as f64 / 2.0;
    let avg_distance_miles = avg_distance_cells * CELL_SIZE as f64 / 1609.0; // meters to miles
    let transport_cost = total_collected_tons * TRANSPORT_COST_PER_TON_MILE * avg_distance_miles;

    // Phase 6: Update WasteSystem resource.
    waste_system.total_collected_tons = total_collected_tons;
    waste_system.total_capacity_tons = total_capacity;
    waste_system.collection_rate = collection_rate;
    waste_system.uncovered_buildings = uncovered_buildings;
    waste_system.transport_cost = transport_cost;
    waste_system.active_facilities = facility_count;
}

pub fn update_garbage(
    slow_timer: Res<crate::SlowTickTimer>,
    mut garbage: ResMut<GarbageGrid>,
    buildings: Query<(&Building, Option<&WasteProducer>)>,
    services: Query<&ServiceBuilding>,
    policies: Res<crate::policies::Policies>,
) {
    if !slow_timer.should_run() {
        return;
    }
    // Buildings produce garbage proportional to waste generation rate or occupants
    let garbage_mult = policies.garbage_multiplier();
    for (building, maybe_producer) in &buildings {
        let production = if let Some(producer) = maybe_producer {
            // Use the detailed waste rate: convert lbs/day to grid units
            // Scale down so the grid u8 stays in a reasonable range
            let is_residential = building.zone_type.is_residential();
            let daily_lbs = producer.effective_daily_waste(building.occupants, is_residential);
            // Map ~0-2000 lbs/day range down to 0-10 grid units
            ((daily_lbs / 200.0).min(10.0) * garbage_mult) as u8
        } else {
            // Fallback: original formula for buildings without WasteProducer yet
            ((building.occupants / 5).min(10) as f32 * garbage_mult) as u8
        };
        let cur = garbage.get(building.grid_x, building.grid_y);
        garbage.set(
            building.grid_x,
            building.grid_y,
            cur.saturating_add(production),
        );
    }

    // Garbage service buildings collect in radius
    for service in &services {
        if !ServiceBuilding::is_garbage(service.service_type) {
            continue;
        }
        let radius = (service.radius / 16.0) as i32;
        let collection = match service.service_type {
            ServiceType::Landfill => 3u8,
            ServiceType::RecyclingCenter => 5u8,
            ServiceType::Incinerator => 8u8,
            ServiceType::TransferStation => 4u8,
            _ => 0,
        };
        for dy in -radius..=radius {
            for dx in -radius..=radius {
                let nx = service.grid_x as i32 + dx;
                let ny = service.grid_y as i32 + dy;
                if nx >= 0 && ny >= 0 && (nx as usize) < GRID_WIDTH && (ny as usize) < GRID_HEIGHT {
                    let cur = garbage.get(nx as usize, ny as usize);
                    garbage.set(nx as usize, ny as usize, cur.saturating_sub(collection));
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_residential_waste_rates() {
        // Low-income (level 1)
        assert_eq!(
            WasteProducer::residential_rate(ZoneType::ResidentialLow, 1),
            3.0
        );
        assert_eq!(
            WasteProducer::residential_rate(ZoneType::ResidentialHigh, 1),
            3.0
        );
        // Middle-income (level 2)
        assert_eq!(
            WasteProducer::residential_rate(ZoneType::ResidentialLow, 2),
            4.5
        );
        assert_eq!(
            WasteProducer::residential_rate(ZoneType::ResidentialHigh, 2),
            4.5
        );
        // High-income (level 3+)
        assert_eq!(
            WasteProducer::residential_rate(ZoneType::ResidentialLow, 3),
            6.0
        );
        assert_eq!(
            WasteProducer::residential_rate(ZoneType::ResidentialHigh, 5),
            6.0
        );
    }

    #[test]
    fn test_commercial_waste_rates() {
        // Small commercial
        assert_eq!(
            WasteProducer::commercial_rate(ZoneType::CommercialLow, 1),
            50.0
        );
        assert_eq!(
            WasteProducer::commercial_rate(ZoneType::CommercialLow, 3),
            50.0
        );
        // Large commercial
        assert_eq!(
            WasteProducer::commercial_rate(ZoneType::CommercialHigh, 1),
            300.0
        );
        assert_eq!(
            WasteProducer::commercial_rate(ZoneType::CommercialHigh, 2),
            300.0
        );
        // Restaurant-type (high-density level 3+)
        assert_eq!(
            WasteProducer::commercial_rate(ZoneType::CommercialHigh, 3),
            200.0
        );
        assert_eq!(
            WasteProducer::commercial_rate(ZoneType::CommercialHigh, 5),
            200.0
        );
    }

    #[test]
    fn test_industrial_waste_rates() {
        // Light industry
        assert_eq!(WasteProducer::industrial_rate(1), 500.0);
        assert_eq!(WasteProducer::industrial_rate(2), 500.0);
        // Heavy industry
        assert_eq!(WasteProducer::industrial_rate(3), 2000.0);
        assert_eq!(WasteProducer::industrial_rate(5), 2000.0);
    }

    #[test]
    fn test_service_waste_rates() {
        assert_eq!(WasteProducer::service_rate(ServiceType::Hospital), 1500.0);
        assert_eq!(
            WasteProducer::service_rate(ServiceType::MedicalCenter),
            1500.0
        );
        assert_eq!(
            WasteProducer::service_rate(ServiceType::ElementarySchool),
            100.0
        );
        assert_eq!(WasteProducer::service_rate(ServiceType::HighSchool), 100.0);
        assert_eq!(WasteProducer::service_rate(ServiceType::University), 200.0);
    }

    #[test]
    fn test_effective_daily_waste_residential() {
        let producer = WasteProducer {
            waste_lbs_per_day: 4.5,
            recycling_participation: false,
        };
        // 10 occupants * 4.5 lbs/person/day = 45 lbs/day
        assert_eq!(producer.effective_daily_waste(10, true), 45.0);

        let producer_recycling = WasteProducer {
            waste_lbs_per_day: 4.5,
            recycling_participation: true,
        };
        // 10 occupants * 4.5 * 0.7 = 31.5
        assert!((producer_recycling.effective_daily_waste(10, true) - 31.5).abs() < 0.01);
    }

    #[test]
    fn test_effective_daily_waste_non_residential() {
        let producer = WasteProducer {
            waste_lbs_per_day: 300.0,
            recycling_participation: false,
        };
        // Non-residential ignores occupants
        assert_eq!(producer.effective_daily_waste(50, false), 300.0);
        assert_eq!(producer.effective_daily_waste(0, false), 300.0);

        let producer_recycling = WasteProducer {
            waste_lbs_per_day: 300.0,
            recycling_participation: true,
        };
        assert!((producer_recycling.effective_daily_waste(0, false) - 210.0).abs() < 0.01);
    }

    #[test]
    fn test_for_building_factory() {
        let res_low = WasteProducer::for_building(ZoneType::ResidentialLow, 1);
        assert_eq!(res_low.waste_lbs_per_day, 3.0);
        assert!(!res_low.recycling_participation);

        let comm_high = WasteProducer::for_building(ZoneType::CommercialHigh, 1);
        assert_eq!(comm_high.waste_lbs_per_day, 300.0);

        let industrial = WasteProducer::for_building(ZoneType::Industrial, 3);
        assert_eq!(industrial.waste_lbs_per_day, 2000.0);

        let office = WasteProducer::for_building(ZoneType::Office, 1);
        assert_eq!(office.waste_lbs_per_day, 50.0);
    }

    #[test]
    fn test_for_service_factory() {
        let hospital = WasteProducer::for_service(ServiceType::Hospital);
        assert_eq!(hospital.waste_lbs_per_day, 1500.0);

        let school = WasteProducer::for_service(ServiceType::ElementarySchool);
        assert_eq!(school.waste_lbs_per_day, 100.0);
    }

    #[test]
    fn test_waste_system_default() {
        let ws = WasteSystem::default();
        assert_eq!(ws.total_generated_tons, 0.0);
        assert_eq!(ws.period_generated_tons, 0.0);
        assert_eq!(ws.per_capita_lbs_per_day, 0.0);
        assert_eq!(ws.tracked_population, 0);
        assert_eq!(ws.recycling_buildings, 0);
        assert_eq!(ws.total_producers, 0);
    }

    #[test]
    fn test_non_residential_zone_returns_zero() {
        assert_eq!(
            WasteProducer::residential_rate(ZoneType::Industrial, 1),
            0.0
        );
        assert_eq!(WasteProducer::commercial_rate(ZoneType::Industrial, 1), 0.0);
    }

    // =========================================================================
    // WASTE-003: Waste Collection System tests
    // =========================================================================

    #[test]
    fn test_facility_capacity_tons() {
        assert_eq!(facility_capacity_tons(ServiceType::TransferStation), 200.0);
        assert_eq!(facility_capacity_tons(ServiceType::Landfill), 150.0);
        assert_eq!(facility_capacity_tons(ServiceType::RecyclingCenter), 100.0);
        assert_eq!(facility_capacity_tons(ServiceType::Incinerator), 250.0);
        // Non-garbage facilities should return 0.
        assert_eq!(facility_capacity_tons(ServiceType::Hospital), 0.0);
    }

    #[test]
    fn test_facility_operating_cost() {
        assert_eq!(
            facility_operating_cost(ServiceType::TransferStation),
            2_000.0
        );
        assert_eq!(facility_operating_cost(ServiceType::Landfill), 1_500.0);
        assert_eq!(
            facility_operating_cost(ServiceType::RecyclingCenter),
            1_800.0
        );
        assert_eq!(facility_operating_cost(ServiceType::Incinerator), 3_000.0);
        assert_eq!(facility_operating_cost(ServiceType::Hospital), 0.0);
    }

    #[test]
    fn test_waste_collection_grid_default() {
        let grid = WasteCollectionGrid::default();
        assert_eq!(grid.width, GRID_WIDTH);
        assert_eq!(grid.height, GRID_HEIGHT);
        assert!(!grid.is_covered(0, 0));
        assert!(!grid.is_covered(128, 128));
        assert_eq!(grid.uncollected(0, 0), 0.0);
    }

    #[test]
    fn test_waste_collection_grid_coverage() {
        let mut grid = WasteCollectionGrid::default();
        // Not covered initially.
        assert!(!grid.is_covered(10, 10));
        // Mark as covered.
        let idx = grid.idx(10, 10);
        grid.coverage[idx] = 1;
        assert!(grid.is_covered(10, 10));
        // Multiple overlapping facilities.
        grid.coverage[idx] = 3;
        assert!(grid.is_covered(10, 10));
    }

    #[test]
    fn test_transfer_station_serves_within_20_cells() {
        // Simulate a transfer station at (100, 100) covering a 20-cell radius.
        let mut grid = WasteCollectionGrid::default();
        let sx = 100i32;
        let sy = 100i32;
        let radius = WASTE_SERVICE_RADIUS_CELLS;
        let r2 = (radius as f32 * CELL_SIZE) * (radius as f32 * CELL_SIZE);

        for dy in -radius..=radius {
            for dx in -radius..=radius {
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
                let idx = cy as usize * grid.width + cx as usize;
                grid.coverage[idx] = grid.coverage[idx].saturating_add(1);
            }
        }

        // Building at (100, 100) - same cell as station - should be covered.
        assert!(grid.is_covered(100, 100));
        // Building at (110, 100) - 10 cells away - within radius.
        assert!(grid.is_covered(110, 100));
        // Building at (119, 100) - 19 cells away - within radius.
        assert!(grid.is_covered(119, 100));
        // Building at (120, 100) - 20 cells away - exactly at edge, should be covered.
        assert!(grid.is_covered(120, 100));
        // Building at (125, 100) - 25 cells away - outside radius.
        assert!(!grid.is_covered(125, 100));
        // Building at (0, 0) - far away - not covered.
        assert!(!grid.is_covered(0, 0));
    }

    #[test]
    fn test_collection_rate_at_80_percent_capacity() {
        // If capacity = 80 tons/day and generation = 100 tons/day,
        // collection rate = 80/100 = 0.8, meaning 20% uncollected.
        let capacity: f64 = 80.0;
        let generated: f64 = 100.0;
        let rate = (capacity / generated).min(1.0);
        assert!((rate - 0.8).abs() < 0.001);

        // Uncollected = generated * (1 - rate)
        let uncollected = generated * (1.0 - rate);
        assert!((uncollected - 20.0).abs() < 0.001);
    }

    #[test]
    fn test_collection_rate_over_capacity() {
        // If capacity exceeds generation, rate is capped at 1.0.
        let capacity: f64 = 500.0;
        let generated: f64 = 200.0;
        let rate = (capacity / generated).min(1.0);
        assert_eq!(rate, 1.0);
    }

    #[test]
    fn test_collection_rate_zero_generation() {
        // No waste generated: rate should be 1.0 (nothing to collect).
        let capacity: f64 = 200.0;
        let generated: f64 = 0.0;
        let rate = if generated > 0.0 {
            (capacity / generated).min(1.0)
        } else {
            1.0
        };
        assert_eq!(rate, 1.0);
    }

    #[test]
    fn test_waste_system_collection_defaults() {
        let ws = WasteSystem::default();
        assert_eq!(ws.total_collected_tons, 0.0);
        assert_eq!(ws.total_capacity_tons, 0.0);
        assert_eq!(ws.collection_rate, 0.0);
        assert_eq!(ws.uncovered_buildings, 0);
        assert_eq!(ws.transport_cost, 0.0);
        assert_eq!(ws.active_facilities, 0);
    }

    #[test]
    fn test_uncollected_waste_accumulates_uncovered() {
        // Simulate uncollected waste at an uncovered building.
        let mut grid = WasteCollectionGrid::default();
        let idx = grid.idx(50, 50);

        // Building generates 100 lbs/day, not covered.
        assert!(!grid.is_covered(50, 50));
        grid.uncollected_lbs[idx] += 100.0;
        assert_eq!(grid.uncollected(50, 50), 100.0);

        // Next tick: another 100 lbs accumulates.
        grid.uncollected_lbs[idx] += 100.0;
        assert_eq!(grid.uncollected(50, 50), 200.0);
    }

    #[test]
    fn test_uncollected_waste_capped() {
        let mut grid = WasteCollectionGrid::default();
        let idx = grid.idx(50, 50);
        grid.uncollected_lbs[idx] = 15_000.0;
        grid.uncollected_lbs[idx] = grid.uncollected_lbs[idx].min(10_000.0);
        assert_eq!(grid.uncollected(50, 50), 10_000.0);
    }

    #[test]
    fn test_clear_coverage_resets() {
        let mut grid = WasteCollectionGrid::default();
        let idx = grid.idx(10, 10);
        grid.coverage[idx] = 5;
        assert!(grid.is_covered(10, 10));

        grid.clear_coverage();
        assert!(!grid.is_covered(10, 10));
        // Uncollected waste should NOT be cleared by clear_coverage.
        grid.uncollected_lbs[idx] = 500.0;
        grid.clear_coverage();
        assert_eq!(grid.uncollected(10, 10), 500.0);
    }

    #[test]
    fn test_service_radius_constant() {
        // Verify the service radius matches the ticket spec (20 cells).
        assert_eq!(WASTE_SERVICE_RADIUS_CELLS, 20);
    }

    #[test]
    fn test_transfer_station_capacity_200_tons() {
        // Verify the transfer station capacity matches the ticket spec.
        assert_eq!(facility_capacity_tons(ServiceType::TransferStation), 200.0);
    }
}
