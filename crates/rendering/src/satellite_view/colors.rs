//! Color helpers for satellite view rendering.

use simulation::grid::{CellType, ZoneType};
use simulation::weather::Weather;

/// Terrain color for satellite view (simplified, no per-cell noise).
pub(crate) fn satellite_terrain_color(cell: &simulation::grid::Cell, weather: &Weather) -> [u8; 4] {
    let (r, g, b) = if cell.zone != ZoneType::None && cell.cell_type != CellType::Road {
        match cell.zone {
            ZoneType::ResidentialLow => (0.52, 0.56, 0.46),
            ZoneType::ResidentialMedium => (0.57, 0.58, 0.51),
            ZoneType::ResidentialHigh => (0.62, 0.60, 0.57),
            ZoneType::CommercialLow => (0.58, 0.57, 0.54),
            ZoneType::CommercialHigh => (0.60, 0.58, 0.55),
            ZoneType::Industrial => (0.55, 0.52, 0.47),
            ZoneType::Office => (0.64, 0.62, 0.58),
            ZoneType::MixedUse => (0.60, 0.58, 0.52),
            ZoneType::None => unreachable!(),
        }
    } else {
        match cell.cell_type {
            CellType::Water => {
                let depth = (1.0 - cell.elevation / 0.35).clamp(0.0, 1.0);
                (
                    0.12 + depth * 0.04,
                    0.22 + depth * 0.08,
                    0.38 + depth * 0.18,
                )
            }
            CellType::Road => (0.35, 0.35, 0.38),
            CellType::Grass => {
                let [sr, sg, sb] = weather.season.grass_color();
                (sr, sg, sb)
            }
        }
    };

    to_rgba8(r, g, b)
}

/// Building color for satellite view based on zone type and level.
pub(crate) fn zone_satellite_color(zone: ZoneType, level: u8) -> [u8; 4] {
    let level_factor = 1.0 - (level as f32 - 1.0) * 0.08;
    let (r, g, b) = match zone {
        ZoneType::ResidentialLow => (0.70, 0.75, 0.65),
        ZoneType::ResidentialMedium => (0.65, 0.68, 0.55),
        ZoneType::ResidentialHigh => (0.72, 0.70, 0.68),
        ZoneType::CommercialLow => (0.65, 0.60, 0.70),
        ZoneType::CommercialHigh => (0.60, 0.55, 0.68),
        ZoneType::Industrial => (0.68, 0.62, 0.50),
        ZoneType::Office => (0.62, 0.65, 0.72),
        ZoneType::MixedUse => (0.67, 0.62, 0.65),
        ZoneType::None => (0.5, 0.5, 0.5),
    };
    to_rgba8(r * level_factor, g * level_factor, b * level_factor)
}

/// Road line color for satellite view.
pub(crate) fn road_satellite_color(road_type: simulation::grid::RoadType) -> [u8; 4] {
    use simulation::grid::RoadType;
    match road_type {
        RoadType::Path => [160, 145, 120, 255],
        RoadType::OneWay => [90, 90, 100, 255],
        RoadType::Local => [80, 80, 90, 255],
        RoadType::Avenue => [70, 70, 80, 255],
        RoadType::Boulevard => [60, 60, 75, 255],
        RoadType::Highway => [55, 55, 70, 255],
    }
}

/// Road line width in texture pixels for satellite view.
pub(crate) fn road_satellite_width(road_type: simulation::grid::RoadType) -> f32 {
    use simulation::grid::RoadType;
    match road_type {
        RoadType::Path => 0.8,
        RoadType::OneWay => 1.0,
        RoadType::Local => 1.2,
        RoadType::Avenue => 1.8,
        RoadType::Boulevard => 2.4,
        RoadType::Highway => 3.0,
    }
}

/// Convert floating-point RGB (0.0-1.0) to `[u8; 4]` RGBA with full alpha.
pub(crate) fn to_rgba8(r: f32, g: f32, b: f32) -> [u8; 4] {
    [
        (r * 255.0).clamp(0.0, 255.0) as u8,
        (g * 255.0).clamp(0.0, 255.0) as u8,
        (b * 255.0).clamp(0.0, 255.0) as u8,
        255,
    ]
}
