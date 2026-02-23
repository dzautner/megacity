//! CSO calculation helpers, overflow detection system, and plugin registration.

use bevy::prelude::*;

use crate::grid::{CellType, WorldGrid};
use crate::stormwater::StormwaterGrid;
use crate::SlowTickTimer;

use super::{
    CsoEvent, SewerSystemState, SewerType, BASE_COMBINED_CAPACITY_PER_CELL,
    GALLONS_PER_CAPITA_PER_DAY, POLLUTION_PER_GALLON_CSO, STORMWATER_TO_SEWER_FACTOR,
};

// =============================================================================
// Helper functions
// =============================================================================

/// Estimate the city's sewage flow in gallons per hour from population.
///
/// `sewage_gph = population * GALLONS_PER_CAPITA_PER_DAY / 24.0`
pub(crate) fn sewage_flow_gph(population: u32) -> f32 {
    population as f32 * GALLONS_PER_CAPITA_PER_DAY / 24.0
}

/// Calculate stormwater inflow to the combined sewer in gallons per hour.
///
/// Only a fraction (`STORMWATER_TO_SEWER_FACTOR`) of total runoff enters the
/// combined sewer; the remainder flows overland or into separate drainage.
pub(crate) fn stormwater_inflow_gph(total_runoff: f32) -> f32 {
    total_runoff * STORMWATER_TO_SEWER_FACTOR
}

/// Count road cells in the world grid (these represent sewer-serviced cells).
pub(crate) fn count_road_cells(grid: &WorldGrid) -> u32 {
    grid.cells
        .iter()
        .filter(|c| c.cell_type == CellType::Road)
        .count() as u32
}

/// Calculate the combined sewer capacity from road cells.
///
/// Only cells that are NOT separated contribute to combined sewer capacity.
/// Separated cells don't carry stormwater through the combined pipe.
pub(crate) fn calculate_combined_capacity(total_road_cells: u32, separated_cells: u32) -> f32 {
    let combined_cells = total_road_cells.saturating_sub(separated_cells);
    combined_cells as f32 * BASE_COMBINED_CAPACITY_PER_CELL
}

/// Calculate the effective combined flow.
///
/// For combined sewers: sewage + stormwater inflow.
/// For fully separated sewers: only sewage (stormwater goes to storm drains).
/// For partially separated: blended based on separation_coverage.
pub(crate) fn calculate_combined_flow(
    sewage_gph: f32,
    stormwater_gph: f32,
    separation_coverage: f32,
) -> f32 {
    // The unseparated fraction of the city still sends stormwater into the sewer.
    let unseparated_fraction = 1.0 - separation_coverage;
    sewage_gph + stormwater_gph * unseparated_fraction
}

// =============================================================================
// System
// =============================================================================

