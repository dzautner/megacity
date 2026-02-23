//! Procedural meshes for civic / landmark buildings:
//! cell towers, data centers, transfer stations, city halls, cathedrals,
//! museums, cemeteries, and crematoriums.

use simulation::services::ServiceType;

use super::mesh_data::darken;
use super::MeshData;

/// Populate `m` with geometry for a civic/landmark building.
#[allow(clippy::too_many_arguments)]
pub(crate) fn generate_civic_mesh(
    m: &mut MeshData,
    service_type: ServiceType,
    s: f32,
    scale_x: f32,
    scale_z: f32,
) {
    match service_type {
        ServiceType::CellTower => {
            let color = [0.6, 0.6, 0.6, 1.0];
            m.add_cylinder(0.0, s * 0.5, 0.0, s * 0.03, s * 1.0, 6, color);
            m.add_cuboid(
                0.0,
                s * 0.85,
                0.0,
                s * 0.15,
                s * 0.01,
                s * 0.01,
                [0.5, 0.5, 0.55, 1.0],
            );
            m.add_cuboid(
                0.0,
                s * 0.75,
                0.0,
                s * 0.01,
                s * 0.01,
                s * 0.12,
                [0.5, 0.5, 0.55, 1.0],
            );
        }
        ServiceType::DataCenter => {
            let color = [0.35, 0.40, 0.50, 1.0];
            let hw = s * 0.4 * scale_x;
            let hh = s * 0.25;
            let hd = s * 0.4 * scale_z;
            m.add_cuboid(0.0, hh, 0.0, hw, hh, hd, color);
            m.add_cuboid(
                hw * 0.5,
                hh * 2.0 + s * 0.05,
                hd * 0.5,
                s * 0.08,
                s * 0.05,
                s * 0.08,
                [0.4, 0.45, 0.5, 1.0],
            );
            m.add_cuboid(
                -hw * 0.5,
                hh * 2.0 + s * 0.05,
                -hd * 0.5,
                s * 0.08,
                s * 0.05,
                s * 0.08,
                [0.4, 0.45, 0.5, 1.0],
            );
        }
        ServiceType::TransferStation => {
            let color = [0.55, 0.50, 0.40, 1.0];
            m.add_cuboid(0.0, s * 0.2, 0.0, s * 0.4, s * 0.2, s * 0.35, color);
        }
        ServiceType::CityHall => {
            let color = [0.85, 0.80, 0.65, 1.0];
            let hw = s * 0.42;
            let hh = s * 0.40;
            let hd = s * 0.42;
            m.add_cuboid(0.0, hh, 0.0, hw, hh, hd, color);
            // Columns at entrance
            for i in 0..5 {
                let x = -hw * 0.5 + i as f32 * hw * 0.25;
                m.add_cuboid(
                    x,
                    hh,
                    hd + s * 0.04,
                    s * 0.025,
                    hh,
                    s * 0.025,
                    [0.8, 0.78, 0.72, 1.0],
                );
            }
            // Dome/cupola
            m.add_cylinder(
                0.0,
                hh * 2.0 + s * 0.1,
                0.0,
                s * 0.12,
                s * 0.15,
                8,
                darken(color, 0.85),
            );
            // Flag
            m.add_cylinder(
                0.0,
                hh * 2.0 + s * 0.25,
                0.0,
                s * 0.015,
                s * 0.3,
                4,
                [0.6, 0.6, 0.6, 1.0],
            );
            // Steps
            m.add_cuboid(
                0.0,
                hh * 0.1,
                hd + s * 0.08,
                hw * 0.7,
                hh * 0.1,
                s * 0.06,
                darken(color, 0.8),
            );
        }
        ServiceType::Cathedral => {
            let color = [0.78, 0.72, 0.60, 1.0];
            let hw = s * 0.40;
            let hh = s * 0.50;
            let hd = s * 0.42;
            m.add_cuboid(0.0, hh, 0.0, hw, hh, hd, color);
            // Tall peaked roof
            m.add_roof_prism(
                0.0,
                hh * 2.0,
                0.0,
                hw * 1.02,
                hh * 0.6,
                hd * 1.02,
                darken(color, 0.75),
            );
            // Bell tower
            m.add_cuboid(
                hw * 0.35,
                hh * 2.8,
                -hd * 0.3,
                s * 0.08,
                hh * 0.8,
                s * 0.08,
                darken(color, 0.85),
            );
            m.add_roof_prism(
                hw * 0.35,
                hh * 2.8 + hh * 0.8,
                -hd * 0.3,
                s * 0.10,
                s * 0.12,
                s * 0.10,
                darken(color, 0.7),
            );
            // Rose window
            m.add_cuboid(
                0.0,
                hh * 1.5,
                hd + 0.03,
                s * 0.08,
                s * 0.08,
                s * 0.01,
                [0.5, 0.3, 0.6, 1.0],
            );
            // Entrance arch
            m.add_cuboid(
                0.0,
                hh * 0.35,
                hd + 0.05,
                hw * 0.2,
                hh * 0.35,
                0.05,
                darken(color, 0.4),
            );
        }
        ServiceType::Museum => {
            let color = [0.88, 0.85, 0.78, 1.0];
            let hw = s * 0.42;
            let hh = s * 0.35;
            let hd = s * 0.42;
            m.add_cuboid(0.0, hh, 0.0, hw, hh, hd, color);
            // Classical columns
            for i in 0..6 {
                let x = -hw * 0.6 + i as f32 * hw * 0.24;
                m.add_cuboid(
                    x,
                    hh,
                    hd + s * 0.05,
                    s * 0.03,
                    hh,
                    s * 0.03,
                    [0.82, 0.80, 0.75, 1.0],
                );
            }
            // Wide steps
            m.add_cuboid(
                0.0,
                hh * 0.1,
                hd + s * 0.10,
                hw * 0.8,
                hh * 0.1,
                s * 0.08,
                darken(color, 0.85),
            );
            m.add_cuboid(
                0.0,
                hh * 0.2,
                hd + s * 0.06,
                hw * 0.8,
                hh * 0.1,
                s * 0.04,
                darken(color, 0.82),
            );
            // Pediment (triangle above columns)
            m.add_roof_prism(
                0.0,
                hh * 2.0,
                hd + s * 0.02,
                hw * 0.65,
                hh * 0.3,
                s * 0.03,
                darken(color, 0.9),
            );
        }
        ServiceType::Cemetery => {
            let color = [0.3, 0.35, 0.3, 1.0];
            let hw = s * 0.45;
            let hd = s * 0.45;
            // Flat ground base
            m.add_cuboid(0.0, s * 0.02, 0.0, hw, s * 0.02, hd, color);
            // Headstones scattered across the cemetery
            let stone_color = [0.6, 0.6, 0.6, 1.0];
            for row_z in [-0.5_f32, 0.0, 0.5] {
                for col_x in [-0.5_f32, -0.2, 0.1, 0.4] {
                    m.add_cuboid(
                        hw * col_x,
                        s * 0.08,
                        hd * row_z,
                        s * 0.03,
                        s * 0.06,
                        s * 0.015,
                        stone_color,
                    );
                }
            }
            // Tree (cypress-style, tall and thin)
            m.add_cylinder(
                hw * 0.35,
                s * 0.15,
                hd * 0.35,
                s * 0.02,
                s * 0.2,
                6,
                [0.35, 0.25, 0.15, 1.0],
            );
            m.add_cylinder(
                hw * 0.35,
                s * 0.35,
                hd * 0.35,
                s * 0.06,
                s * 0.25,
                6,
                [0.15, 0.35, 0.12, 1.0],
            );
            // Gate/fence at entrance
            m.add_cuboid(
                0.0,
                s * 0.10,
                hd,
                hw * 0.15,
                s * 0.10,
                s * 0.02,
                darken(color, 0.6),
            );
        }
        ServiceType::Crematorium => {
            let color = [0.4, 0.25, 0.25, 1.0];
            let hw = s * 0.38;
            let hh = s * 0.30;
            let hd = s * 0.38;
            // Main building
            m.add_cuboid(0.0, hh, 0.0, hw, hh, hd, color);
            // Tall chimney/smokestack
            m.add_cylinder(
                hw * 0.6,
                hh * 2.5,
                -hd * 0.4,
                s * 0.05,
                hh * 2.0,
                8,
                [0.5, 0.45, 0.45, 1.0],
            );
            // Entrance
            m.add_cuboid(
                0.0,
                hh * 0.35,
                hd + 0.05,
                hw * 0.2,
                hh * 0.35,
                0.05,
                darken(color, 0.4),
            );
            // Peaked roof
            m.add_roof_prism(
                0.0,
                hh * 2.0,
                0.0,
                hw * 1.02,
                hh * 0.3,
                hd * 1.02,
                darken(color, 0.8),
            );
        }
        _ => {}
    }
}
