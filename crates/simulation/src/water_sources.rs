use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::grid::{CellType, WorldGrid};
use crate::groundwater::{GroundwaterGrid, WaterQualityGrid};
use crate::pollution::PollutionGrid;
use crate::water_demand::WaterSupply;
use crate::SlowTickTimer;

// =============================================================================
// Constants
// =============================================================================

/// 1 MGD = 1,000,000 gallons per day.
const MGD_TO_GPD: f32 = 1_000_000.0;

/// Groundwater well capacity in MGD.
const WELL_CAPACITY_MGD: f32 = 0.5;

/// Surface water intake capacity in MGD.
const SURFACE_INTAKE_CAPACITY_MGD: f32 = 5.0;

/// Reservoir capacity in MGD.
const RESERVOIR_CAPACITY_MGD: f32 = 20.0;

/// Desalination plant capacity in MGD.
const DESALINATION_CAPACITY_MGD: f32 = 10.0;

/// Reservoir storage buffer in days.
const RESERVOIR_BUFFER_DAYS: u32 = 90;

/// Reservoir footprint in grid cells (width x height).
const RESERVOIR_FOOTPRINT: (usize, usize) = (8, 8);

// =============================================================================
// Types
// =============================================================================

/// The type of water supply source.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WaterSourceType {
    /// Groundwater well pump. Low cost, depends on groundwater level/quality.
    Well,
    /// Surface water intake. Must be placed on/near water cell.
    SurfaceIntake,
    /// Reservoir. Large footprint, stores 90-day buffer.
    Reservoir,
    /// Desalination plant. Placed on coast, very high cost, consistent quality.
    Desalination,
}

impl WaterSourceType {
    pub fn name(self) -> &'static str {
        match self {
            WaterSourceType::Well => "Groundwater Well",
            WaterSourceType::SurfaceIntake => "Surface Water Intake",
            WaterSourceType::Reservoir => "Reservoir",
            WaterSourceType::Desalination => "Desalination Plant",
        }
    }

    /// Base capacity in MGD (million gallons per day).
    pub fn capacity_mgd(self) -> f32 {
        match self {
            WaterSourceType::Well => WELL_CAPACITY_MGD,
            WaterSourceType::SurfaceIntake => SURFACE_INTAKE_CAPACITY_MGD,
            WaterSourceType::Reservoir => RESERVOIR_CAPACITY_MGD,
            WaterSourceType::Desalination => DESALINATION_CAPACITY_MGD,
        }
    }

    /// Construction cost.
    pub fn build_cost(self) -> f64 {
        match self {
            WaterSourceType::Well => 500.0,
            WaterSourceType::SurfaceIntake => 3_000.0,
            WaterSourceType::Reservoir => 15_000.0,
            WaterSourceType::Desalination => 20_000.0,
        }
    }

    /// Monthly operating cost (base, before quality adjustments).
    pub fn operating_cost(self) -> f64 {
        match self {
            WaterSourceType::Well => 15.0,
            WaterSourceType::SurfaceIntake => 80.0,
            WaterSourceType::Reservoir => 200.0,
            WaterSourceType::Desalination => 500.0,
        }
    }

    /// Footprint in grid cells (width, height).
    pub fn footprint(self) -> (usize, usize) {
        match self {
            WaterSourceType::Well => (1, 1),
            WaterSourceType::SurfaceIntake => (2, 2),
            WaterSourceType::Reservoir => RESERVOIR_FOOTPRINT,
            WaterSourceType::Desalination => (3, 3),
        }
    }

    /// Base water quality output (0.0 = contaminated, 1.0 = pure).
    pub fn base_quality(self) -> f32 {
        match self {
            WaterSourceType::Well => 0.7,
            WaterSourceType::SurfaceIntake => 0.6,
            WaterSourceType::Reservoir => 0.8,
            WaterSourceType::Desalination => 0.95,
        }
    }
}

