//! Integration tests for POLL-014: Soil Remediation and Phytoremediation.

use crate::health::HealthGrid;
use crate::land_value::LandValueGrid;
use crate::soil_contamination::{SoilContaminationGrid, UPDATE_INTERVAL};
use crate::soil_remediation::{
    RemediationMethod, SoilRemediationState, BUILDABLE_THRESHOLD,
};
use crate::test_harness::TestCity;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

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
fn test_soil_remediation_state_exists() {
    let city = TestCity::new();
    let state = city.resource::<SoilRemediationState>();
    assert!(state.sites.is_empty());
}

// ---------------------------------------------------------------------------
// Excavation removes contamination quickly
// ---------------------------------------------------------------------------

#[test]
fn test_excavation_removes_contamination_at_10_per_cycle() {
    let mut city = TestCity::new();

    // Set initial contamination
    {
        let world = city.world_mut();
        let mut soil = world.resource_mut::<SoilContaminationGrid>();
        soil.set(50, 50, 100.0);
    }

    // Add excavation site
    {
        let world = city.world_mut();
        let mut state = world.resource_mut::<SoilRemediationState>();
        state.add_site(50, 50, RemediationMethod::Excavation);
    }

    tick_soil_cycle(&mut city);

    let soil = city.resource::<SoilContaminationGrid>();
    let remaining = soil.get(50, 50);
    // After 1 cycle: ~100 * decay - 10 ≈ 90 (decay is negligible)
    assert!(
        remaining < 91.0,
        "Excavation should remove ~10 per cycle, remaining: {}",
        remaining
    );
    assert!(
        remaining > 85.0,
        "Excavation should not remove more than ~10 per cycle, remaining: {}",
        remaining
    );
}

// ---------------------------------------------------------------------------
// Bioremediation is moderate speed
// ---------------------------------------------------------------------------

#[test]
fn test_bioremediation_removes_contamination_at_3_per_cycle() {
    let mut city = TestCity::new();

    {
        let world = city.world_mut();
        let mut soil = world.resource_mut::<SoilContaminationGrid>();
        soil.set(60, 60, 100.0);
    }
    {
        let world = city.world_mut();
        let mut state = world.resource_mut::<SoilRemediationState>();
        state.add_site(60, 60, RemediationMethod::Bioremediation);
    }

    tick_soil_cycle(&mut city);

    let soil = city.resource::<SoilContaminationGrid>();
    let remaining = soil.get(60, 60);
    // After 1 cycle: ~100 * decay - 3 ≈ 97
    assert!(
        remaining < 98.0,
        "Bioremediation should remove ~3 per cycle, remaining: {}",
        remaining
    );
    assert!(
        remaining > 94.0,
        "Bioremediation should not remove too much, remaining: {}",
        remaining
    );
}

// ---------------------------------------------------------------------------
// Phytoremediation is slowest but works
// ---------------------------------------------------------------------------

#[test]
fn test_phytoremediation_is_slower_but_works() {
    let mut city = TestCity::new();

    {
        let world = city.world_mut();
        let mut soil = world.resource_mut::<SoilContaminationGrid>();
        soil.set(70, 70, 50.0);
    }
    {
        let world = city.world_mut();
        let mut state = world.resource_mut::<SoilRemediationState>();
        state.add_site(70, 70, RemediationMethod::Phytoremediation);
    }

    // Run several cycles
    tick_soil_cycles(&mut city, 10);

    let soil = city.resource::<SoilContaminationGrid>();
    let remaining = soil.get(70, 70);
    // After 10 cycles: ~50 - 10*0.5 = ~45 (with tiny decay)
    assert!(
        remaining < 46.0,
        "Phytoremediation should slowly reduce contamination, remaining: {}",
        remaining
    );
    assert!(
        remaining > 40.0,
        "Phytoremediation rate should be slow, remaining: {}",
        remaining
    );
}

// ---------------------------------------------------------------------------
// Excavation is faster than phytoremediation
// ---------------------------------------------------------------------------

#[test]
fn test_excavation_faster_than_phytoremediation() {
    let mut city = TestCity::new();

    // Set same initial contamination for both
    {
        let world = city.world_mut();
        let mut soil = world.resource_mut::<SoilContaminationGrid>();
        soil.set(50, 50, 200.0);
        soil.set(100, 100, 200.0);
    }
    {
        let world = city.world_mut();
        let mut state = world.resource_mut::<SoilRemediationState>();
        state.add_site(50, 50, RemediationMethod::Excavation);
        state.add_site(100, 100, RemediationMethod::Phytoremediation);
    }

    tick_soil_cycles(&mut city, 5);

    let soil = city.resource::<SoilContaminationGrid>();
    let excavation = soil.get(50, 50);
    let phyto = soil.get(100, 100);
    assert!(
        excavation < phyto,
        "Excavation ({}) should clean faster than phytoremediation ({})",
        excavation,
        phyto
    );
}

// ---------------------------------------------------------------------------
// Containment blocks spread
// ---------------------------------------------------------------------------

