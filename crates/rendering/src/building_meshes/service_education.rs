//! Procedural meshes for education buildings:
//! elementary/high schools, kindergartens, universities, and libraries.

use simulation::services::ServiceType;

use super::mesh_data::{darken, lighten};
use super::MeshData;

/// Populate `m` with geometry for an education building.
pub(crate) fn generate_education_mesh(m: &mut MeshData, service_type: ServiceType, s: f32) {
    match service_type {
        ServiceType::ElementarySchool | ServiceType::HighSchool | ServiceType::Kindergarten => {
            let color = [0.94, 0.78, 0.63, 1.0];
            let hw = s * 0.4;
            let hh = s * 0.25;
            let hd = s * 0.4;
            // L-shaped building: main wing
            m.add_cuboid(0.0, hh, 0.0, hw, hh, hd * 0.6, color);
            // Side wing
            m.add_cuboid(
                hw * 0.5,
                hh,
                hd * 0.3,
                hw * 0.5,
                hh,
                hd * 0.4,
                darken(color, 0.95),
            );
            // Peaked roof on main wing
            m.add_roof_prism(
                0.0,
                hh * 2.0,
                0.0,
                hw,
                hh * 0.4,
                hd * 0.6,
                darken(color, 0.85),
            );
            // Flagpole
            m.add_cylinder(
                hw * 0.8,
                hh * 2.0 + s * 0.15,
                hd * 0.8,
                s * 0.015,
                s * 0.4,
                4,
                [0.6, 0.6, 0.6, 1.0],
            );
            // Windows
            let spacing = (hw * 2.0) / 5.0;
            for i in 1..5 {
                let wx = -hw + i as f32 * spacing;
                m.add_cuboid(
                    wx,
                    hh,
                    hd * 0.6 - 0.05,
                    s * 0.035,
                    s * 0.04,
                    0.08,
                    [0.2, 0.22, 0.3, 1.0],
                );
            }
        }
        ServiceType::University => {
            let color = [0.47, 0.47, 0.72, 1.0];
            let hw = s * 0.42;
            let hh = s * 0.45;
            let hd = s * 0.42;
            m.add_cuboid(0.0, hh, 0.0, hw, hh, hd, color);
            // Clock tower
            m.add_cuboid(
                0.0,
                hh * 2.5,
                0.0,
                s * 0.08,
                hh * 0.8,
                s * 0.08,
                darken(color, 0.85),
            );
            // Dome on top
            m.add_cylinder(
                0.0,
                hh * 2.0 + hh * 1.6 + s * 0.06,
                0.0,
                s * 0.12,
                s * 0.10,
                8,
                lighten(color, 1.2),
            );
            // Entrance columns
            for i in 0..4 {
                let x = -hw * 0.4 + i as f32 * hw * 0.27;
                m.add_cuboid(
                    x,
                    hh,
                    hd + s * 0.03,
                    s * 0.025,
                    hh,
                    s * 0.025,
                    [0.75, 0.75, 0.78, 1.0],
                );
            }
        }
        ServiceType::Library => {
            let color = [0.85, 0.70, 0.50, 1.0];
            let hw = s * 0.38;
            let hh = s * 0.30;
            let hd = s * 0.38;
            m.add_cuboid(0.0, hh, 0.0, hw, hh, hd, color);
            // Classical columns at entrance
            for i in 0..3 {
                let x = -hw * 0.3 + i as f32 * hw * 0.3;
                m.add_cuboid(
                    x,
                    hh,
                    hd + s * 0.04,
                    s * 0.03,
                    hh,
                    s * 0.03,
                    [0.8, 0.78, 0.72, 1.0],
                );
            }
            // Wide steps
            m.add_cuboid(
                0.0,
                hh * 0.15,
                hd + s * 0.08,
                hw * 0.6,
                hh * 0.15,
                s * 0.06,
                darken(color, 0.8),
            );
        }
        _ => {}
    }
}
