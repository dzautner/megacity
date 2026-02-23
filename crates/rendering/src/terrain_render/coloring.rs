use bevy::prelude::*;

use simulation::colorblind::ColorblindMode;
use simulation::config::{GRID_HEIGHT, GRID_WIDTH};
use simulation::grid::{CellType, WorldGrid, ZoneType};
use simulation::network_viz::NetworkVizData;
use simulation::weather::Season;

use crate::aqi_colors;
use crate::color_ramps::{self, CIVIDIS, GROUNDWATER_LEVEL, GROUNDWATER_QUALITY, INFERNO, VIRIDIS};
use crate::colorblind_palette;
use crate::overlay::{DualOverlayMode, OverlayMode};

use super::types::OverlayGrids;

pub fn terrain_color(
    cell: &simulation::grid::Cell,
    gx: usize,
    gy: usize,
    season: Season,
    snow_depth: f32,
    cb_mode: ColorblindMode,
) -> Color {
    // Per-cell noise for variation (no two cells look identical)
    let noise = ((gx.wrapping_mul(7919).wrapping_add(gy.wrapping_mul(6271))) % 100) as f32 / 100.0;
    let v = (noise - 0.5) * 0.04; // +/- 2% color variation

    let base_color = if cell.zone != ZoneType::None && cell.cell_type != CellType::Road {
        // Urban ground: light concrete/pavement tones (must contrast with dark road asphalt)
        let zone_kind = match cell.zone {
            ZoneType::ResidentialLow => colorblind_palette::ZoneColorKind::ResidentialLow,
            ZoneType::ResidentialMedium => colorblind_palette::ZoneColorKind::ResidentialMedium,
            ZoneType::ResidentialHigh => colorblind_palette::ZoneColorKind::ResidentialHigh,
            ZoneType::CommercialLow => colorblind_palette::ZoneColorKind::CommercialLow,
            ZoneType::CommercialHigh => colorblind_palette::ZoneColorKind::CommercialHigh,
            ZoneType::Industrial => colorblind_palette::ZoneColorKind::Industrial,
            ZoneType::Office => colorblind_palette::ZoneColorKind::Office,
            ZoneType::MixedUse => colorblind_palette::ZoneColorKind::MixedUse,
            ZoneType::None => unreachable!(),
        };
        let (r, g, b) = colorblind_palette::zone_color(zone_kind, cb_mode);
        Color::srgb(
            (r + v).clamp(0.0, 1.0),
            (g + v * 0.8).clamp(0.0, 1.0),
            (b + v * 0.6).clamp(0.0, 1.0),
        )
    } else {
        match cell.cell_type {
            CellType::Water => {
                let depth = 1.0 - cell.elevation / 0.35;
                // Urban waterways: gray-green, not deep blue
                let r = 0.12 + depth * 0.04 + v * 0.5;
                let g = 0.22 + depth * 0.08 + v * 0.3;
                let b = 0.38 + depth * 0.18 + v * 0.2;
                Color::srgb(r, g, b)
            }
            CellType::Road => {
                // Road cells render as light sidewalk/pavement — the asphalt strip is drawn on top
                let (r, g, b) = if cell.road_type == simulation::grid::RoadType::Path {
                    (0.48, 0.44, 0.36) // Dirt path
                } else {
                    (0.62, 0.60, 0.57) // Light concrete sidewalk (contrasts with dark asphalt)
                };
                Color::srgb(
                    (r + v * 0.3).clamp(0.0, 1.0),
                    (g + v * 0.3).clamp(0.0, 1.0),
                    (b + v * 0.2).clamp(0.0, 1.0),
                )
            }
            CellType::Grass => {
                // Grass color varies by season with per-cell noise variation
                let [sr, sg, sb] = season.grass_color();
                let elev = cell.elevation;
                let patch =
                    ((gx.wrapping_mul(31).wrapping_add(gy.wrapping_mul(47))) % 100) as f32 / 100.0;
                let r = sr + elev * 0.06 + patch * 0.08 + v;
                let g = sg + elev * 0.10 + patch * 0.04 + v * 0.5;
                let b = sb + elev * 0.04 + patch * 0.03 + v * 0.3;
                Color::srgb(r.clamp(0.0, 1.0), g.clamp(0.0, 1.0), b.clamp(0.0, 1.0))
            }
        }
    };

    // Snow overlay: blend toward white based on snow depth.
    // Water cells don't get snow overlay. Full white at 6+ inches.
    if snow_depth > 0.0 && cell.cell_type != CellType::Water {
        let snow_factor = (snow_depth / 6.0).min(1.0);
        // Snow white with slight blue tint and per-cell noise for variation
        let snow_r = 0.92 + v * 0.3;
        let snow_g = 0.94 + v * 0.2;
        let snow_b = 0.98 + v * 0.1;
        let srgba = base_color.to_srgba();
        let r = srgba.red * (1.0 - snow_factor) + snow_r * snow_factor;
        let g = srgba.green * (1.0 - snow_factor) + snow_g * snow_factor;
        let b = srgba.blue * (1.0 - snow_factor) + snow_b * snow_factor;
        Color::srgb(r.clamp(0.0, 1.0), g.clamp(0.0, 1.0), b.clamp(0.0, 1.0))
    } else {
        base_color
    }
}

