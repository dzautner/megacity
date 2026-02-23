//! Overlay-to-legend metadata mapping.
//!
//! Maps each `OverlayMode` to its legend representation (continuous ramp,
//! binary swatches, tiered bands, or directional label).

use rendering::color_ramps::{CIVIDIS, GROUNDWATER_LEVEL, GROUNDWATER_QUALITY, INFERNO, VIRIDIS};
use rendering::colorblind_palette;
use rendering::overlay::OverlayMode;

use super::systems::bevy_color_to_egui;
use super::types::{LegendKind, TieredEntry};

/// Pre-computed AQI legend entries (static to avoid per-frame allocation).
/// Colors are the EPA standard AQI colors converted to egui Color32.
static AQI_LEGEND_ENTRIES: [TieredEntry; 6] = [
    TieredEntry {
        color: bevy_egui::egui::Color32::from_rgb(0, 148, 23),
        label: "0-50 Good",
    },
    TieredEntry {
        color: bevy_egui::egui::Color32::from_rgb(255, 222, 0),
        label: "51-100 Moderate",
    },
    TieredEntry {
        color: bevy_egui::egui::Color32::from_rgb(255, 125, 0),
        label: "101-150 Sensitive",
    },
    TieredEntry {
        color: bevy_egui::egui::Color32::from_rgb(255, 0, 0),
        label: "151-200 Unhealthy",
    },
    TieredEntry {
        color: bevy_egui::egui::Color32::from_rgb(153, 51, 153),
        label: "201-300 Very Unhealthy",
    },
    TieredEntry {
        color: bevy_egui::egui::Color32::from_rgb(128, 0, 33),
        label: "301-500 Hazardous",
    },
];

pub(crate) fn legend_for_mode(
    mode: OverlayMode,
    cb_mode: simulation::colorblind::ColorblindMode,
) -> Option<(&'static str, LegendKind)> {
    match mode {
        OverlayMode::None => None,
        OverlayMode::Power => {
            let palette = colorblind_palette::power_palette(cb_mode);
            let on = bevy_color_to_egui(palette.on);
            let off = bevy_color_to_egui(palette.off);
            Some((
                "Power",
                LegendKind::Binary {
                    on_color: on,
                    off_color: off,
                    on_label: "Powered",
                    off_label: "No Power",
                },
            ))
        }
        OverlayMode::Water => {
            let palette = colorblind_palette::water_palette(cb_mode);
            let on = bevy_color_to_egui(palette.on);
            let off = bevy_color_to_egui(palette.off);
            Some((
                "Water",
                LegendKind::Binary {
                    on_color: on,
                    off_color: off,
                    on_label: "Connected",
                    off_label: "No Water",
                },
            ))
        }
        OverlayMode::Traffic => Some((
            "Traffic",
            LegendKind::Continuous {
                ramp: &INFERNO,
                min_label: "Free Flow",
                max_label: "Gridlock",
            },
        )),
        OverlayMode::Pollution => Some((
            "Air Quality (AQI)",
            LegendKind::Tiered {
                entries: &AQI_LEGEND_ENTRIES,
            },
        )),
        OverlayMode::LandValue => Some((
            "Land Value",
            LegendKind::Continuous {
                ramp: &CIVIDIS,
                min_label: "Low",
                max_label: "High",
            },
        )),
        OverlayMode::Education => Some((
            "Education",
            LegendKind::Continuous {
                ramp: &VIRIDIS,
                min_label: "None",
                max_label: "University",
            },
        )),
        OverlayMode::Garbage => Some((
            "Garbage",
            LegendKind::Continuous {
                ramp: &INFERNO,
                min_label: "Clean",
                max_label: "Full",
            },
        )),
        OverlayMode::Noise => Some((
            "Noise",
            LegendKind::Continuous {
                ramp: &INFERNO,
                min_label: "Quiet",
                max_label: "Loud",
            },
        )),
        OverlayMode::WaterPollution => Some((
            "Water Pollution",
            LegendKind::Continuous {
                ramp: &VIRIDIS,
                min_label: "Polluted",
                max_label: "Clean",
            },
        )),
        OverlayMode::GroundwaterLevel => Some((
            "Groundwater Level",
            LegendKind::Continuous {
                ramp: &GROUNDWATER_LEVEL,
                min_label: "Dry",
                max_label: "Saturated",
            },
        )),
        OverlayMode::GroundwaterQuality => Some((
            "Groundwater Quality",
            LegendKind::Continuous {
                ramp: &GROUNDWATER_QUALITY,
                min_label: "Contaminated",
                max_label: "Clean",
            },
        )),
        OverlayMode::Wind => Some((
            "Wind",
            LegendKind::Directional {
                description: "Arrows show wind direction and speed",
            },
        )),
    }
}
