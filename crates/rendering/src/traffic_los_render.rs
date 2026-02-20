//! Traffic LOS road color feedback rendering.
//!
//! When the traffic overlay is active, this system tints road segment meshes
//! with a green-to-red color ramp based on their Level of Service grade.
//! Road segments are sampled at their midpoint to determine the dominant LOS
//! for the segment, then the mesh material color is updated accordingly.
//!
//! Color ramp: green (A) -> yellow (C) -> orange (D) -> red (F).

use bevy::prelude::*;

use simulation::config::CELL_SIZE;
use simulation::road_segments::RoadSegmentStore;
use simulation::traffic_los::{LosGrade, TrafficLosGrid};

use crate::overlay::OverlayMode;
use crate::road_render::RoadSegmentMesh;

/// LOS color ramp: green (free flow) -> yellow -> orange -> red (gridlock).
/// Returns an sRGB [r, g, b, a] array for a given LOS grade.
fn los_color(grade: LosGrade) -> Color {
    match grade {
        LosGrade::A => Color::srgb(0.20, 0.72, 0.20), // green
        LosGrade::B => Color::srgb(0.55, 0.78, 0.22), // yellow-green
        LosGrade::C => Color::srgb(0.90, 0.82, 0.15), // yellow
        LosGrade::D => Color::srgb(0.95, 0.55, 0.10), // orange
        LosGrade::E => Color::srgb(0.90, 0.25, 0.10), // red-orange
        LosGrade::F => Color::srgb(0.75, 0.08, 0.08), // deep red
    }
}

/// The default (neutral) road material color when no overlay is active.
const ROAD_DEFAULT_COLOR: Color = Color::WHITE;

/// System that updates road segment mesh materials based on traffic LOS.
///
/// When the traffic overlay is active, each road segment mesh is colored
/// according to its LOS grade (sampled from the grid cells the segment covers).
/// When the overlay is disabled, colors are reset to white (neutral).
pub fn update_road_los_colors(
    overlay: Res<crate::overlay::OverlayState>,
    los_grid: Res<TrafficLosGrid>,
    store: Res<RoadSegmentStore>,
    segments_query: Query<(&RoadSegmentMesh, &MeshMaterial3d<StandardMaterial>)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let is_traffic_overlay = overlay.mode == OverlayMode::Traffic;

    // Only update when overlay state or LOS data changes
    if !overlay.is_changed() && !los_grid.is_changed() {
        return;
    }

    for (seg_mesh, material_handle) in &segments_query {
        let Some(material) = materials.get_mut(&material_handle.0) else {
            continue;
        };

        if !is_traffic_overlay {
            // Reset to default color when overlay is off
            material.base_color = ROAD_DEFAULT_COLOR;
            continue;
        }

        // Find the segment in the store
        let Some(segment) = store.get_segment(seg_mesh.segment_id) else {
            material.base_color = ROAD_DEFAULT_COLOR;
            continue;
        };

        // Determine LOS by sampling the rasterized cells of the segment.
        // Use the worst (highest) LOS grade among all cells for conservative grading.
        let grade = if segment.rasterized_cells.is_empty() {
            // Fallback: sample the midpoint of the curve
            let mid = segment.evaluate(0.5);
            let gx = (mid.x / CELL_SIZE).clamp(0.0, (los_grid.width - 1) as f32) as usize;
            let gy = (mid.y / CELL_SIZE).clamp(0.0, (los_grid.height - 1) as f32) as usize;
            los_grid.get(gx, gy)
        } else {
            // Worst LOS across all rasterized cells
            let mut worst = LosGrade::A;
            for &(cx, cy) in &segment.rasterized_cells {
                if cx < los_grid.width && cy < los_grid.height {
                    let g = los_grid.get(cx, cy);
                    if (g as u8) > (worst as u8) {
                        worst = g;
                    }
                }
            }
            worst
        };

        material.base_color = los_color(grade);
    }
}

pub struct TrafficLosRenderPlugin;

impl Plugin for TrafficLosRenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, update_road_los_colors);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_los_color_green_to_red() {
        let a = los_color(LosGrade::A).to_srgba();
        let f = los_color(LosGrade::F).to_srgba();

        // LOS A should be greenish (G > R)
        assert!(
            a.green > a.red,
            "LOS A should be green: r={} g={}",
            a.red,
            a.green
        );

        // LOS F should be reddish (R > G)
        assert!(
            f.red > f.green,
            "LOS F should be red: r={} g={}",
            f.red,
            f.green
        );
    }

    #[test]
    fn test_los_colors_distinct() {
        let grades = [
            LosGrade::A,
            LosGrade::B,
            LosGrade::C,
            LosGrade::D,
            LosGrade::E,
            LosGrade::F,
        ];

        // Each grade should produce a distinct color
        for i in 0..grades.len() {
            for j in (i + 1)..grades.len() {
                let ci = los_color(grades[i]).to_srgba();
                let cj = los_color(grades[j]).to_srgba();
                let diff = (ci.red - cj.red).abs()
                    + (ci.green - cj.green).abs()
                    + (ci.blue - cj.blue).abs();
                assert!(
                    diff > 0.05,
                    "LOS {:?} and {:?} should have distinct colors",
                    grades[i],
                    grades[j]
                );
            }
        }
    }

    #[test]
    fn test_los_color_monotonic_red() {
        // Red channel should generally increase from A to F
        let a_red = los_color(LosGrade::A).to_srgba().red;
        let f_red = los_color(LosGrade::F).to_srgba().red;
        assert!(
            f_red > a_red,
            "LOS F should have more red than LOS A: A.r={} F.r={}",
            a_red,
            f_red
        );
    }

    #[test]
    fn test_los_color_monotonic_green() {
        // Green channel should generally decrease from A to F
        let a_green = los_color(LosGrade::A).to_srgba().green;
        let f_green = los_color(LosGrade::F).to_srgba().green;
        assert!(
            a_green > f_green,
            "LOS A should have more green than LOS F: A.g={} F.g={}",
            a_green,
            f_green
        );
    }
}
