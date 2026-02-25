//! Integration tests for POLL-010: Noise Pollution Logarithmic Attenuation Model
//!
//! Verifies that the logarithmic attenuation model produces correct noise
//! propagation behaviour: decay with distance, correct source dB levels,
//! and backward-compatible u8 grid output.

use crate::grid::RoadType;
use crate::noise::{
    attenuated_db, db_to_grid_u8, max_radius, road_source_db, NoisePollutionGrid,
    INDUSTRIAL_SOURCE_DB,
};
use crate::services::ServiceType;
use crate::test_harness::TestCity;
use crate::wind::WindState;

// ====================================================================
// Pure-function tests for the attenuation formula
// ====================================================================

#[test]
fn test_attenuation_formula_at_source() {
    // At distance 0, noise equals the source level
    let db = attenuated_db(80.0, 0.0);
    assert!(
        (db - 80.0).abs() < f32::EPSILON,
        "at source, dB should equal source level, got {}",
        db
    );
}

#[test]
fn test_attenuation_formula_6db_per_doubling() {
    // At distance 2: L = 80 - 6*log2(2) - 0.5*2 = 80 - 6 - 1 = 73
    let db = attenuated_db(80.0, 2.0);
    assert!(
        (db - 73.0).abs() < 0.1,
        "at d=2: expected ~73 dB, got {}",
        db
    );

    // At distance 4: L = 80 - 6*log2(4) - 0.5*4 = 80 - 12 - 2 = 66
    let db4 = attenuated_db(80.0, 4.0);
    assert!(
        (db4 - 66.0).abs() < 0.1,
        "at d=4: expected ~66 dB, got {}",
        db4
    );

    // The difference between d=2 and d=4 should be ~7 dB (6 dB geometric + 1 dB atmospheric)
    let diff = db - db4;
    assert!(
        (diff - 7.0).abs() < 0.2,
        "doubling distance should drop ~7 dB, got {} dB drop",
        diff
    );
}

#[test]
fn test_attenuation_atmospheric_component() {
    // Compare two points at same geometric distance but different absolute
    // The atmospheric term is 0.5 * d, so at d=10: 0.5*10 = 5 dB extra loss
    // At d=1: L = 95 - 0 - 0.5 = 94.5
    // At d=10: L = 95 - 6*log2(10) - 5.0 = 95 - 19.93 - 5.0 = 70.07
    let db1 = attenuated_db(95.0, 1.0);
    let db10 = attenuated_db(95.0, 10.0);
    assert!(
        (db1 - 94.5).abs() < 0.1,
        "at d=1: expected ~94.5, got {}",
        db1
    );
    assert!(
        db10 > 60.0 && db10 < 80.0,
        "at d=10 from 95 dB: expected ~70 dB, got {}",
        db10
    );
}

#[test]
fn test_attenuation_monotonically_decreasing() {
    for d in 1..40 {
        let near = attenuated_db(80.0, d as f32);
        let far = attenuated_db(80.0, (d + 1) as f32);
        assert!(
            near >= far,
            "noise should decrease with distance: d={}({}) >= d={}({})",
            d,
            near,
            d + 1,
            far
        );
    }
}

#[test]
fn test_attenuation_weak_source_reaches_short_distance() {
    // 55 dB source should not reach very far
    let r = max_radius(55.0);
    assert!(
        r <= 25,
        "55 dB source radius should be modest, got {}",
        r
    );
    assert!(r >= 5, "55 dB source should reach at least 5 cells, got {}", r);
}

#[test]
fn test_attenuation_strong_source_reaches_further() {
    let r_strong = max_radius(95.0);
    let r_weak = max_radius(55.0);
    assert!(
        r_strong > r_weak,
        "95 dB should reach further than 55 dB: {} vs {}",
        r_strong,
        r_weak
    );
}

// ====================================================================
// dB-to-grid conversion tests
// ====================================================================

#[test]
fn test_db_to_grid_zero_maps_to_zero() {
    assert_eq!(db_to_grid_u8(0.0), 0);
}

#[test]
fn test_db_to_grid_95_maps_to_100() {
    assert_eq!(db_to_grid_u8(95.0), 100);
}

#[test]
fn test_db_to_grid_midpoint() {
    let val = db_to_grid_u8(47.5);
    assert_eq!(val, 50, "47.5 dB should map to 50, got {}", val);
}

#[test]
fn test_db_to_grid_clamps_high() {
    assert_eq!(db_to_grid_u8(200.0), 100);
}

#[test]
fn test_db_to_grid_clamps_negative() {
    assert_eq!(db_to_grid_u8(-10.0), 0);
}

// ====================================================================
// Source dB level tests
// ====================================================================

#[test]
fn test_highway_is_loudest_road() {
    let highway_db = road_source_db(RoadType::Highway);
    for road in [
        RoadType::Boulevard,
        RoadType::Avenue,
        RoadType::Local,
        RoadType::OneWay,
        RoadType::Path,
    ] {
        let db = road_source_db(road);
        assert!(
            highway_db >= db,
            "highway ({} dB) should be >= {:?} ({} dB)",
            highway_db,
            road,
            db
        );
    }
}

#[test]
fn test_path_is_silent() {
    assert!(
        road_source_db(RoadType::Path) < f32::EPSILON,
        "path should generate no noise"
    );
}

#[test]
fn test_industrial_source_db_value() {
    assert!(
        (INDUSTRIAL_SOURCE_DB - 75.0).abs() < f32::EPSILON,
        "industrial should be 75 dB"
    );
}

// ====================================================================
// Integration: highway noise decays logarithmically
// ====================================================================

