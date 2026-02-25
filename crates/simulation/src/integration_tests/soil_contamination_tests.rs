//! Integration tests for POLL-013: Soil Contamination Grid and Persistence Model.

use crate::buildings::Building;
use crate::grid::ZoneType;
use crate::groundwater::WaterQualityGrid;
use crate::landfill::{LandfillLinerType, LandfillState};
use crate::services::{ServiceBuilding, ServiceType};
use crate::soil_contamination::{SoilContaminationGrid, UPDATE_INTERVAL};
use crate::test_harness::TestCity;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Spawn an industrial building at (x, y) with the given level.
fn spawn_industrial(city: &mut TestCity, x: usize, y: usize, level: u8) {
    let world = city.world_mut();
    let entity = world
        .spawn(Building {
            zone_type: ZoneType::Industrial,
            level,
            grid_x: x,
            grid_y: y,
            capacity: 50,
            occupants: 0,
        })
        .id();
    let mut grid = world.resource_mut::<crate::grid::WorldGrid>();
    if grid.in_bounds(x, y) {
        grid.get_mut(x, y).building_id = Some(entity);
        grid.get_mut(x, y).zone = ZoneType::Industrial;
    }
}

/// Spawn a landfill service building at (x, y).
fn spawn_landfill_service(city: &mut TestCity, x: usize, y: usize) {
    let world = city.world_mut();
    world.spawn(ServiceBuilding {
        service_type: ServiceType::Landfill,
        grid_x: x,
        grid_y: y,
        radius: ServiceBuilding::coverage_radius(ServiceType::Landfill),
    });
}

/// Run enough ticks for one soil contamination update cycle.
fn tick_soil_cycle(city: &mut TestCity) {
    city.tick(UPDATE_INTERVAL);
}

/// Run N soil contamination update cycles.
fn tick_soil_cycles(city: &mut TestCity, n: u32) {
    city.tick(UPDATE_INTERVAL * n);
}

// ---------------------------------------------------------------------------
// Resource existence
// ---------------------------------------------------------------------------

#[test]
fn test_soil_contamination_grid_exists() {
    let city = TestCity::new();
    let grid = city.resource::<SoilContaminationGrid>();
    assert_eq!(grid.levels.len(), 256 * 256);
    assert_eq!(grid.get(0, 0), 0.0);
}

// ---------------------------------------------------------------------------
// Industrial source contamination
// ---------------------------------------------------------------------------

#[test]
fn test_industrial_building_contaminates_soil() {
    let mut city = TestCity::new();
    spawn_industrial(&mut city, 50, 50, 3);

    tick_soil_cycle(&mut city);

    let grid = city.resource::<SoilContaminationGrid>();
    let contamination = grid.get(50, 50);
    // Level 3 industrial: 3.0 * 3 = 9.0 per cycle
    assert!(
        contamination > 8.0,
        "Industrial building should contaminate soil, got {}",
        contamination
    );
}

#[test]
fn test_industrial_contamination_scales_with_level() {
    let mut city = TestCity::new();
    spawn_industrial(&mut city, 50, 50, 1);
    spawn_industrial(&mut city, 100, 100, 5);

    tick_soil_cycle(&mut city);

    let grid = city.resource::<SoilContaminationGrid>();
    let low_level = grid.get(50, 50);
    let high_level = grid.get(100, 100);
    assert!(
        high_level > low_level,
        "Higher level industrial should produce more contamination: L1={}, L5={}",
        low_level,
        high_level
    );
}

// ---------------------------------------------------------------------------
// Landfill source contamination
// ---------------------------------------------------------------------------

#[test]
fn test_landfill_contaminates_soil_via_state() {
    let mut city = TestCity::new();

    // Add a landfill site via LandfillState
    {
        let world = city.world_mut();
        let mut state = world.resource_mut::<LandfillState>();
        state.add_site_with_options(60, 60, 10000.0, LandfillLinerType::Unlined);
    }

    tick_soil_cycle(&mut city);

    let grid = city.resource::<SoilContaminationGrid>();
    let contamination = grid.get(60, 60);
    assert!(
        contamination > 4.0,
        "Unlined landfill should contaminate soil, got {}",
        contamination
    );
}

#[test]
fn test_lined_landfill_less_contamination_than_unlined() {
    let mut city = TestCity::new();

    {
        let world = city.world_mut();
        let mut state = world.resource_mut::<LandfillState>();
        state.add_site_with_options(60, 60, 10000.0, LandfillLinerType::Unlined);
        state.add_site_with_options(120, 120, 10000.0, LandfillLinerType::Lined);
    }

    tick_soil_cycles(&mut city, 3);

    let grid = city.resource::<SoilContaminationGrid>();
    let unlined = grid.get(60, 60);
    let lined = grid.get(120, 120);
    assert!(
        unlined > lined,
        "Unlined landfill ({}) should have more contamination than lined ({})",
        unlined,
        lined
    );
}

// ---------------------------------------------------------------------------
// Landfill service building fallback
// ---------------------------------------------------------------------------

#[test]
fn test_landfill_service_building_fallback() {
    let mut city = TestCity::new();
    spawn_landfill_service(&mut city, 70, 70);

    tick_soil_cycle(&mut city);

    let grid = city.resource::<SoilContaminationGrid>();
    let contamination = grid.get(70, 70);
    assert!(
        contamination > 4.0,
        "Untracked landfill service building should contaminate soil, got {}",
        contamination
    );
}