/// Component attached to water source building entities.
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct WaterSource {
    pub source_type: WaterSourceType,
    /// Effective capacity in MGD, may be reduced by environmental conditions.
    pub capacity_mgd: f32,
    /// Current water quality (0.0-1.0). Degrades when polluted.
    pub quality: f32,
    /// Current effective operating cost (increases with poor quality).
    pub operating_cost: f64,
    /// Grid position (top-left corner for multi-cell buildings).
    pub grid_x: usize,
    pub grid_y: usize,
    /// For reservoirs: current stored water in gallons.
    pub stored_gallons: f32,
    /// For reservoirs: maximum storage capacity in gallons.
    pub storage_capacity: f32,
}

impl WaterSource {
    /// Create a new water source with default values for the given type.
    pub fn new(source_type: WaterSourceType, grid_x: usize, grid_y: usize) -> Self {
        let storage_capacity = if source_type == WaterSourceType::Reservoir {
            source_type.capacity_mgd() * MGD_TO_GPD * RESERVOIR_BUFFER_DAYS as f32
        } else {
            0.0
        };

        Self {
            source_type,
            capacity_mgd: source_type.capacity_mgd(),
            quality: source_type.base_quality(),
            operating_cost: source_type.operating_cost(),
            grid_x,
            grid_y,
            stored_gallons: storage_capacity, // Start full
            storage_capacity,
        }
    }
}

