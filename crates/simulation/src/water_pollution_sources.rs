//! POLL-006: Water Pollution Point Source Emissions by Building Type
//!
//! Expands water pollution sources beyond just industrial buildings. Each source
//! type has a per-type emission rate. Sewage outfalls have treatment-level-aware
//! emissions.
//!
//! ## Source Types & Base Emission Rates
//!
//! | Source Type         | Base Rate |
//! |---------------------|-----------|
//! | SewageOutfall       | 80 (untreated), adjusted by treatment level |
//! | HeavyIndustry       | 50 |
//! | LightIndustry       | 20 |
//! | PowerPlantCooling   | 15 (thermal pollution) |
//! | LandfillLeachate    | 25 |
//! | AgriculturalRunoff  | 18 |
//! | ConstructionRunoff  | 10 |
//! | CommercialDischarge | 8  |
//!
//! ## Treatment Effectiveness
//!
//! | Level    | Reduction |
//! |----------|-----------|
//! | None     | 0%        |
//! | Primary  | 60%       |
//! | Secondary| 85%       |
//! | Tertiary | 95%       |
//! | Advanced | 99%       |

use bevy::prelude::*;
use bitcode::{Decode, Encode};
use serde::{Deserialize, Serialize};

use crate::buildings::Building;
use crate::config::{GRID_HEIGHT, GRID_WIDTH};
use crate::grid::{CellType, WorldGrid, ZoneType};
use crate::services::{ServiceBuilding, ServiceType};
use crate::utilities::{UtilitySource, UtilityType};
use crate::water_pollution::WaterPollutionGrid;
use crate::water_treatment::TreatmentLevel;
use crate::SlowTickTimer;

// =============================================================================
// Source type enum
// =============================================================================

/// Point source types that emit water pollution.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, Encode, Decode)]
pub enum WaterPollutionSourceType {
    /// Sewage outfall — emission depends on treatment level.
    SewageOutfall,
    /// Heavy industry (level >= 3 industrial buildings).
    HeavyIndustry,
    /// Light industry (level 1-2 industrial buildings).
    LightIndustry,
    /// Power plant cooling water (thermal pollution).
    PowerPlantCooling,
    /// Landfill leachate runoff.
    LandfillLeachate,
    /// Agricultural runoff from farm zones.
    AgriculturalRunoff,
    /// Construction site sediment runoff.
    ConstructionRunoff,
    /// Commercial building discharge (grease, chemicals).
    CommercialDischarge,
}

impl WaterPollutionSourceType {
    /// Base emission rate for this source type (pollution units per slow tick).
    pub fn base_emission_rate(self) -> u8 {
        match self {
            WaterPollutionSourceType::SewageOutfall => 80,
            WaterPollutionSourceType::HeavyIndustry => 50,
            WaterPollutionSourceType::LightIndustry => 20,
            WaterPollutionSourceType::PowerPlantCooling => 15,
            WaterPollutionSourceType::LandfillLeachate => 25,
            WaterPollutionSourceType::AgriculturalRunoff => 18,
            WaterPollutionSourceType::ConstructionRunoff => 10,
            WaterPollutionSourceType::CommercialDischarge => 8,
        }
    }

    /// Pollution spread radius in grid cells.
    pub fn spread_radius(self) -> i32 {
        match self {
            WaterPollutionSourceType::SewageOutfall => 5,
            WaterPollutionSourceType::HeavyIndustry => 4,
            WaterPollutionSourceType::LightIndustry => 3,
            WaterPollutionSourceType::PowerPlantCooling => 4,
            WaterPollutionSourceType::LandfillLeachate => 3,
            WaterPollutionSourceType::AgriculturalRunoff => 3,
            WaterPollutionSourceType::ConstructionRunoff => 2,
            WaterPollutionSourceType::CommercialDischarge => 2,
        }
    }

