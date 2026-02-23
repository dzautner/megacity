//! Tests for the overlay legend module.

use rendering::overlay::OverlayMode;
use simulation::colorblind::ColorblindMode;

use super::metadata::legend_for_mode;
use super::systems::bevy_color_to_egui;
use super::types::LegendKind;

#[test]
fn legend_returns_none_for_no_overlay() {
    assert!(legend_for_mode(OverlayMode::None, ColorblindMode::Normal).is_none());
}

#[test]
fn legend_returns_some_for_all_active_overlays() {
    let modes = [
        OverlayMode::Power,
        OverlayMode::Water,
        OverlayMode::Traffic,
        OverlayMode::Pollution,
        OverlayMode::LandValue,
        OverlayMode::Education,
        OverlayMode::Garbage,
        OverlayMode::Noise,
        OverlayMode::WaterPollution,
        OverlayMode::GroundwaterLevel,
        OverlayMode::GroundwaterQuality,
        OverlayMode::Wind,
    ];
    for mode in modes {
        let result = legend_for_mode(mode, ColorblindMode::Normal);
        assert!(
            result.is_some(),
            "legend_for_mode should return Some for {:?}",
            mode
        );
        let (name, _) = result.unwrap();
        assert!(
            !name.is_empty(),
            "Legend name should not be empty for {:?}",
            mode
        );
    }
}

#[test]
fn legend_works_with_all_colorblind_modes() {
    for cb_mode in ColorblindMode::ALL {
        // Power and Water are binary and change palette per colorblind mode
        let (name, kind) = legend_for_mode(OverlayMode::Power, cb_mode).unwrap();
        assert_eq!(name, "Power");
        assert!(matches!(kind, LegendKind::Binary { .. }));

        let (name, kind) = legend_for_mode(OverlayMode::Water, cb_mode).unwrap();
        assert_eq!(name, "Water");
        assert!(matches!(kind, LegendKind::Binary { .. }));

        // Continuous overlays should work too
        let (name, kind) = legend_for_mode(OverlayMode::Traffic, cb_mode).unwrap();
        assert_eq!(name, "Traffic");
        assert!(matches!(kind, LegendKind::Continuous { .. }));
    }
}

#[test]
fn wind_overlay_returns_directional_legend() {
    let (name, kind) = legend_for_mode(OverlayMode::Wind, ColorblindMode::Normal).unwrap();
    assert_eq!(name, "Wind");
    assert!(matches!(kind, LegendKind::Directional { .. }));
}

#[test]
fn binary_overlays_have_distinct_on_off_colors() {
    for cb_mode in ColorblindMode::ALL {
        for mode in [OverlayMode::Power, OverlayMode::Water] {
            let (_, kind) = legend_for_mode(mode, cb_mode).unwrap();
            if let LegendKind::Binary {
                on_color,
                off_color,
                ..
            } = kind
            {
                assert_ne!(
                    on_color, off_color,
                    "On/off colors should be distinct for {:?} in {:?} mode",
                    mode, cb_mode
                );
            }
        }
    }
}

#[test]
fn bevy_color_to_egui_produces_valid_output() {
    // Verify the conversion produces non-zero output and preserves
    // relative channel ordering (red > green > blue for an orange color).
    let bevy_color = bevy::prelude::Color::srgb(0.9, 0.5, 0.1);
    let egui_color = bevy_color_to_egui(bevy_color);
    // Red channel should be the highest
    assert!(egui_color.r() > egui_color.g(), "red should exceed green");
    assert!(egui_color.g() > egui_color.b(), "green should exceed blue");
    // All channels should be non-zero for this input
    assert!(egui_color.r() > 0);
    assert!(egui_color.g() > 0);
    assert!(egui_color.b() > 0);
}

#[test]
fn pollution_overlay_returns_tiered_legend() {
    let (name, kind) = legend_for_mode(OverlayMode::Pollution, ColorblindMode::Normal).unwrap();
    assert_eq!(name, "Air Quality (AQI)");
    assert!(
        matches!(kind, LegendKind::Tiered { .. }),
        "Pollution overlay should use Tiered legend"
    );
}

#[test]
fn pollution_tiered_legend_has_six_entries() {
    let (_, kind) = legend_for_mode(OverlayMode::Pollution, ColorblindMode::Normal).unwrap();
    if let LegendKind::Tiered { entries } = kind {
        assert_eq!(entries.len(), 6, "AQI legend should have 6 tiers");
        // Verify all labels are non-empty
        for entry in entries {
            assert!(
                !entry.label.is_empty(),
                "Each tier label should be non-empty"
            );
        }
    } else {
        panic!("Expected Tiered legend kind");
    }
}

#[test]
fn pollution_tiered_legend_has_distinct_colors() {
    let (_, kind) = legend_for_mode(OverlayMode::Pollution, ColorblindMode::Normal).unwrap();
    if let LegendKind::Tiered { entries } = kind {
        for i in 0..entries.len() {
            for j in (i + 1)..entries.len() {
                assert_ne!(
                    entries[i].color, entries[j].color,
                    "Tiers {} and {} should have distinct colors",
                    i, j
                );
            }
        }
    } else {
        panic!("Expected Tiered legend kind");
    }
}