// =============================================================================
// Systems
// =============================================================================

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
fn is_near_water(grid: &WorldGrid, gx: usize, gy: usize, radius: i32) -> bool {
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

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{GRID_HEIGHT, GRID_WIDTH};

    #[test]
    fn test_well_capacity_is_half_mgd() {
        let well = WaterSource::new(WaterSourceType::Well, 10, 10);
        assert!(
            (well.capacity_mgd - 0.5).abs() < f32::EPSILON,
            "Well capacity should be 0.5 MGD, got {}",
            well.capacity_mgd
        );
    }

    #[test]
    fn test_surface_intake_capacity() {
        let intake = WaterSource::new(WaterSourceType::SurfaceIntake, 10, 10);
        assert!(
            (intake.capacity_mgd - 5.0).abs() < f32::EPSILON,
            "Surface intake capacity should be 5.0 MGD, got {}",
            intake.capacity_mgd
        );
    }

    #[test]
    fn test_reservoir_capacity() {
        let reservoir = WaterSource::new(WaterSourceType::Reservoir, 10, 10);
        assert!(
            (reservoir.capacity_mgd - 20.0).abs() < f32::EPSILON,
            "Reservoir capacity should be 20.0 MGD, got {}",
            reservoir.capacity_mgd
        );
    }

    #[test]
    fn test_desalination_capacity() {
        let desal = WaterSource::new(WaterSourceType::Desalination, 10, 10);
        assert!(
            (desal.capacity_mgd - 10.0).abs() < f32::EPSILON,
            "Desalination capacity should be 10.0 MGD, got {}",
            desal.capacity_mgd
        );
    }

    #[test]
    fn test_reservoir_stores_90_day_buffer() {
        let reservoir = WaterSource::new(WaterSourceType::Reservoir, 10, 10);
        let expected_storage = RESERVOIR_CAPACITY_MGD * MGD_TO_GPD * RESERVOIR_BUFFER_DAYS as f32;
        assert!(
            (reservoir.storage_capacity - expected_storage).abs() < 1.0,
            "Reservoir should store 90-day buffer: expected {}, got {}",
            expected_storage,
            reservoir.storage_capacity
        );
        // Verify it starts full
        assert!(
            (reservoir.stored_gallons - expected_storage).abs() < 1.0,
            "Reservoir should start full"
        );
    }

    #[test]
    fn test_well_has_no_storage() {
        let well = WaterSource::new(WaterSourceType::Well, 10, 10);
        assert_eq!(well.storage_capacity, 0.0);
        assert_eq!(well.stored_gallons, 0.0);
    }

    #[test]
    fn test_desalination_highest_quality() {
        let desal = WaterSource::new(WaterSourceType::Desalination, 10, 10);
        let well = WaterSource::new(WaterSourceType::Well, 10, 10);
        let intake = WaterSource::new(WaterSourceType::SurfaceIntake, 10, 10);
        let reservoir = WaterSource::new(WaterSourceType::Reservoir, 10, 10);

        assert!(
            desal.quality > well.quality,
            "Desalination quality should exceed well quality"
        );
        assert!(
            desal.quality > intake.quality,
            "Desalination quality should exceed surface intake quality"
        );
        assert!(
            desal.quality > reservoir.quality,
            "Desalination quality should exceed reservoir quality"
        );
    }

    #[test]
    fn test_operating_cost_hierarchy() {
        // Well < Surface Intake < Reservoir < Desalination
        assert!(
            WaterSourceType::Well.operating_cost()
                < WaterSourceType::SurfaceIntake.operating_cost()
        );
        assert!(
            WaterSourceType::SurfaceIntake.operating_cost()
                < WaterSourceType::Reservoir.operating_cost()
        );
        assert!(
            WaterSourceType::Reservoir.operating_cost()
                < WaterSourceType::Desalination.operating_cost()
        );
    }

    #[test]
    fn test_build_cost_hierarchy() {
        // Well < Surface Intake < Reservoir < Desalination
        assert!(WaterSourceType::Well.build_cost() < WaterSourceType::SurfaceIntake.build_cost());
        assert!(
            WaterSourceType::SurfaceIntake.build_cost() < WaterSourceType::Reservoir.build_cost()
        );
        assert!(
            WaterSourceType::Reservoir.build_cost() < WaterSourceType::Desalination.build_cost()
        );
    }

    #[test]
    fn test_reservoir_footprint_8x8() {
        let (w, h) = WaterSourceType::Reservoir.footprint();
        assert_eq!(w, 8);
        assert_eq!(h, 8);
    }

    #[test]
    fn test_is_near_water_true() {
        let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        grid.get_mut(12, 10).cell_type = CellType::Water;
        assert!(is_near_water(&grid, 10, 10, 2));
    }

    #[test]
    fn test_is_near_water_false() {
        let grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        // Default is Grass, no water
        assert!(!is_near_water(&grid, 10, 10, 2));
    }

    #[test]
    fn test_water_source_type_names() {
        assert_eq!(WaterSourceType::Well.name(), "Groundwater Well");
        assert_eq!(
            WaterSourceType::SurfaceIntake.name(),
            "Surface Water Intake"
        );
        assert_eq!(WaterSourceType::Reservoir.name(), "Reservoir");
        assert_eq!(WaterSourceType::Desalination.name(), "Desalination Plant");
    }

    #[test]
    fn test_mgd_to_gpd_conversion() {
        let well = WaterSource::new(WaterSourceType::Well, 10, 10);
        let supply_gpd = well.capacity_mgd * MGD_TO_GPD;
        assert!(
            (supply_gpd - 500_000.0).abs() < 1.0,
            "0.5 MGD should equal 500,000 GPD, got {}",
            supply_gpd
        );
    }

    #[test]
    fn test_quality_penalty_increases_cost() {
        let base_cost = WaterSourceType::Well.operating_cost();
        // At quality 0.0, cost should double
        let quality = 0.0_f32;
        let penalty = if quality < 0.5 {
            1.0 + (1.0 - quality * 2.0)
        } else {
            1.0
        };
        let adjusted_cost = base_cost * penalty as f64;
        assert!(
            adjusted_cost > base_cost,
            "Adjusted cost {} should exceed base cost {}",
            adjusted_cost,
            base_cost
        );
        assert!(
            (adjusted_cost - base_cost * 2.0).abs() < 0.01,
            "At quality 0, cost should be 2x base"
        );
    }
}

pub struct WaterSourcesPlugin;

impl Plugin for WaterSourcesPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (
                update_water_sources,
                aggregate_water_source_supply,
                replenish_reservoirs,
            )
                .after(crate::imports_exports::process_trade)
                .in_set(crate::SimulationSet::Simulation),
        );
    }
}