/// Main CSO update system. Runs on `SlowTickTimer`.
///
/// 1. Estimate sewage flow from population (gallons per capita per day / 24).
/// 2. Calculate stormwater inflow from `StormwaterGrid.total_runoff`.
/// 3. Combined flow = sewage + stormwater (adjusted for separation coverage).
/// 4. If combined flow > combined capacity: CSO occurs.
/// 5. CSO discharge = overflow (combined_flow - capacity).
/// 6. Separated sewers route stormwater to storm drains (no CSO contribution).
/// 7. Track CSO frequency for environmental compliance.
/// 8. Pollution contribution proportional to CSO discharge.
pub fn update_sewer_overflow(
    slow_timer: Res<SlowTickTimer>,
    mut sewer_state: ResMut<SewerSystemState>,
    grid: Res<WorldGrid>,
    stormwater: Res<StormwaterGrid>,
    mut cso_events: EventWriter<CsoEvent>,
) {
    if !slow_timer.should_run() {
        return;
    }

    // --- Phase 1: Count road cells and update sewer infrastructure stats ---
    let total_road_cells = count_road_cells(&grid);
    sewer_state.total_sewer_cells = total_road_cells;

    let separation_coverage = if total_road_cells > 0 {
        (sewer_state.cells_with_separated_sewer as f32 / total_road_cells as f32).clamp(0.0, 1.0)
    } else {
        0.0
    };
    sewer_state.separation_coverage = separation_coverage;

    // Update the sewer_type based on coverage threshold.
    // If >95% separated, consider the system as Separated.
    sewer_state.sewer_type = if separation_coverage > 0.95 {
        SewerType::Separated
    } else {
        SewerType::Combined
    };

    // --- Phase 2: Calculate combined sewer capacity ---
    let combined_capacity =
        calculate_combined_capacity(total_road_cells, sewer_state.cells_with_separated_sewer);
    sewer_state.combined_capacity = combined_capacity;

    // --- Phase 3: Estimate population from road cells ---
    // Use a rough population estimate: ~50 people per road cell as a proxy.
    // In a real integration this would come from CityStats, but we estimate here
    // to keep the module self-contained for the system signature.
    // A typical city has road cells roughly proportional to population.
    let estimated_population = total_road_cells * 50;

    // --- Phase 4: Calculate flows ---
    let sewage_gph = sewage_flow_gph(estimated_population);
    let stormwater_gph = stormwater_inflow_gph(stormwater.total_runoff);
    let combined_flow = calculate_combined_flow(sewage_gph, stormwater_gph, separation_coverage);
    sewer_state.current_flow = combined_flow;

    // --- Phase 5: Check for CSO ---
    if combined_flow > combined_capacity && combined_capacity > 0.0 {
        let discharge = combined_flow - combined_capacity;
        let pollution = discharge * POLLUTION_PER_GALLON_CSO;

        sewer_state.cso_active = true;
        sewer_state.cso_discharge_gallons = discharge;
        sewer_state.cso_events_total += 1;
        sewer_state.cso_events_this_year += 1;
        sewer_state.annual_cso_volume += discharge;
        sewer_state.pollution_contribution = pollution;

        // Fire a Bevy event so other systems can react
        cso_events.send(CsoEvent {
            discharge_gallons: discharge,
            pollution_units: pollution,
        });
    } else {
        // No overflow this tick
        sewer_state.cso_active = false;
        sewer_state.cso_discharge_gallons = 0.0;
        sewer_state.pollution_contribution = 0.0;
    }
}

// =============================================================================
// Plugin
// =============================================================================

pub struct CsoPlugin;