    /// Display name for the source type.
    pub fn name(self) -> &'static str {
        match self {
            WaterPollutionSourceType::SewageOutfall => "Sewage Outfall",
            WaterPollutionSourceType::HeavyIndustry => "Heavy Industry",
            WaterPollutionSourceType::LightIndustry => "Light Industry",
            WaterPollutionSourceType::PowerPlantCooling => "Power Plant Cooling",
            WaterPollutionSourceType::LandfillLeachate => "Landfill Leachate",
            WaterPollutionSourceType::AgriculturalRunoff => "Agricultural Runoff",
            WaterPollutionSourceType::ConstructionRunoff => "Construction Runoff",
            WaterPollutionSourceType::CommercialDischarge => "Commercial Discharge",
        }
    }
}

// =============================================================================
// Treatment effectiveness
// =============================================================================

/// Returns the emission rate for a sewage outfall given its treatment level.
///
/// Base rate of 80 is reduced by the treatment effectiveness:
/// - None:      80 * (1 - 0.00) = 80
/// - Primary:   80 * (1 - 0.60) = 32
/// - Secondary: 80 * (1 - 0.85) = 12
/// - Tertiary:  80 * (1 - 0.95) = 4
/// - Advanced:  80 * (1 - 0.99) ≈ 1
pub fn sewage_emission_for_treatment(level: TreatmentLevel) -> u8 {
    let base = WaterPollutionSourceType::SewageOutfall.base_emission_rate() as f32;
    let reduced = base * (1.0 - level.removal_efficiency());
    reduced.round() as u8
}

// =============================================================================
// Aggregate tracking resource
// =============================================================================

/// City-wide aggregate tracking of water pollution point sources.
#[derive(Resource, Clone, Debug, Default, Serialize, Deserialize, Encode, Decode)]
pub struct WaterPollutionSourcesState {
    /// Number of active pollution sources by type.
    pub source_counts: [u32; 8],
    /// Total emissions contributed this period by type.
    pub emissions_by_type: [u32; 8],
    /// Total emissions across all source types this period.
    pub total_emissions: u32,
}

impl WaterPollutionSourcesState {
    fn type_index(source_type: WaterPollutionSourceType) -> usize {
        match source_type {
            WaterPollutionSourceType::SewageOutfall => 0,
            WaterPollutionSourceType::HeavyIndustry => 1,
            WaterPollutionSourceType::LightIndustry => 2,
            WaterPollutionSourceType::PowerPlantCooling => 3,
            WaterPollutionSourceType::LandfillLeachate => 4,
            WaterPollutionSourceType::AgriculturalRunoff => 5,
            WaterPollutionSourceType::ConstructionRunoff => 6,
            WaterPollutionSourceType::CommercialDischarge => 7,
        }
    }

    fn record_source(&mut self, source_type: WaterPollutionSourceType, emission: u32) {
        let idx = Self::type_index(source_type);
        self.source_counts[idx] += 1;
        self.emissions_by_type[idx] += emission;
        self.total_emissions += emission;
    }

    fn reset(&mut self) {
        self.source_counts = [0; 8];
        self.emissions_by_type = [0; 8];
        self.total_emissions = 0;
    }

    /// Get the source count for a given type.
    pub fn count_for(&self, source_type: WaterPollutionSourceType) -> u32 {
        self.source_counts[Self::type_index(source_type)]
    }

    /// Get the total emissions for a given type.
    pub fn emissions_for(&self, source_type: WaterPollutionSourceType) -> u32 {
        self.emissions_by_type[Self::type_index(source_type)]
    }
}

impl crate::Saveable for WaterPollutionSourcesState {
    const SAVE_KEY: &'static str = "water_pollution_sources";

    fn save_to_bytes(&self) -> Option<Vec<u8>> {
        if self.total_emissions == 0 && self.source_counts.iter().all(|&c| c == 0) {
            return None;
        }
        bitcode::encode(self).ok()
    }

    fn load_from_bytes(bytes: &[u8]) -> Self {
        crate::decode_or_warn(Self::SAVE_KEY, bytes)
    }
}

// =============================================================================
// Emission application helper
// =============================================================================