/// Blend two overlay colors for dual-overlay mode.
/// In Blend mode: linear interpolation based on blend_factor.
/// In Split mode: left half of grid uses primary, right half uses secondary.
pub(super) fn blend_dual_overlays(
    primary: Color,
    secondary: Color,
    gx: usize,
    mode: &DualOverlayMode,
    blend_factor: f32,
) -> Color {
    match mode {
        DualOverlayMode::Blend => {
            let a = primary.to_srgba().to_f32_array();
            let b = secondary.to_srgba().to_f32_array();
            let t = blend_factor.clamp(0.0, 1.0);
            Color::srgb(
                a[0] * (1.0 - t) + b[0] * t,
                a[1] * (1.0 - t) + b[1] * t,
                a[2] * (1.0 - t) + b[2] * t,
            )
        }
        DualOverlayMode::Split => {
            let mid = GRID_WIDTH / 2;
            if gx < mid {
                primary
            } else {
                secondary
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn apply_overlay(
    base: Color,
    cell: &simulation::grid::Cell,
    gx: usize,
    gy: usize,
    _grid: &WorldGrid,
    overlay: &OverlayMode,
    grids: &OverlayGrids,
    cb_mode: ColorblindMode,
    network_viz: &NetworkVizData,
) -> Color {
    match overlay {
        OverlayMode::None => base,
        OverlayMode::Power => {
            if cell.cell_type == CellType::Water {
                return base;
            }
            if let Some(src_color) = network_viz.power_source_color(gx, gy) {
                let tint = Color::srgba(src_color[0], src_color[1], src_color[2], 0.45);
                color_ramps::blend_tint(base, tint)
            } else if cell.has_power {
                let palette = colorblind_palette::power_palette(cb_mode);
                color_ramps::overlay_binary(base, &palette, true)
            } else {
                let palette = colorblind_palette::power_palette(cb_mode);
                color_ramps::overlay_binary(base, &palette, false)
            }
        }
        OverlayMode::Water => {
            if cell.cell_type == CellType::Water {
                return base;
            }
            if let Some(src_color) = network_viz.water_source_color(gx, gy) {
                let tint = Color::srgba(src_color[0], src_color[1], src_color[2], 0.45);
                color_ramps::blend_tint(base, tint)
            } else if cell.has_water {
                let palette = colorblind_palette::water_palette(cb_mode);
                color_ramps::overlay_binary(base, &palette, true)
            } else {
                let palette = colorblind_palette::water_palette(cb_mode);
                color_ramps::overlay_binary(base, &palette, false)
            }
        }
        OverlayMode::Traffic => {
            if cell.cell_type == CellType::Road {
                if let Some(traffic) = grids.traffic {
                    let congestion = traffic.congestion_level(gx, gy);
                    // Inferno: black (no traffic) -> red/orange -> yellow (gridlock)
                    color_ramps::overlay_continuous(&INFERNO, congestion)
                } else {
                    base
                }
            } else {
                color_ramps::darken(base, 0.5)
            }
        }
        OverlayMode::Pollution => {
            if cell.cell_type == CellType::Water {
                return base;
            }
            if let Some(pollution) = grids.pollution {
                let concentration = pollution.get(gx, gy);
                // EPA AQI 6-tier color scheme (POLL-020)
                aqi_colors::aqi_overlay_color(concentration)
            } else {
                color_ramps::darken(base, 0.8)
            }
        }
        OverlayMode::LandValue => {
            if cell.cell_type == CellType::Water {
                return base;
            }
            if let Some(land_value) = grids.land_value {
                let value = land_value.get(gx, gy) as f32 / 255.0;
                // Cividis: dark navy (low) -> yellow (high) -- CVD safe
                color_ramps::overlay_continuous(&CIVIDIS, value)
            } else {
                color_ramps::darken(base, 0.8)
            }
        }
        OverlayMode::Education => {
            if cell.cell_type == CellType::Water {
                return base;
            }
            if let Some(education) = grids.education {
                let level = education.get(gx, gy) as f32 / 3.0;
                // Viridis: purple (uneducated) -> teal -> yellow (highly educated)
                color_ramps::overlay_continuous(&VIRIDIS, level)
            } else {
                color_ramps::darken(base, 0.8)
            }
        }
        OverlayMode::Garbage => {
            if cell.cell_type == CellType::Water {
                return base;
            }
            if let Some(garbage) = grids.garbage {
                let level = (garbage.get(gx, gy) as f32 / 30.0).clamp(0.0, 1.0);
                // Inferno: dark (clean) -> bright (lots of garbage)
                color_ramps::overlay_continuous(&INFERNO, level)
            } else {
                color_ramps::darken(base, 0.8)
            }
        }
        OverlayMode::Noise => {
            if cell.cell_type == CellType::Water {
                return base;
            }
            if let Some(noise) = grids.noise {
                let level = (noise.get(gx, gy) as f32 / 100.0).clamp(0.0, 1.0);
                // Inferno: black (quiet) -> red/orange -> yellow (loud)
                color_ramps::overlay_continuous(&INFERNO, level)
            } else {
                color_ramps::darken(base, 0.8)
            }
        }
        OverlayMode::WaterPollution => {
            if let Some(wp) = grids.water_pollution {
                let level = (wp.get(gx, gy) as f32 / 255.0).clamp(0.0, 1.0);
                if cell.cell_type == CellType::Water {
                    // Viridis reversed: yellow (clean) -> teal -> purple (polluted)
                    // Reverse t so clean water = bright, polluted = dark
                    color_ramps::overlay_continuous(&VIRIDIS, 1.0 - level)
                } else if level > 0.0 {
                    // Land cells near polluted water get a subtle brown tint
                    color_ramps::blend_tint(base, Color::srgba(0.5, 0.35, 0.15, level * 0.4))
                } else {
                    color_ramps::darken(base, 0.7)
                }
            } else {
                color_ramps::darken(base, 0.8)
            }
        }
        OverlayMode::GroundwaterLevel => {
            if cell.cell_type == CellType::Water {
                return base;
            }
            if let Some(gw) = grids.groundwater {
                let level = gw.get(gx, gy);
                let t = level as f32 / 255.0;
                let color = color_ramps::overlay_continuous(&GROUNDWATER_LEVEL, t);
                // Depletion warning: cells with level < 30% (~76) get a pulsing highlight
                if level < 76 {
                    // Blend toward warning orange for depleted cells
                    let warning_intensity = (1.0 - level as f32 / 76.0) * 0.3;
                    color_ramps::blend_tint(color, Color::srgba(1.0, 0.6, 0.0, warning_intensity))
                } else {
                    color
                }
            } else {
                color_ramps::darken(base, 0.8)
            }
        }
        OverlayMode::GroundwaterQuality => {
            if cell.cell_type == CellType::Water {
                return base;
            }
            if let Some(wq) = grids.water_quality {
                let quality = wq.get(gx, gy);
                let t = quality as f32 / 255.0;
                color_ramps::overlay_continuous(&GROUNDWATER_QUALITY, t)
            } else {
                color_ramps::darken(base, 0.8)
            }
        }
        OverlayMode::Wind => {
            // Wind overlay uses gizmo streamlines, no terrain recolor needed.
            // Slightly darken the terrain for contrast with the streamline particles.
            color_ramps::darken(base, 0.7)
        }
    }
}

/// Cheap coastline tint: if a non-water cell borders water (or vice versa),
/// blend its color slightly toward a shore tone. Only checks 4 cardinal neighbors
/// (cell type only, no color recomputation).
pub(super) fn coast_tint(
    grid: &WorldGrid,
    gx: usize,
    gy: usize,
    cell_color: [f32; 4],
    cell_type: CellType,
) -> [f32; 4] {
    // Count how many cardinal neighbors are on the other side of the shore
    let mut water_neighbors = 0u32;
    if gx > 0 && grid.get(gx - 1, gy).cell_type == CellType::Water {
        water_neighbors += 1;
    }
    if gx + 1 < GRID_WIDTH && grid.get(gx + 1, gy).cell_type == CellType::Water {
        water_neighbors += 1;
    }
    if gy > 0 && grid.get(gx, gy - 1).cell_type == CellType::Water {
        water_neighbors += 1;
    }
    if gy + 1 < GRID_HEIGHT && grid.get(gx, gy + 1).cell_type == CellType::Water {
        water_neighbors += 1;
    }

    if cell_type == CellType::Water {
        // Water cell next to land: lighten toward sandy shore
        let land_neighbors = 4 - water_neighbors;
        if land_neighbors == 0 {
            return cell_color;
        }
        let blend = land_neighbors as f32 * 0.15; // up to 0.6 for corner water cells
        let shore: [f32; 4] = [0.35, 0.38, 0.32, 1.0]; // muddy shore
        return [
            cell_color[0] + (shore[0] - cell_color[0]) * blend,
            cell_color[1] + (shore[1] - cell_color[1]) * blend,
            cell_color[2] + (shore[2] - cell_color[2]) * blend,
            cell_color[3],
        ];
    }

    // Land cell next to water: darken/blue-tint slightly
    if water_neighbors == 0 {
        return cell_color;
    }
    let blend = water_neighbors as f32 * 0.12;
    let wet: [f32; 4] = [0.18, 0.28, 0.32, 1.0]; // wet ground
    [
        cell_color[0] + (wet[0] - cell_color[0]) * blend,
        cell_color[1] + (wet[1] - cell_color[1]) * blend,
        cell_color[2] + (wet[2] - cell_color[2]) * blend,
        cell_color[3],
    ]
}

pub fn cell_color(cell: &simulation::grid::Cell) -> Color {
    terrain_color(cell, 0, 0, Season::Spring, 0.0, ColorblindMode::Normal)
}