impl Plugin for CsoPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SewerSystemState>()
            .add_event::<CsoEvent>()
            .add_systems(
                FixedUpdate,
                update_sewer_overflow
                    .after(crate::imports_exports::process_trade)
                    .in_set(crate::SimulationSet::Simulation),
            );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{GRID_HEIGHT, GRID_WIDTH};

    // -------------------------------------------------------------------------
    // Sewage flow calculation tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_sewage_flow_gph_zero_population() {
        let flow = sewage_flow_gph(0);
        assert_eq!(flow, 0.0);
    }

    #[test]
    fn test_sewage_flow_gph_single_person() {
        let flow = sewage_flow_gph(1);
        let expected = 80.0_f32 / 24.0_f32; // ~3.333 gallons/hr
        assert!((flow - expected).abs() < 0.01_f32);
    }

    #[test]
    fn test_sewage_flow_gph_thousand_people() {
        let flow = sewage_flow_gph(1000);
        let expected = 1000.0_f32 * 80.0_f32 / 24.0_f32; // ~3333.33 gallons/hr
        assert!((flow - expected).abs() < 0.1_f32);
    }

    #[test]
    fn test_sewage_flow_scales_linearly() {
        let flow_100 = sewage_flow_gph(100);
        let flow_200 = sewage_flow_gph(200);
        assert!((flow_200 - flow_100 * 2.0_f32).abs() < 0.01_f32);
    }

    // -------------------------------------------------------------------------
    // Stormwater inflow calculation tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_stormwater_inflow_zero_runoff() {
        let inflow = stormwater_inflow_gph(0.0);
        assert_eq!(inflow, 0.0);
    }

    #[test]
    fn test_stormwater_inflow_half_of_runoff() {
        let total_runoff = 10_000.0_f32;
        let inflow = stormwater_inflow_gph(total_runoff);
        let expected = total_runoff * 0.5;
        assert!((inflow - expected).abs() < f32::EPSILON);
    }

    #[test]
    fn test_stormwater_inflow_scales_linearly() {
        let inflow_a = stormwater_inflow_gph(5_000.0);
        let inflow_b = stormwater_inflow_gph(10_000.0);
        assert!((inflow_b - inflow_a * 2.0_f32).abs() < 0.01_f32);
    }

    // -------------------------------------------------------------------------
    // Road cell counting tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_count_road_cells_empty_grid() {
        let grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        // Default cells are Grass, so no road cells
        let count = count_road_cells(&grid);
        assert_eq!(count, 0);
    }

    #[test]
    fn test_count_road_cells_with_roads() {
        let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        grid.get_mut(0, 0).cell_type = CellType::Road;
        grid.get_mut(1, 0).cell_type = CellType::Road;
        grid.get_mut(2, 0).cell_type = CellType::Road;
        let count = count_road_cells(&grid);
        assert_eq!(count, 3);
    }

    #[test]
    fn test_count_road_cells_ignores_water() {
        let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        grid.get_mut(0, 0).cell_type = CellType::Road;
        grid.get_mut(1, 0).cell_type = CellType::Water;
        let count = count_road_cells(&grid);
        assert_eq!(count, 1);
    }

    // -------------------------------------------------------------------------
    // Combined capacity calculation tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_combined_capacity_no_roads() {
        let capacity = calculate_combined_capacity(0, 0);
        assert_eq!(capacity, 0.0);
    }

    #[test]
    fn test_combined_capacity_all_combined() {
        let capacity = calculate_combined_capacity(10, 0);
        let expected = 10.0 * BASE_COMBINED_CAPACITY_PER_CELL;
        assert!((capacity - expected).abs() < f32::EPSILON);
    }

    #[test]
    fn test_combined_capacity_partially_separated() {
        let capacity = calculate_combined_capacity(10, 3);
        // 7 combined cells remain
        let expected = 7.0 * BASE_COMBINED_CAPACITY_PER_CELL;
        assert!((capacity - expected).abs() < f32::EPSILON);
    }

    #[test]
    fn test_combined_capacity_fully_separated() {
        let capacity = calculate_combined_capacity(10, 10);
        // No combined cells remain
        assert_eq!(capacity, 0.0);
    }

    #[test]
    fn test_combined_capacity_separated_exceeds_total() {
        // Edge case: separated cells > total (shouldn't happen but must not panic)
        let capacity = calculate_combined_capacity(5, 10);
        assert_eq!(capacity, 0.0);
    }

    // -------------------------------------------------------------------------
    // Combined flow calculation tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_combined_flow_no_separation() {
        let sewage = 1000.0_f32;
        let stormwater = 500.0_f32;
        let flow = calculate_combined_flow(sewage, stormwater, 0.0);
        // 0% separated => 100% stormwater enters sewer
        assert!((flow - 1500.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_combined_flow_full_separation() {
        let sewage = 1000.0_f32;
        let stormwater = 500.0_f32;
        let flow = calculate_combined_flow(sewage, stormwater, 1.0);
        // 100% separated => 0% stormwater enters sewer
        assert!((flow - 1000.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_combined_flow_half_separation() {
        let sewage = 1000.0_f32;
        let stormwater = 500.0_f32;
        let flow = calculate_combined_flow(sewage, stormwater, 0.5);
        // 50% separated => 50% stormwater enters sewer
        let expected = 1000.0_f32 + 500.0_f32 * 0.5_f32;
        assert!((flow - expected).abs() < f32::EPSILON);
    }

    #[test]
    fn test_combined_flow_zero_stormwater() {
        let sewage = 1000.0_f32;
        let flow = calculate_combined_flow(sewage, 0.0, 0.0);
        // Only sewage
        assert!((flow - 1000.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_combined_flow_zero_sewage() {
        let stormwater = 500.0_f32;
        let flow = calculate_combined_flow(0.0, stormwater, 0.0);
        // Only stormwater
        assert!((flow - 500.0).abs() < f32::EPSILON);
    }
}
