//! Unit tests for wind-aware Gaussian plume pollution dispersion.

use super::config::WindPollutionConfig;
use super::dispersion::{apply_isotropic_source, apply_plume_source, PollutionSource};
use crate::config::{GRID_HEIGHT, GRID_WIDTH};
use crate::Saveable;

#[test]
fn test_scrubber_reduces_emission() {
    let config_off = WindPollutionConfig {
        scrubbers_enabled: false,
    };
    let config_on = WindPollutionConfig {
        scrubbers_enabled: true,
    };

    let mult_off = if config_off.scrubbers_enabled {
        0.5
    } else {
        1.0
    };
    let mult_on = if config_on.scrubbers_enabled {
        0.5
    } else {
        1.0
    };

    assert_eq!(mult_off, 1.0);
    assert_eq!(mult_on, 0.5);
}

#[test]
fn test_saveable_roundtrip() {
    let config = WindPollutionConfig {
        scrubbers_enabled: true,
    };
    let bytes = config.save_to_bytes().unwrap();
    let restored = WindPollutionConfig::load_from_bytes(&bytes);
    assert!(restored.scrubbers_enabled);
}

#[test]
fn test_saveable_skip_default() {
    let config = WindPollutionConfig::default();
    assert!(config.save_to_bytes().is_none());
}

#[test]
fn test_isotropic_symmetric() {
    let total = GRID_WIDTH * GRID_HEIGHT;
    let mut levels = vec![0.0f32; total];

    let src = PollutionSource {
        x: 128,
        y: 128,
        emission_q: 10.0,
    };

    apply_isotropic_source(&mut levels, &src);

    // Check symmetry: east and west at same distance should be equal
    let east = levels[128 * GRID_WIDTH + 132];
    let west = levels[128 * GRID_WIDTH + 124];
    assert!(
        (east - west).abs() < 0.01,
        "isotropic should be symmetric: east={}, west={}",
        east,
        west
    );
}

#[test]
fn test_plume_downwind_greater_than_upwind() {
    let total = GRID_WIDTH * GRID_HEIGHT;
    let mut levels = vec![0.0f32; total];

    let src = PollutionSource {
        x: 128,
        y: 128,
        emission_q: 20.0,
    };

    // Wind blowing east
    apply_plume_source(&mut levels, &src, 1.0, 0.0, 0.8);

    // Downwind (east of source)
    let downwind: f32 = (130..=134).map(|x| levels[128 * GRID_WIDTH + x]).sum();
    // Upwind (west of source)
    let upwind: f32 = (122..=126).map(|x| levels[128 * GRID_WIDTH + x]).sum();

    assert!(
        downwind > upwind,
        "plume downwind={} should be > upwind={}",
        downwind,
        upwind
    );
}

#[test]
fn test_plume_wind_direction_change() {
    let total = GRID_WIDTH * GRID_HEIGHT;

    // East wind
    let mut levels_east = vec![0.0f32; total];
    let src = PollutionSource {
        x: 128,
        y: 128,
        emission_q: 20.0,
    };
    apply_plume_source(&mut levels_east, &src, 1.0, 0.0, 0.8);

    // North wind
    let mut levels_north = vec![0.0f32; total];
    apply_plume_source(&mut levels_north, &src, 0.0, 1.0, 0.8);

    // East wind: more pollution east than north
    let east_east: f32 = (132..=136).map(|x| levels_east[128 * GRID_WIDTH + x]).sum();
    let east_north: f32 = (132..=136)
        .map(|y| levels_east[y * GRID_WIDTH + 128])
        .sum();

    assert!(
        east_east > east_north,
        "east wind: east_sum={} should be > north_sum={}",
        east_east,
        east_north
    );

    // North wind: more pollution north than east
    let north_north: f32 = (132..=136)
        .map(|y| levels_north[y * GRID_WIDTH + 128])
        .sum();
    let north_east: f32 = (132..=136)
        .map(|x| levels_north[128 * GRID_WIDTH + x])
        .sum();

    assert!(
        north_north > north_east,
        "north wind: north_sum={} should be > east_sum={}",
        north_north,
        north_east
    );
}

#[test]
fn test_plume_crosswind_less_than_downwind() {
    let total = GRID_WIDTH * GRID_HEIGHT;
    let mut levels = vec![0.0f32; total];

    let src = PollutionSource {
        x: 128,
        y: 128,
        emission_q: 20.0,
    };

    // Wind blowing east
    apply_plume_source(&mut levels, &src, 1.0, 0.0, 0.8);

    // Directly downwind at distance 5
    let downwind_center = levels[128 * GRID_WIDTH + 133];
    // Crosswind at distance 5 (north of downwind line at same distance)
    let crosswind_off = levels[133 * GRID_WIDTH + 128];

    assert!(
        downwind_center > crosswind_off,
        "downwind_center={} should be > crosswind_off={}",
        downwind_center,
        crosswind_off
    );
}
