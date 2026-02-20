use bevy_egui::egui;

use rendering::overlay::{OverlayMode, OverlayState};
use simulation::config::{GRID_HEIGHT, GRID_WIDTH};
use simulation::grid::{CellType, WorldGrid, ZoneType};

pub(crate) const MINIMAP_SIZE: usize = 128;
pub(crate) const SAMPLE_STEP: usize = 2; // Sample every Nth cell

pub(crate) fn build_minimap_pixels(grid: &WorldGrid, overlay: &OverlayState) -> Vec<egui::Color32> {
    let mut pixels = vec![egui::Color32::BLACK; MINIMAP_SIZE * MINIMAP_SIZE];

    for my in 0..MINIMAP_SIZE {
        for mx in 0..MINIMAP_SIZE {
            let gx = (mx * SAMPLE_STEP).min(GRID_WIDTH - 1);
            let gy_raw = (MINIMAP_SIZE - 1 - my) * SAMPLE_STEP; // Flip Y for screen coords
            let gy = gy_raw.min(GRID_HEIGHT - 1);
            let cell = grid.get(gx, gy);

            let color = match overlay.mode {
                OverlayMode::Power if cell.cell_type != CellType::Water => {
                    if cell.has_power {
                        egui::Color32::from_rgb(200, 200, 50)
                    } else {
                        egui::Color32::from_rgb(150, 30, 30)
                    }
                }
                OverlayMode::Water if cell.cell_type != CellType::Water => {
                    if cell.has_water {
                        egui::Color32::from_rgb(50, 120, 200)
                    } else {
                        egui::Color32::from_rgb(150, 30, 30)
                    }
                }
                _ => {
                    // Normal colors
                    if cell.building_id.is_some() {
                        if cell.zone.is_residential() {
                            egui::Color32::from_rgb(80, 180, 80)
                        } else if cell.zone.is_commercial() {
                            egui::Color32::from_rgb(60, 100, 200)
                        } else if cell.zone == ZoneType::Industrial {
                            egui::Color32::from_rgb(200, 170, 40)
                        } else if cell.zone == ZoneType::Office {
                            egui::Color32::from_rgb(150, 120, 210)
                        } else if cell.zone.is_mixed_use() {
                            egui::Color32::from_rgb(160, 140, 80)
                        } else {
                            egui::Color32::from_rgb(140, 140, 140)
                        }
                    } else if cell.zone != ZoneType::None {
                        if cell.zone.is_residential() {
                            egui::Color32::from_rgb(60, 120, 60)
                        } else if cell.zone.is_commercial() {
                            egui::Color32::from_rgb(40, 60, 140)
                        } else if cell.zone == ZoneType::Industrial {
                            egui::Color32::from_rgb(140, 120, 30)
                        } else if cell.zone == ZoneType::Office {
                            egui::Color32::from_rgb(100, 80, 160)
                        } else if cell.zone.is_mixed_use() {
                            egui::Color32::from_rgb(120, 100, 50)
                        } else {
                            egui::Color32::from_rgb(80, 80, 80)
                        }
                    } else {
                        match cell.cell_type {
                            CellType::Water => egui::Color32::from_rgb(20, 60, 160),
                            CellType::Road => egui::Color32::from_rgb(80, 80, 80),
                            CellType::Grass => {
                                let g = (80.0 + cell.elevation * 100.0) as u8;
                                egui::Color32::from_rgb(30, g, 25)
                            }
                        }
                    }
                }
            };

            pixels[my * MINIMAP_SIZE + mx] = color;
        }
    }

    pixels
}
