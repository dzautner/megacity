//! Procedural meshes for emergency-service buildings:
//! fire stations, police stations, prisons, and hospitals.

use simulation::services::ServiceType;

use super::mesh_data::darken;
use super::MeshData;

/// Populate `m` with geometry for an emergency-service building.
#[allow(clippy::too_many_arguments)]
pub(crate) fn generate_emergency_mesh(
    m: &mut MeshData,
    service_type: ServiceType,
    s: f32,
    scale_x: f32,
    scale_z: f32,
) {
    match service_type {
        ServiceType::FireStation | ServiceType::FireHouse | ServiceType::FireHQ => {
            let color = [1.0, 0.50, 0.50, 1.0];
            let hw = s * 0.4 * scale_x;
            let hh = s * 0.3;
            let hd = s * 0.4 * scale_z;
            m.add_cuboid(0.0, hh, 0.0, hw, hh, hd, color);
            // Garage doors
            m.add_cuboid(
                -hw * 0.3,
                hh * 0.45,
                hd + 0.05,
                hw * 0.25,
                hh * 0.45,
                0.05,
                darken(color, 0.4),
            );
            m.add_cuboid(
                hw * 0.3,
                hh * 0.45,
                hd + 0.05,
                hw * 0.25,
                hh * 0.45,
                0.05,
                darken(color, 0.4),
            );
            // Tower (hose drying tower)
            m.add_cuboid(
                hw * 0.7,
                hh * 2.5,
                hd * 0.5,
                s * 0.08,
                hh * 1.2,
                s * 0.08,
                darken(color, 0.8),
            );
            // Flag pole
            m.add_cylinder(
                -hw * 0.7,
                hh * 2.5,
                hd * 0.7,
                s * 0.015,
                s * 0.6,
                4,
                [0.6, 0.6, 0.6, 1.0],
            );
        }
        ServiceType::PoliceStation | ServiceType::PoliceKiosk | ServiceType::PoliceHQ => {
            let color = [0.41, 0.53, 0.66, 1.0];
            let hw = s * 0.4 * scale_x;
            let hh = s * 0.3;
            let hd = s * 0.4 * scale_z;
            m.add_cuboid(0.0, hh, 0.0, hw, hh, hd, color);
            // Entrance columns
            m.add_cuboid(
                -hw * 0.3,
                hh,
                hd + s * 0.04,
                s * 0.03,
                hh,
                s * 0.03,
                [0.7, 0.7, 0.7, 1.0],
            );
            m.add_cuboid(
                hw * 0.3,
                hh,
                hd + s * 0.04,
                s * 0.03,
                hh,
                s * 0.03,
                [0.7, 0.7, 0.7, 1.0],
            );
            // Blue dome
            m.add_cylinder(
                0.0,
                hh * 2.0 + s * 0.05,
                0.0,
                s * 0.06,
                s * 0.08,
                6,
                [0.3, 0.5, 0.9, 1.0],
            );
            // Flag pole
            m.add_cylinder(
                hw * 0.8,
                hh * 2.0 + s * 0.2,
                hd * 0.8,
                s * 0.015,
                s * 0.5,
                4,
                [0.6, 0.6, 0.6, 1.0],
            );
        }
        ServiceType::Prison => {
            let color = [0.45, 0.45, 0.45, 1.0];
            let hw = s * 0.45 * scale_x;
            let hh = s * 0.35;
            let hd = s * 0.45 * scale_z;
            m.add_cuboid(0.0, hh, 0.0, hw, hh, hd, color);
            let wall = darken(color, 0.7);
            let wt = s * 0.03;
            m.add_cuboid(0.0, hh * 1.2, hd, hw, hh * 0.15, wt, wall);
            m.add_cuboid(0.0, hh * 1.2, -hd, hw, hh * 0.15, wt, wall);
            m.add_cuboid(hw, hh * 1.2, 0.0, wt, hh * 0.15, hd, wall);
            m.add_cuboid(-hw, hh * 1.2, 0.0, wt, hh * 0.15, hd, wall);
            // Guard towers at corners
            m.add_cuboid(
                hw,
                hh * 2.0,
                hd,
                s * 0.06,
                hh * 0.5,
                s * 0.06,
                darken(color, 0.6),
            );
            m.add_cuboid(
                -hw,
                hh * 2.0,
                -hd,
                s * 0.06,
                hh * 0.5,
                s * 0.06,
                darken(color, 0.6),
            );
        }
        ServiceType::Hospital | ServiceType::MedicalClinic | ServiceType::MedicalCenter => {
            let color = [0.94, 0.69, 0.75, 1.0];
            let hw = s * 0.4 * scale_x;
            let hh = s * 0.45;
            let hd = s * 0.4 * scale_z;
            // Multi-story building
            m.add_cuboid(0.0, hh, 0.0, hw, hh, hd, color);
            // Windows on each floor
            let n_floors = 4;
            let floor_h = hh * 2.0 / n_floors as f32;
            for floor in 0..n_floors {
                let y = floor_h * 0.5 + floor as f32 * floor_h;
                let spacing = (hw * 2.0) / 5.0;
                for i in 1..5 {
                    let wx = -hw + i as f32 * spacing;
                    m.add_cuboid(
                        wx,
                        y,
                        hd - 0.05,
                        s * 0.03,
                        s * 0.04,
                        0.08,
                        [0.2, 0.22, 0.3, 1.0],
                    );
                }
            }
            // Red cross on facade
            let cross = [0.9, 0.1, 0.1, 1.0];
            m.add_cuboid(
                0.0,
                hh * 1.7,
                hd + 0.03,
                s * 0.15,
                s * 0.04,
                s * 0.02,
                cross,
            );
            m.add_cuboid(
                0.0,
                hh * 1.7,
                hd + 0.03,
                s * 0.04,
                s * 0.15,
                s * 0.02,
                cross,
            );
            // Entrance
            m.add_cuboid(
                0.0,
                hh * 0.3,
                hd + 0.05,
                hw * 0.25,
                hh * 0.3,
                0.05,
                darken(color, 0.5),
            );
        }
        _ => {}
    }
}