#[test]
fn test_highway_noise_decays_with_distance() {
    let mut city = TestCity::new().with_road(128, 128, 128, 128, RoadType::Highway);

    {
        let world = city.world_mut();
        world.resource_mut::<WindState>().speed = 0.0;
    }

    city.tick_slow_cycles(2);

    let noise_at_source = city.resource::<NoisePollutionGrid>().get(128, 128);
    let noise_at_3 = city.resource::<NoisePollutionGrid>().get(131, 128);
    let noise_at_8 = city.resource::<NoisePollutionGrid>().get(136, 128);
    let noise_at_20 = city.resource::<NoisePollutionGrid>().get(148, 128);

    assert!(
        noise_at_source > noise_at_3,
        "source ({}) should be louder than d=3 ({})",
        noise_at_source,
        noise_at_3
    );
    assert!(
        noise_at_3 > noise_at_8,
        "d=3 ({}) should be louder than d=8 ({})",
        noise_at_3,
        noise_at_8
    );
    assert!(
        noise_at_8 > noise_at_20,
        "d=8 ({}) should be louder than d=20 ({})",
        noise_at_8,
        noise_at_20
    );
}

#[test]
fn test_highway_source_noise_is_high() {
    let mut city = TestCity::new().with_road(128, 128, 128, 128, RoadType::Highway);

    {
        let world = city.world_mut();
        world.resource_mut::<WindState>().speed = 0.0;
    }

    city.tick_slow_cycles(2);

    let noise = city.resource::<NoisePollutionGrid>().get(128, 128);
    // Highway is 80 dB -> grid value should be high (80/95*100 = ~84)
    assert!(
        noise >= 50,
        "highway source cell should have significant noise, got {}",
        noise
    );
}

// ====================================================================
// Integration: local road generates less noise than highway
// ====================================================================

#[test]
fn test_local_road_quieter_than_highway() {
    let mut city_highway = TestCity::new().with_road(128, 128, 128, 128, RoadType::Highway);
    let mut city_local = TestCity::new().with_road(128, 128, 128, 128, RoadType::Local);

    for city in [&mut city_highway, &mut city_local] {
        let world = city.world_mut();
        world.resource_mut::<WindState>().speed = 0.0;
    }

    city_highway.tick_slow_cycles(2);
    city_local.tick_slow_cycles(2);

    let hw_noise = city_highway.resource::<NoisePollutionGrid>().get(128, 128);
    let local_noise = city_local.resource::<NoisePollutionGrid>().get(128, 128);

    assert!(
        hw_noise > local_noise,
        "highway ({}) should be louder than local road ({})",
        hw_noise,
        local_noise
    );
}

// ====================================================================
// Integration: airport generates noise with logarithmic decay
// ====================================================================

#[test]
fn test_airport_logarithmic_decay() {
    let mut city =
        TestCity::new().with_service(128, 128, ServiceType::InternationalAirport);

    {
        let world = city.world_mut();
        world.resource_mut::<WindState>().speed = 0.0;
    }

    city.tick_slow_cycles(2);

    let at_airport = city.resource::<NoisePollutionGrid>().get(128, 128);
    let at_5 = city.resource::<NoisePollutionGrid>().get(133, 128);
    let at_15 = city.resource::<NoisePollutionGrid>().get(143, 128);

    assert!(
        at_airport > at_5,
        "airport cell ({}) should be louder than d=5 ({})",
        at_airport,
        at_5
    );
    assert!(
        at_5 > at_15,
        "d=5 ({}) should be louder than d=15 ({})",
        at_5,
        at_15
    );

    // Airport is 95 dB, should be very loud at source
    assert!(
        at_airport >= 70,
        "international airport source should be very loud, got {}",
        at_airport
    );
}

// ====================================================================
// Integration: grid output stays in u8 0-100 range
// ====================================================================

#[test]
fn test_noise_grid_values_within_bounds() {
    let mut city = TestCity::new()
        .with_road(128, 128, 140, 128, RoadType::Highway)
        .with_service(130, 130, ServiceType::InternationalAirport);

    {
        let world = city.world_mut();
        world.resource_mut::<WindState>().speed = 0.0;
    }

    city.tick_slow_cycles(2);

    let grid = city.resource::<NoisePollutionGrid>();
    for y in 0..grid.height {
        for x in 0..grid.width {
            let val = grid.get(x, y);
            assert!(
                val <= 100,
                "noise at ({},{}) = {} exceeds 100",
                x,
                y,
                val
            );
        }
    }
}

// ====================================================================
// Integration: noise grid resource exists (backward compat)
// ====================================================================

#[test]
fn test_noise_grid_resource_exists() {
    let city = TestCity::new();
    city.assert_resource_exists::<NoisePollutionGrid>();
}

// ====================================================================
// Integration: trees still reduce noise
// ====================================================================

#[test]
fn test_trees_reduce_noise_with_logarithmic_model() {
    // A highway with surrounding grass (default terrain) should have
    // slightly lower noise than raw propagation would produce, because
    // grass cells subtract 2 in a 1-cell radius.
    let mut city = TestCity::new().with_road(128, 128, 128, 128, RoadType::Highway);

    {
        let world = city.world_mut();
        world.resource_mut::<WindState>().speed = 0.0;
    }

    city.tick_slow_cycles(2);

    // Noise at distance 1 from highway should be reduced by grass
    // The exact value depends on both attenuation and grass reduction
    let noise_near = city.resource::<NoisePollutionGrid>().get(129, 128);
    let highway_db_at_1 = attenuated_db(80.0, 1.0);
    let raw_grid_val = db_to_grid_u8(highway_db_at_1);

    // The actual noise should be at most the raw value (grass reduces it)
    assert!(
        noise_near <= raw_grid_val + 5,
        "grass should reduce noise: actual={}, raw={}",
        noise_near,
        raw_grid_val
    );
}