// ---------------------------------------------------------------------------
// Decay is very slow (near-permanent)
// ---------------------------------------------------------------------------

#[test]
fn test_soil_contamination_decays_very_slowly() {
    let mut city = TestCity::new();

    // Manually set contamination
    {
        let world = city.world_mut();
        let mut grid = world.resource_mut::<SoilContaminationGrid>();
        grid.set(80, 80, 200.0);
    }

    // Run one cycle (no sources, only decay)
    tick_soil_cycle(&mut city);

    let grid = city.resource::<SoilContaminationGrid>();
    let after_one_cycle = grid.get(80, 80);
    let loss_pct = (200.0 - after_one_cycle) / 200.0 * 100.0;
    assert!(
        loss_pct < 0.1,
        "Soil contamination should lose less than 0.1% per cycle, lost {:.4}%",
        loss_pct
    );
    assert!(
        after_one_cycle > 199.0,
        "Contamination should remain nearly intact: {}",
        after_one_cycle
    );
}

// ---------------------------------------------------------------------------
// Lateral spread
// ---------------------------------------------------------------------------

#[test]
fn test_lateral_spread_above_threshold() {
    let mut city = TestCity::new();

    // Set contamination above spread threshold (50)
    {
        let world = city.world_mut();
        let mut grid = world.resource_mut::<SoilContaminationGrid>();
        grid.set(100, 100, 200.0);
    }

    tick_soil_cycle(&mut city);

    let grid = city.resource::<SoilContaminationGrid>();
    // Cardinal neighbors should have received some contamination
    let neighbor_contam = grid.get(101, 100);
    assert!(
        neighbor_contam > 0.0,
        "Neighbor cell should receive lateral spread, got {}",
        neighbor_contam
    );
}

#[test]
fn test_no_lateral_spread_below_threshold() {
    let mut city = TestCity::new();

    // Set contamination below spread threshold (50)
    {
        let world = city.world_mut();
        let mut grid = world.resource_mut::<SoilContaminationGrid>();
        grid.set(100, 100, 30.0);
    }

    tick_soil_cycle(&mut city);

    let grid = city.resource::<SoilContaminationGrid>();
    let neighbor_contam = grid.get(101, 100);
    assert_eq!(
        neighbor_contam, 0.0,
        "Below threshold, neighbors should not receive spread, got {}",
        neighbor_contam
    );
}

// ---------------------------------------------------------------------------
// Demolished building retains contamination
// ---------------------------------------------------------------------------

#[test]
fn test_demolished_building_retains_contamination() {
    let mut city = TestCity::new();
    spawn_industrial(&mut city, 50, 50, 3);

    // Build up contamination over several cycles
    tick_soil_cycles(&mut city, 5);

    let before_demolish = city.resource::<SoilContaminationGrid>().get(50, 50);
    assert!(
        before_demolish > 0.0,
        "Should have contamination before demolish"
    );

    // Demolish the building (despawn entity, clear grid)
    {
        let world = city.world_mut();
        let entity = {
            let grid = world.resource::<crate::grid::WorldGrid>();
            grid.get(50, 50).building_id
        };
        if let Some(entity) = entity {
            world.despawn(entity);
        }
        let mut grid = world.resource_mut::<crate::grid::WorldGrid>();
        grid.get_mut(50, 50).building_id = None;
        grid.get_mut(50, 50).zone = ZoneType::None;
    }

    // Run more cycles - contamination should persist without the source
    tick_soil_cycles(&mut city, 3);

    let after_demolish = city.resource::<SoilContaminationGrid>().get(50, 50);
    // Contamination decays very slowly, so it should still be significant
    assert!(
        after_demolish > before_demolish * 0.9,
        "Contamination should persist after demolition: before={}, after={}",
        before_demolish,
        after_demolish
    );
}

// ---------------------------------------------------------------------------
// Groundwater seepage
// ---------------------------------------------------------------------------

#[test]
fn test_soil_contamination_seeps_into_groundwater() {
    let mut city = TestCity::new();

    // Record initial water quality
    let initial_quality = city.resource::<WaterQualityGrid>().get(90, 90);

    // Set significant soil contamination
    {
        let world = city.world_mut();
        let mut grid = world.resource_mut::<SoilContaminationGrid>();
        grid.set(90, 90, 300.0);
    }

    tick_soil_cycle(&mut city);

    let final_quality = city.resource::<WaterQualityGrid>().get(90, 90);
    assert!(
        final_quality < initial_quality,
        "Groundwater quality should decrease from soil contamination: initial={}, final={}",
        initial_quality,
        final_quality
    );
}

// ---------------------------------------------------------------------------
// Multiple cycles accumulate contamination
// ---------------------------------------------------------------------------

#[test]
fn test_contamination_accumulates_over_cycles() {
    let mut city = TestCity::new();
    spawn_industrial(&mut city, 50, 50, 2);

    tick_soil_cycle(&mut city);
    let after_1 = city.resource::<SoilContaminationGrid>().get(50, 50);

    tick_soil_cycles(&mut city, 4);
    let after_5 = city.resource::<SoilContaminationGrid>().get(50, 50);

    assert!(
        after_5 > after_1,
        "Contamination should accumulate: after_1={}, after_5={}",
        after_1,
        after_5
    );
}