#[test]
fn test_containment_stops_lateral_spread() {
    let mut city = TestCity::new();

    // Set high contamination above spread threshold (50)
    {
        let world = city.world_mut();
        let mut soil = world.resource_mut::<SoilContaminationGrid>();
        soil.set(100, 100, 200.0);
    }
    {
        let world = city.world_mut();
        let mut state = world.resource_mut::<SoilRemediationState>();
        state.add_site(100, 100, RemediationMethod::Containment);
    }

    tick_soil_cycles(&mut city, 5);

    let soil = city.resource::<SoilContaminationGrid>();
    // Contained cell should still have most of its contamination (only natural decay)
    let center = soil.get(100, 100);
    assert!(
        center > 190.0,
        "Containment should not reduce contamination, center: {}",
        center
    );

    // Neighbors should have less spread than without containment
    let neighbor = soil.get(101, 100);
    // With containment, neighbor should be clamped to at most the center level
    // and any spread should be limited.
    assert!(
        neighbor <= center,
        "Neighbor ({}) should not exceed contained center ({})",
        neighbor,
        center
    );
}

// ---------------------------------------------------------------------------
// Auto-removal of completed remediation
// ---------------------------------------------------------------------------

#[test]
fn test_completed_remediation_auto_removes_site() {
    let mut city = TestCity::new();

    // Set low contamination that will be cleaned quickly
    {
        let world = city.world_mut();
        let mut soil = world.resource_mut::<SoilContaminationGrid>();
        soil.set(50, 50, 15.0);
    }
    {
        let world = city.world_mut();
        let mut state = world.resource_mut::<SoilRemediationState>();
        state.add_site(50, 50, RemediationMethod::Excavation);
    }

    // One cycle should reduce 15 - 10 = 5, which is below threshold (10)
    tick_soil_cycle(&mut city);

    let state = city.resource::<SoilRemediationState>();
    assert!(
        state.sites.is_empty(),
        "Completed remediation site should be auto-removed, sites: {:?}",
        state.sites.len()
    );
}

// ---------------------------------------------------------------------------
// Health penalty for contaminated soil
// ---------------------------------------------------------------------------

#[test]
fn test_health_penalty_for_contaminated_soil() {
    let mut city = TestCity::new();

    // Run one slow tick to stabilize health values (fresh grid starts at 0)
    city.tick_slow_cycle();

    // Record initial health after stabilization
    let initial_health = city.resource::<HealthGrid>().get(80, 80);
    assert!(
        initial_health > 0,
        "Health should be non-zero after stabilization: {}",
        initial_health
    );

    // Set contamination above health threshold
    {
        let world = city.world_mut();
        let mut soil = world.resource_mut::<SoilContaminationGrid>();
        soil.set(80, 80, 200.0);
    }

    // Health penalty runs on slow tick
    city.tick_slow_cycle();

    let health = city.resource::<HealthGrid>().get(80, 80);
    assert!(
        health < initial_health,
        "Health should decrease on contaminated soil: initial={}, after={}",
        initial_health,
        health
    );
}

#[test]
fn test_no_health_penalty_below_threshold() {
    let mut city = TestCity::new();

    // Record initial health
    let initial_health = {
        // Run one slow tick to stabilize health values
        city.tick_slow_cycle();
        city.resource::<HealthGrid>().get(80, 80)
    };

    // Set contamination below threshold
    {
        let world = city.world_mut();
        let mut soil = world.resource_mut::<SoilContaminationGrid>();
        soil.set(80, 80, 20.0); // below 30 threshold
    }

    city.tick_slow_cycle();

    let health = city.resource::<HealthGrid>().get(80, 80);
    // Health should not be penalized for low contamination
    // (may change due to other systems but should not go below initial)
    assert!(
        health >= initial_health.saturating_sub(1),
        "No health penalty below threshold: initial={}, after={}",
        initial_health,
        health
    );
}

// ---------------------------------------------------------------------------
// Land value penalty for contaminated soil
// ---------------------------------------------------------------------------

#[test]
fn test_land_value_penalty_for_contaminated_soil() {
    let mut city = TestCity::new();

    // Stabilize land values first
    city.tick_slow_cycle();
    let initial_lv = city.resource::<LandValueGrid>().get(90, 90);

    // Set high contamination
    {
        let world = city.world_mut();
        let mut soil = world.resource_mut::<SoilContaminationGrid>();
        soil.set(90, 90, 300.0);
    }

    city.tick_slow_cycle();

    let lv = city.resource::<LandValueGrid>().get(90, 90);
    assert!(
        lv < initial_lv,
        "Land value should decrease on contaminated soil: initial={}, after={}",
        initial_lv,
        lv
    );
}

// ---------------------------------------------------------------------------
// Full remediation cycle: contaminate -> remediate -> clean
// ---------------------------------------------------------------------------

#[test]
fn test_full_remediation_lifecycle() {
    let mut city = TestCity::new();

    // Contaminate cell
    {
        let world = city.world_mut();
        let mut soil = world.resource_mut::<SoilContaminationGrid>();
        soil.set(50, 50, 50.0);
    }

    // Start excavation
    {
        let world = city.world_mut();
        let mut state = world.resource_mut::<SoilRemediationState>();
        state.add_site(50, 50, RemediationMethod::Excavation);
    }

    // Run enough cycles to fully clean (50/10 = 5 cycles)
    tick_soil_cycles(&mut city, 6);

    let soil = city.resource::<SoilContaminationGrid>();
    let remaining = soil.get(50, 50);
    assert!(
        remaining < BUILDABLE_THRESHOLD,
        "Cell should be clean after sufficient remediation, remaining: {}",
        remaining
    );

    // Site should be auto-removed
    let state = city.resource::<SoilRemediationState>();
    assert!(
        state.sites.is_empty(),
        "Remediation site should be removed after completion"
    );
}
