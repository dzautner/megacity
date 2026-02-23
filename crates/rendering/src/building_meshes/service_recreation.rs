//! Procedural meshes for recreation / park buildings:
//! small & large parks, playgrounds, sports fields, plazas, and stadiums.

use simulation::services::ServiceType;

use super::mesh_data::darken;
use super::MeshData;

/// Populate `m` with geometry for a recreation/park building.
#[allow(clippy::too_many_arguments)]
pub(crate) fn generate_recreation_mesh(
    m: &mut MeshData,
    service_type: ServiceType,
    s: f32,
    scale_x: f32,
    scale_z: f32,
) {
    match service_type {
        ServiceType::SmallPark | ServiceType::LargePark => {
            let color = [0.25, 0.65, 0.25, 1.0];
            let hw = s * 0.45 * scale_x;
            let hd = s * 0.45 * scale_z;
            // Flat green base
            m.add_cuboid(0.0, s * 0.02, 0.0, hw, s * 0.02, hd, color);
            // Paths (lighter strips)
            m.add_cuboid(
                0.0,
                s * 0.03,
                0.0,
                hw * 0.08,
                s * 0.01,
                hd,
                [0.6, 0.55, 0.45, 1.0],
            );
            m.add_cuboid(
                0.0,
                s * 0.03,
                0.0,
                hw,
                s * 0.01,
                hd * 0.08,
                [0.6, 0.55, 0.45, 1.0],
            );
            // Trees (trunk + canopy)
            m.add_cylinder(
                -hw * 0.4,
                s * 0.15,
                -hd * 0.4,
                s * 0.03,
                s * 0.2,
                6,
                [0.45, 0.30, 0.15, 1.0],
            );
            m.add_cylinder(
                -hw * 0.4,
                s * 0.35,
                -hd * 0.4,
                s * 0.12,
                s * 0.15,
                8,
                [0.15, 0.5, 0.12, 1.0],
            );
            m.add_cylinder(
                hw * 0.35,
                s * 0.18,
                hd * 0.3,
                s * 0.04,
                s * 0.25,
                6,
                [0.45, 0.30, 0.15, 1.0],
            );
            m.add_cylinder(
                hw * 0.35,
                s * 0.40,
                hd * 0.3,
                s * 0.15,
                s * 0.18,
                8,
                [0.12, 0.45, 0.10, 1.0],
            );
            // Bench
            m.add_cuboid(
                hw * 0.15,
                s * 0.06,
                hd * 0.5,
                s * 0.08,
                s * 0.02,
                s * 0.03,
                [0.50, 0.35, 0.20, 1.0],
            );
            // Fountain (for large park)
            if matches!(service_type, ServiceType::LargePark) {
                m.add_cylinder(
                    0.0,
                    s * 0.08,
                    0.0,
                    s * 0.10,
                    s * 0.08,
                    8,
                    [0.6, 0.6, 0.65, 1.0],
                );
                m.add_cylinder(
                    0.0,
                    s * 0.15,
                    0.0,
                    s * 0.04,
                    s * 0.10,
                    6,
                    [0.3, 0.5, 0.7, 1.0],
                );
            }
        }
        ServiceType::Playground => {
            let color = [0.25, 0.65, 0.25, 1.0];
            m.add_cuboid(0.0, s * 0.02, 0.0, s * 0.45, s * 0.02, s * 0.45, color);
            // Play structures (small colored cuboids)
            m.add_cuboid(
                -s * 0.15,
                s * 0.12,
                -s * 0.1,
                s * 0.06,
                s * 0.10,
                s * 0.06,
                [0.9, 0.3, 0.2, 1.0],
            );
            m.add_cuboid(
                s * 0.15,
                s * 0.08,
                s * 0.1,
                s * 0.08,
                s * 0.06,
                s * 0.04,
                [0.2, 0.5, 0.9, 1.0],
            );
            m.add_cuboid(
                0.0,
                s * 0.15,
                0.0,
                s * 0.04,
                s * 0.13,
                s * 0.04,
                [0.9, 0.8, 0.2, 1.0],
            );
        }
        ServiceType::SportsField => {
            let color = [0.20, 0.60, 0.20, 1.0];
            m.add_cuboid(0.0, s * 0.02, 0.0, s * 0.45, s * 0.02, s * 0.45, color);
            // Goal posts
            m.add_cuboid(
                -s * 0.40,
                s * 0.10,
                0.0,
                s * 0.02,
                s * 0.10,
                s * 0.02,
                [1.0, 1.0, 1.0, 1.0],
            );
            m.add_cuboid(
                s * 0.40,
                s * 0.10,
                0.0,
                s * 0.02,
                s * 0.10,
                s * 0.02,
                [1.0, 1.0, 1.0, 1.0],
            );
            // Field lines
            m.add_cuboid(
                0.0,
                s * 0.025,
                0.0,
                s * 0.01,
                s * 0.005,
                s * 0.35,
                [1.0, 1.0, 1.0, 0.8],
            );
        }
        ServiceType::Plaza => {
            let color = [0.60, 0.58, 0.52, 1.0];
            m.add_cuboid(0.0, s * 0.02, 0.0, s * 0.45, s * 0.02, s * 0.45, color);
            // Lamp posts
            m.add_cylinder(
                -s * 0.25,
                s * 0.20,
                -s * 0.25,
                s * 0.015,
                s * 0.35,
                4,
                [0.4, 0.4, 0.42, 1.0],
            );
            m.add_cylinder(
                s * 0.25,
                s * 0.20,
                s * 0.25,
                s * 0.015,
                s * 0.35,
                4,
                [0.4, 0.4, 0.42, 1.0],
            );
            // Fountain centerpiece
            m.add_cylinder(
                0.0,
                s * 0.08,
                0.0,
                s * 0.12,
                s * 0.08,
                8,
                [0.55, 0.55, 0.58, 1.0],
            );
            m.add_cylinder(
                0.0,
                s * 0.18,
                0.0,
                s * 0.04,
                s * 0.12,
                6,
                [0.3, 0.5, 0.7, 1.0],
            );
        }
        ServiceType::Stadium => {
            let hw = s * 0.45;
            let hh = s * 0.25;
            let hd = s * 0.45;
            // Field
            m.add_cuboid(
                0.0,
                s * 0.02,
                0.0,
                hw * 0.7,
                s * 0.02,
                hd * 0.7,
                [0.2, 0.6, 0.2, 1.0],
            );
            // Stands (4 sides, stacked rings)
            let stand = [0.55, 0.55, 0.58, 1.0];
            m.add_cuboid(0.0, hh * 0.5, hd, hw, hh * 0.5, s * 0.08, stand);
            m.add_cuboid(
                0.0,
                hh,
                hd * 0.9,
                hw * 0.9,
                hh * 0.3,
                s * 0.06,
                darken(stand, 0.9),
            );
            m.add_cuboid(0.0, hh * 0.5, -hd, hw, hh * 0.5, s * 0.08, stand);
            m.add_cuboid(
                0.0,
                hh,
                -hd * 0.9,
                hw * 0.9,
                hh * 0.3,
                s * 0.06,
                darken(stand, 0.9),
            );
            m.add_cuboid(hw, hh * 0.5, 0.0, s * 0.08, hh * 0.5, hd, stand);
            m.add_cuboid(-hw, hh * 0.5, 0.0, s * 0.08, hh * 0.5, hd, stand);
            // Flag poles at corners
            m.add_cylinder(
                hw,
                hh * 2.0,
                hd,
                s * 0.015,
                s * 0.4,
                4,
                [0.6, 0.6, 0.6, 1.0],
            );
            m.add_cylinder(
                -hw,
                hh * 2.0,
                -hd,
                s * 0.015,
                s * 0.4,
                4,
                [0.6, 0.6, 0.6, 1.0],
            );
        }
        _ => {}
    }
}
