use crate::config::{GRID_HEIGHT, GRID_WIDTH};
use crate::grid::ZoneType;
use crate::services::ServiceType;
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
        let rate = if zone.is_mixed_use() {
            // MixedUse: combination of residential and commercial waste
            let res_rate = Self::residential_rate(ZoneType::ResidentialHigh, level);
            let comm_rate = Self::commercial_rate(ZoneType::CommercialLow, level);
            (res_rate + comm_rate) * 0.5
        } else if zone.is_residential() {
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
    pub(crate) fn idx(&self, x: usize, y: usize) -> usize {
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
