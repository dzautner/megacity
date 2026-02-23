//! Procedural meshes for infrastructure service buildings:
//! heating boilers, district heating plants, geothermal plants,
//! water treatment plants, and well pumps.

use simulation::services::ServiceType;

use super::mesh_data::darken;
use super::MeshData;

/// Populate `m` with geometry for an infrastructure building.
#[allow(clippy::too_many_arguments)]
pub(crate) fn generate_infrastructure_mesh(
    m: &mut MeshData,
    service_type: ServiceType,
    s: f32,
    scale_x: f32,
    scale_z: f32,
) {
    match service_type {
        ServiceType::HeatingBoiler => {
            let color = [0.85, 0.40, 0.20, 1.0];
            let hw = s * 0.35;
            let hh = s * 0.25;
            let hd = s * 0.35;
            m.add_cuboid(0.0, hh, 0.0, hw, hh, hd, color);
            // Chimney / smokestack
            m.add_cylinder(
                hw * 0.5,
                hh * 2.5,
                -hd * 0.3,
                s * 0.06,
                hh * 2.0,
                8,
                [0.5, 0.5, 0.5, 1.0],
            );
            // Pipe network on side
            m.add_cuboid(
                -hw * 0.6,
                hh * 0.8,
                0.0,
                s * 0.03,
                hh * 0.6,
                hd * 0.5,
                [0.6, 0.6, 0.65, 1.0],
            );
            // Door
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
        ServiceType::DistrictHeatingPlant => {
            let color = [0.75, 0.35, 0.15, 1.0];
            let hw = s * 0.45 * scale_x;
            let hh = s * 0.35;
            let hd = s * 0.45 * scale_z;
            m.add_cuboid(0.0, hh, 0.0, hw, hh, hd, color);
            // Two chimneys
            m.add_cylinder(
                -hw * 0.3,
                hh * 2.8,
                -hd * 0.3,
                s * 0.08,
                hh * 2.0,
                8,
                [0.5, 0.5, 0.55, 1.0],
            );
            m.add_cylinder(
                hw * 0.3,
                hh * 2.5,
                -hd * 0.3,
                s * 0.06,
                hh * 1.8,
                8,
                [0.55, 0.55, 0.6, 1.0],
            );
            // Pipe network along the front
            m.add_cuboid(
                0.0,
                hh * 0.6,
                hd + s * 0.04,
                hw * 0.8,
                s * 0.04,
                s * 0.04,
                [0.6, 0.6, 0.65, 1.0],
            );
            m.add_cuboid(
                0.0,
                hh * 1.0,
                hd + s * 0.04,
                hw * 0.8,
                s * 0.04,
                s * 0.04,
                [0.6, 0.6, 0.65, 1.0],
            );
            // Loading bay
            m.add_cuboid(
                hw * 0.5,
                hh * 0.4,
                hd + 0.05,
                hw * 0.3,
                hh * 0.4,
                0.05,
                darken(color, 0.4),
            );
        }
        ServiceType::GeothermalPlant => {
            let color = [0.55, 0.40, 0.25, 1.0];
            let hw = s * 0.45 * scale_x;
            let hh = s * 0.30;
            let hd = s * 0.45 * scale_z;
            m.add_cuboid(0.0, hh, 0.0, hw, hh, hd, color);
            // Geothermal dome (heat exchanger)
            m.add_cylinder(
                0.0,
                hh * 2.0 + s * 0.08,
                0.0,
                s * 0.18,
                s * 0.14,
                10,
                [0.65, 0.45, 0.30, 1.0],
            );
            // Steam vents
            m.add_cylinder(
                -hw * 0.5,
                hh * 2.0,
                hd * 0.4,
                s * 0.04,
                s * 0.3,
                6,
                [0.7, 0.7, 0.75, 1.0],
            );
            m.add_cylinder(
                hw * 0.5,
                hh * 2.0,
                -hd * 0.4,
                s * 0.04,
                s * 0.3,
                6,
                [0.7, 0.7, 0.75, 1.0],
            );
            // Underground pipe indicators
            m.add_cuboid(
                0.0,
                s * 0.04,
                hd + s * 0.06,
                hw * 0.3,
                s * 0.04,
                s * 0.04,
                [0.5, 0.5, 0.55, 1.0],
            );
        }
        ServiceType::WaterTreatmentPlant => {
            let color = [0.30, 0.55, 0.70, 1.0];
            let hw = s * 0.45 * scale_x;
            let hh = s * 0.25;
            let hd = s * 0.45 * scale_z;
            // Main processing building
            m.add_cuboid(-hw * 0.3, hh, 0.0, hw * 0.35, hh, hd * 0.6, color);
            // Circular settling tanks
            m.add_cylinder(
                hw * 0.25,
                s * 0.08,
                -hd * 0.35,
                s * 0.18,
                s * 0.08,
                12,
                [0.35, 0.60, 0.75, 1.0],
            );
            m.add_cylinder(
                hw * 0.25,
                s * 0.08,
                hd * 0.35,
                s * 0.18,
                s * 0.08,
                12,
                [0.35, 0.60, 0.75, 1.0],
            );
            // Tank rims
            m.add_cylinder(
                hw * 0.25,
                s * 0.16,
                -hd * 0.35,
                s * 0.19,
                s * 0.01,
                12,
                darken(color, 0.7),
            );
            m.add_cylinder(
                hw * 0.25,
                s * 0.16,
                hd * 0.35,
                s * 0.19,
                s * 0.01,
                12,
                darken(color, 0.7),
            );
            // Pipe connecting tanks to building
            m.add_cuboid(
                0.0,
                hh * 0.5,
                0.0,
                hw * 0.5,
                s * 0.03,
                s * 0.03,
                [0.5, 0.5, 0.55, 1.0],
            );
            // Outflow pipe
            m.add_cuboid(
                hw * 0.45,
                s * 0.06,
                hd * 0.6,
                s * 0.04,
                s * 0.04,
                s * 0.15,
                [0.4, 0.55, 0.65, 1.0],
            );
            // Small office/control room on top
            m.add_cuboid(
                -hw * 0.3,
                hh * 2.0 + s * 0.04,
                0.0,
                hw * 0.15,
                s * 0.08,
                hd * 0.2,
                darken(color, 0.85),
            );
        }
        ServiceType::WellPump => {
            let color = [0.40, 0.60, 0.55, 1.0];
            let hw = s * 0.30;
            let hh = s * 0.20;
            let hd = s * 0.30;
            // Concrete base/pad
            m.add_cuboid(
                0.0,
                s * 0.03,
                0.0,
                hw,
                s * 0.03,
                hd,
                [0.55, 0.55, 0.55, 1.0],
            );
            // Pump housing (small building)
            m.add_cuboid(0.0, hh, 0.0, hw * 0.6, hh, hd * 0.6, color);
            // Pump motor on top
            m.add_cylinder(
                0.0,
                hh * 2.0 + s * 0.04,
                0.0,
                s * 0.06,
                s * 0.06,
                8,
                [0.5, 0.5, 0.55, 1.0],
            );
            // Pipe going into the ground
            m.add_cylinder(
                hw * 0.4,
                hh * 0.5,
                hd * 0.4,
                s * 0.03,
                hh * 1.5,
                6,
                [0.45, 0.45, 0.50, 1.0],
            );
            // Horizontal output pipe
            m.add_cuboid(
                hw * 0.4,
                hh * 0.6,
                0.0,
                s * 0.03,
                s * 0.03,
                hd * 0.6,
                [0.45, 0.45, 0.50, 1.0],
            );
            // Small access hatch on pump housing
            m.add_cuboid(
                0.0,
                hh * 0.35,
                hd * 0.6 + 0.05,
                hw * 0.15,
                hh * 0.35,
                0.05,
                darken(color, 0.4),
            );
        }
        _ => {}
    }
}
