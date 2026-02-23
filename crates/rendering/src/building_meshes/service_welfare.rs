//! Procedural meshes for welfare and social-service buildings:
//! homeless shelters, welfare offices, post offices, and mail sorting centers.

use simulation::services::ServiceType;

use super::mesh_data::darken;
use super::MeshData;

/// Populate `m` with geometry for a welfare/social-service building.
pub(crate) fn generate_welfare_mesh(m: &mut MeshData, service_type: ServiceType, s: f32) {
    match service_type {
        ServiceType::HomelessShelter => {
            let color = [0.65, 0.55, 0.45, 1.0];
            let hw = s * 0.40;
            let hh = s * 0.25;
            let hd = s * 0.40;
            m.add_cuboid(0.0, hh, 0.0, hw, hh, hd, color);
            // Peaked roof
            m.add_roof_prism(
                0.0,
                hh * 2.0,
                0.0,
                hw * 1.02,
                hh * 0.35,
                hd * 1.02,
                darken(color, 0.8),
            );
            // Entrance (wide double door)
            m.add_cuboid(
                0.0,
                hh * 0.4,
                hd + 0.05,
                hw * 0.3,
                hh * 0.4,
                0.05,
                darken(color, 0.4),
            );
            // Windows
            m.add_cuboid(
                -hw * 0.5,
                hh,
                hd - 0.05,
                s * 0.04,
                s * 0.04,
                0.08,
                [0.25, 0.25, 0.35, 1.0],
            );
            m.add_cuboid(
                hw * 0.5,
                hh,
                hd - 0.05,
                s * 0.04,
                s * 0.04,
                0.08,
                [0.25, 0.25, 0.35, 1.0],
            );
            // Small sign
            m.add_cuboid(
                hw * 0.7,
                hh * 1.5,
                hd * 0.8,
                s * 0.06,
                s * 0.04,
                s * 0.01,
                [0.3, 0.6, 0.4, 1.0],
            );
        }
        ServiceType::WelfareOffice => {
            let color = [0.45, 0.60, 0.55, 1.0];
            let hw = s * 0.40;
            let hh = s * 0.30;
            let hd = s * 0.40;
            m.add_cuboid(0.0, hh, 0.0, hw, hh, hd, color);
            // Entrance with columns
            m.add_cuboid(
                -hw * 0.25,
                hh,
                hd + s * 0.04,
                s * 0.025,
                hh,
                s * 0.025,
                [0.7, 0.7, 0.72, 1.0],
            );
            m.add_cuboid(
                hw * 0.25,
                hh,
                hd + s * 0.04,
                s * 0.025,
                hh,
                s * 0.025,
                [0.7, 0.7, 0.72, 1.0],
            );
            // Steps
            m.add_cuboid(
                0.0,
                hh * 0.12,
                hd + s * 0.07,
                hw * 0.5,
                hh * 0.12,
                s * 0.05,
                darken(color, 0.8),
            );
            // Windows on each floor
            for i in 1..4 {
                let wx = -hw + i as f32 * hw * 0.5;
                m.add_cuboid(
                    wx,
                    hh,
                    hd - 0.05,
                    s * 0.035,
                    s * 0.04,
                    0.08,
                    [0.2, 0.22, 0.3, 1.0],
                );
            }
            // Flat roof with small sign
            m.add_cuboid(
                0.0,
                hh * 2.0 + s * 0.02,
                0.0,
                hw * 0.3,
                s * 0.02,
                hd * 0.3,
                darken(color, 0.85),
            );
            // Flag pole
            m.add_cylinder(
                hw * 0.7,
                hh * 2.5,
                hd * 0.7,
                s * 0.015,
                s * 0.5,
                4,
                [0.5, 0.5, 0.55, 1.0],
            );
        }
        ServiceType::PostOffice => {
            let color = [0.72, 0.55, 0.38, 1.0];
            let hw = s * 0.38;
            let hh = s * 0.28;
            let hd = s * 0.38;
            m.add_cuboid(0.0, hh, 0.0, hw, hh, hd, color);
            // Peaked roof
            m.add_roof_prism(
                0.0,
                hh * 2.0,
                0.0,
                hw * 1.02,
                hh * 0.35,
                hd * 1.02,
                darken(color, 0.8),
            );
            // Entrance door
            m.add_cuboid(
                0.0,
                hh * 0.4,
                hd + 0.05,
                hw * 0.2,
                hh * 0.4,
                0.05,
                darken(color, 0.4),
            );
            // Windows
            m.add_cuboid(
                -hw * 0.5,
                hh,
                hd - 0.05,
                s * 0.04,
                s * 0.04,
                0.08,
                [0.2, 0.22, 0.3, 1.0],
            );
            m.add_cuboid(
                hw * 0.5,
                hh,
                hd - 0.05,
                s * 0.04,
                s * 0.04,
                0.08,
                [0.2, 0.22, 0.3, 1.0],
            );
            // Mailbox
            m.add_cuboid(
                hw * 0.7,
                s * 0.08,
                hd + s * 0.06,
                s * 0.04,
                s * 0.08,
                s * 0.03,
                [0.2, 0.3, 0.7, 1.0],
            );
            // Flag pole
            m.add_cylinder(
                -hw * 0.7,
                hh * 2.5,
                hd * 0.7,
                s * 0.015,
                s * 0.5,
                4,
                [0.6, 0.6, 0.6, 1.0],
            );
        }
        ServiceType::MailSortingCenter => {
            let color = [0.55, 0.50, 0.45, 1.0];
            let hw = s * 0.45;
            let hh = s * 0.30;
            let hd = s * 0.45;
            m.add_cuboid(0.0, hh, 0.0, hw, hh, hd, color);
            // Flat roof
            m.add_cuboid(
                0.0,
                hh * 2.0 + s * 0.01,
                0.0,
                hw * 1.02,
                s * 0.01,
                hd * 1.02,
                darken(color, 0.85),
            );
            // Loading dock
            m.add_cuboid(
                0.0,
                hh * 0.3,
                -hd - s * 0.06,
                hw * 0.8,
                hh * 0.3,
                s * 0.06,
                darken(color, 0.7),
            );
            // Loading bay doors
            m.add_cuboid(
                -hw * 0.4,
                hh * 0.5,
                hd + 0.05,
                hw * 0.2,
                hh * 0.5,
                0.05,
                darken(color, 0.4),
            );
            m.add_cuboid(
                hw * 0.4,
                hh * 0.5,
                hd + 0.05,
                hw * 0.2,
                hh * 0.5,
                0.05,
                darken(color, 0.4),
            );
            // Conveyor belt indicator on roof
            m.add_cuboid(
                0.0,
                hh * 2.0 + s * 0.06,
                0.0,
                hw * 0.1,
                s * 0.04,
                hd * 0.6,
                [0.4, 0.4, 0.45, 1.0],
            );
            // Sorting center sign
            m.add_cuboid(
                0.0,
                hh * 1.8,
                hd + 0.03,
                hw * 0.3,
                s * 0.04,
                s * 0.01,
                [0.2, 0.3, 0.7, 1.0],
            );
        }
        _ => {}
    }
}