/// Apply pollution from a point source at (sx, sy) to nearby water cells.
fn apply_point_source(
    grid: &WorldGrid,
    water_pollution: &mut WaterPollutionGrid,
    sx: usize,
    sy: usize,
    emission_rate: u8,
    radius: i32,
) {
    if emission_rate == 0 {
        return;
    }
    for dy in -radius..=radius {
        for dx in -radius..=radius {
            let nx = sx as i32 + dx;
            let ny = sy as i32 + dy;
            if nx < 0 || ny < 0 || (nx as usize) >= GRID_WIDTH || (ny as usize) >= GRID_HEIGHT {
                continue;
            }
            let ux = nx as usize;
            let uy = ny as usize;

            if grid.get(ux, uy).cell_type != CellType::Water {
                continue;
            }

            let dist = dx.abs() + dy.abs();
            let decay = (emission_rate as i32 - dist * 3).max(0) as u8;
            if decay > 0 {
                let idx = uy * GRID_WIDTH + ux;
                water_pollution.levels[idx] = water_pollution.levels[idx].saturating_add(decay);
            }
        }
    }
}

// =============================================================================
// Main system
// =============================================================================

/// Identify point sources and apply per-type emissions to water pollution grid.
///
/// This system provides comprehensive multi-source water pollution emissions
/// from buildings, utilities, and services. It runs after the base
/// `update_water_pollution` system on each slow tick.
#[allow(clippy::too_many_arguments)]
pub fn update_water_pollution_sources(
    slow_timer: Res<SlowTickTimer>,
    mut state: ResMut<WaterPollutionSourcesState>,
    mut water_pollution: ResMut<WaterPollutionGrid>,
    grid: Res<WorldGrid>,
    buildings: Query<&Building>,
    services: Query<&ServiceBuilding>,
    utilities: Query<&UtilitySource>,
    treatment_state: Res<crate::water_treatment::WaterTreatmentState>,
) {
    if !slow_timer.should_run() {
        return;
    }

    state.reset();

    // --- Industrial buildings ---
    for building in &buildings {
        if building.zone_type == ZoneType::Industrial {
            let (source_type, rate) = if building.level >= 3 {
                (
                    WaterPollutionSourceType::HeavyIndustry,
                    WaterPollutionSourceType::HeavyIndustry.base_emission_rate(),
                )
            } else {
                (
                    WaterPollutionSourceType::LightIndustry,
                    WaterPollutionSourceType::LightIndustry.base_emission_rate(),
                )
            };
            apply_point_source(
                &grid,
                &mut water_pollution,
                building.grid_x,
                building.grid_y,
                rate,
                source_type.spread_radius(),
            );
            state.record_source(source_type, rate as u32);
        }

        // Commercial buildings emit small amounts of pollution
        if building.zone_type == ZoneType::CommercialHigh
            || building.zone_type == ZoneType::CommercialLow
        {
            let source_type = WaterPollutionSourceType::CommercialDischarge;
            let rate = source_type.base_emission_rate();
            apply_point_source(
                &grid,
                &mut water_pollution,
                building.grid_x,
                building.grid_y,
                rate,
                source_type.spread_radius(),
            );
            state.record_source(source_type, rate as u32);
        }
    }

    // --- Sewage outfalls (SewagePlant utilities) ---
    let best_treatment = best_city_treatment_level(&treatment_state);

    for utility in &utilities {
        if utility.utility_type == UtilityType::SewagePlant {
            let rate = sewage_emission_for_treatment(best_treatment);
            let source_type = WaterPollutionSourceType::SewageOutfall;
            apply_point_source(
                &grid,
                &mut water_pollution,
                utility.grid_x,
                utility.grid_y,
                rate,
                source_type.spread_radius(),
            );
            state.record_source(source_type, rate as u32);
        }

        // Power plant cooling water (thermal pollution)
        if utility.utility_type == UtilityType::PowerPlant {
            let source_type = WaterPollutionSourceType::PowerPlantCooling;
            let rate = source_type.base_emission_rate();
            apply_point_source(
                &grid,
                &mut water_pollution,
                utility.grid_x,
                utility.grid_y,
                rate,
                source_type.spread_radius(),
            );
            state.record_source(source_type, rate as u32);
        }
    }

    // --- Service buildings: Landfill leachate ---
    for service in &services {
        if service.service_type == ServiceType::Landfill {
            let source_type = WaterPollutionSourceType::LandfillLeachate;
            let rate = source_type.base_emission_rate();
            apply_point_source(
                &grid,
                &mut water_pollution,
                service.grid_x,
                service.grid_y,
                rate,
                source_type.spread_radius(),
            );
            state.record_source(source_type, rate as u32);
        }
    }

    // --- Construction runoff from level-1 non-industrial buildings ---
    emit_construction_runoff(&grid, &mut water_pollution, &mut state, &buildings);

    // --- Agricultural runoff from water cells adjacent to farmland ---
    emit_agricultural_runoff(&grid, &mut water_pollution, &mut state);
}

