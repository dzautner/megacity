use bevy::prelude::*;

use crate::grid::{CellType, WorldGrid};
use crate::groundwater::{GroundwaterGrid, WaterQualityGrid};
use crate::pollution::PollutionGrid;
use crate::water_demand::WaterSupply;
use crate::SlowTickTimer;

use super::types::{
    WaterSource, WaterSourceType, DESALINATION_CAPACITY_MGD, MGD_TO_GPD, RESERVOIR_CAPACITY_MGD,
    SURFACE_INTAKE_CAPACITY_MGD, WELL_CAPACITY_MGD,
};

/// System: Update water source capacity and quality based on environment.
///
/// - Wells: capacity depends on groundwater level, quality depends on groundwater quality.
/// - Surface intakes: quality depends on water pollution at the cell.
/// - Reservoirs: quality slowly degrades from pollution, stored water depletes/replenishes.
/// - Desalination: consistent quality, unaffected by pollution.
///
/// Also adjusts operating cost: poor quality increases treatment cost.
pub fn update_water_sources(
    timer: Res<SlowTickTimer>,
    groundwater: Res<GroundwaterGrid>,
    water_quality: Res<WaterQualityGrid>,
    pollution: Res<PollutionGrid>,
    grid: Res<WorldGrid>,
    mut sources: Query<&mut WaterSource>,
) {
    if !timer.should_run() {
        return;
    }

    for mut source in &mut sources {
        let gx = source.grid_x;
        let gy = source.grid_y;

        match source.source_type {
            WaterSourceType::Well => {
                // Capacity scales with groundwater level (0-255 mapped to 0-100%)
                let gw_level = groundwater.get(gx, gy) as f32 / 255.0;
                source.capacity_mgd = WELL_CAPACITY_MGD * gw_level;

                // Quality from groundwater quality grid
                let gw_quality = water_quality.get(gx, gy) as f32 / 255.0;
                source.quality = gw_quality;
            }
            WaterSourceType::SurfaceIntake => {
                // Quality depends on pollution at the cell
                let poll = pollution.get(gx, gy) as f32 / 255.0;
                source.quality = (1.0 - poll * 0.8).max(0.1);

                // Capacity is constant if adjacent to water, zero otherwise
                let near_water = is_near_water(&grid, gx, gy, 2);
                source.capacity_mgd = if near_water {
                    SURFACE_INTAKE_CAPACITY_MGD
                } else {
                    0.0
                };
            }
            WaterSourceType::Reservoir => {
                // Quality slowly degrades from air pollution
                let poll = pollution.get(gx, gy) as f32 / 255.0;
                let quality_loss = poll * 0.05;
                source.quality = (source.quality - quality_loss).max(0.2);

                // Natural quality recovery (slow)
                source.quality = (source.quality + 0.01).min(0.95);

                // Storage: replenish from rainfall (handled elsewhere),
                // deplete from supply. For now, assume steady state.
                let daily_output = RESERVOIR_CAPACITY_MGD * MGD_TO_GPD;
                source.stored_gallons = (source.stored_gallons - daily_output).max(0.0);

                // Capacity depends on stored water
                if source.storage_capacity > 0.0 {
                    let fill_ratio = source.stored_gallons / source.storage_capacity;
                    source.capacity_mgd = RESERVOIR_CAPACITY_MGD * fill_ratio;
                }
            }
            WaterSourceType::Desalination => {
                // Consistent quality, unaffected by environment
                source.quality = 0.95;
                source.capacity_mgd = DESALINATION_CAPACITY_MGD;
            }
        }

        // Operating cost increases when quality is low (more treatment needed)
        let base_cost = source.source_type.operating_cost();
        let quality_penalty: f64 = if source.quality < 0.5 {
            // Double cost at quality 0.0, linear scale
            1.0 + (1.0 - source.quality as f64 * 2.0)
        } else {
            1.0
        };
        source.operating_cost = base_cost * quality_penalty;
    }
}

/// System: Aggregate water supply from all WaterSource entities into WaterSupply resource.
/// Adds to the existing supply from utility infrastructure.
pub fn aggregate_water_source_supply(
    timer: Res<SlowTickTimer>,
    mut water_supply: ResMut<WaterSupply>,
    sources: Query<&WaterSource>,
) {
    if !timer.should_run() {
        return;
    }

    let mut source_supply_gpd: f32 = 0.0;
    for source in &sources {
        source_supply_gpd += source.capacity_mgd * MGD_TO_GPD;
    }

    // Add to total supply (existing utility supply is already computed in water_demand.rs)
    water_supply.total_supply_gpd += source_supply_gpd;

    // Recompute supply ratio
    if water_supply.total_demand_gpd > 0.0 {
        water_supply.supply_ratio = water_supply.total_supply_gpd / water_supply.total_demand_gpd;
    }
}

/// System: Replenish reservoir storage during rain.
pub fn replenish_reservoirs(
    timer: Res<SlowTickTimer>,
    weather: Res<crate::weather::Weather>,
    mut sources: Query<&mut WaterSource>,
) {
    if !timer.should_run() {
        return;
    }

    let rain_replenish: f32 = match weather.current_event {
        crate::weather::WeatherCondition::Rain => 0.02,
        crate::weather::WeatherCondition::HeavyRain => 0.05,
        crate::weather::WeatherCondition::Storm => 0.08,
        _ => 0.0,
    };

    if rain_replenish <= 0.0 {
        return;
    }

    for mut source in &mut sources {
        if source.source_type != WaterSourceType::Reservoir {
            continue;
        }
        let replenish = source.storage_capacity * rain_replenish;
        source.stored_gallons = (source.stored_gallons + replenish).min(source.storage_capacity);
    }
}

// =============================================================================
// Helpers
// =============================================================================

/// Check if a grid position is near a water cell within the given radius.
pub(super) fn is_near_water(grid: &WorldGrid, gx: usize, gy: usize, radius: i32) -> bool {
    for dy in -radius..=radius {
        for dx in -radius..=radius {
            let nx = gx as i32 + dx;
            let ny = gy as i32 + dy;
            if nx < 0 || ny < 0 || (nx as usize) >= grid.width || (ny as usize) >= grid.height {
                continue;
            }
            if grid.get(nx as usize, ny as usize).cell_type == CellType::Water {
                return true;
            }
        }
    }
    false
}