/// Emit construction runoff from recently built buildings (level 1).
fn emit_construction_runoff(
    grid: &WorldGrid,
    water_pollution: &mut WaterPollutionGrid,
    state: &mut WaterPollutionSourcesState,
    buildings: &Query<&Building>,
) {
    for building in buildings {
        // Skip industrial — already counted as LightIndustry
        if building.zone_type == ZoneType::Industrial {
            continue;
        }
        if building.level == 1 {
            let source_type = WaterPollutionSourceType::ConstructionRunoff;
            let rate = source_type.base_emission_rate();
            apply_point_source(
                grid,
                water_pollution,
                building.grid_x,
                building.grid_y,
                rate,
                source_type.spread_radius(),
            );
            state.record_source(source_type, rate as u32);
        }
    }
}

/// Emit agricultural runoff from water cells adjacent to unzoned grass.
fn emit_agricultural_runoff(
    grid: &WorldGrid,
    water_pollution: &mut WaterPollutionGrid,
    state: &mut WaterPollutionSourcesState,
) {
    let source_type = WaterPollutionSourceType::AgriculturalRunoff;
    let rate = source_type.base_emission_rate();
    let radius = source_type.spread_radius();

    // Sample every 16th cell to avoid scanning the full 256x256 grid
    let step = 16;
    for y in (0..GRID_HEIGHT).step_by(step) {
        for x in (0..GRID_WIDTH).step_by(step) {
            let cell = grid.get(x, y);
            if cell.cell_type != CellType::Water {
                continue;
            }
            if has_adjacent_grass(grid, x, y) {
                apply_point_source(grid, water_pollution, x, y, rate, radius);
                state.record_source(source_type, rate as u32);
            }
        }
    }
}

/// Check if a cell has any adjacent unzoned grass cells (proxy for farmland).
fn has_adjacent_grass(grid: &WorldGrid, x: usize, y: usize) -> bool {
    let neighbors = [(0i32, 1i32), (0, -1), (1, 0), (-1, 0)];
    for (dx, dy) in neighbors {
        let nx = x as i32 + dx;
        let ny = y as i32 + dy;
        if nx < 0 || ny < 0 || (nx as usize) >= GRID_WIDTH || (ny as usize) >= GRID_HEIGHT {
            continue;
        }
        let cell = grid.get(nx as usize, ny as usize);
        if cell.cell_type == CellType::Grass && cell.zone == ZoneType::None {
            return true;
        }
    }
    false
}

/// Determine the best treatment level from active water treatment plants.
fn best_city_treatment_level(
    treatment_state: &crate::water_treatment::WaterTreatmentState,
) -> TreatmentLevel {
    treatment_state
        .plants
        .values()
        .map(|p| p.level)
        .max_by_key(|level| match level {
            TreatmentLevel::None => 0,
            TreatmentLevel::Primary => 1,
            TreatmentLevel::Secondary => 2,
            TreatmentLevel::Tertiary => 3,
            TreatmentLevel::Advanced => 4,
        })
        .unwrap_or(TreatmentLevel::None)
}

// =============================================================================
// Plugin
// =============================================================================

pub struct WaterPollutionSourcesPlugin;

impl Plugin for WaterPollutionSourcesPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WaterPollutionSourcesState>();

        // Register saveable for save/load
        let mut registry = app
            .world_mut()
            .resource_mut::<crate::SaveableRegistry>();
        registry.register::<WaterPollutionSourcesState>();

        app.add_systems(
            FixedUpdate,
            update_water_pollution_sources
                .after(crate::water_pollution::update_water_pollution)
                .in_set(crate::SimulationSet::Simulation),
        );
    }
}
