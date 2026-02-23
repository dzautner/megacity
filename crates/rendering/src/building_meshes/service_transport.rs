//! Procedural meshes for transport service buildings:
//! train stations, bus depots, subway/tram stations, airports, and ferry piers.

use simulation::services::ServiceType;

use super::mesh_data::{darken, lighten};
use super::MeshData;

/// Populate `m` with geometry for a transport-service building.
#[allow(clippy::too_many_arguments)]
pub(crate) fn generate_transport_mesh(
    m: &mut MeshData,
    service_type: ServiceType,
    s: f32,
    scale_x: f32,
    scale_z: f32,
) {
    match service_type {
        ServiceType::TrainStation => {
            let color = [0.50, 0.55, 0.62, 1.0];
            let hw = s * 0.45 * scale_x;
            let hh = s * 0.25;
            let hd = s * 0.4 * scale_z;
            // Main station building
            m.add_cuboid(0.0, hh, 0.0, hw * 0.5, hh, hd, color);
            // Platform canopy (flat roof on columns)
            m.add_cuboid(
                hw * 0.3,
                hh * 1.8,
                0.0,
                hw * 0.5,
                s * 0.02,
                hd * 1.1,
                darken(color, 0.8),
            );
            // Canopy columns
            for i in 0..4 {
                let z = -hd * 0.8 + i as f32 * hd * 0.53;
                m.add_cuboid(
                    hw * 0.05,
                    hh * 0.9,
                    z,
                    s * 0.02,
                    hh * 0.9,
                    s * 0.02,
                    [0.5, 0.5, 0.52, 1.0],
                );
                m.add_cuboid(
                    hw * 0.55,
                    hh * 0.9,
                    z,
                    s * 0.02,
                    hh * 0.9,
                    s * 0.02,
                    [0.5, 0.5, 0.52, 1.0],
                );
            }
            // Clock tower
            m.add_cuboid(
                0.0,
                hh * 2.5,
                0.0,
                s * 0.06,
                hh * 0.8,
                s * 0.06,
                darken(color, 0.85),
            );
            m.add_cuboid(
                0.0,
                hh * 3.2,
                0.0,
                s * 0.04,
                s * 0.04,
                s * 0.04,
                [0.8, 0.78, 0.65, 1.0],
            );
        }
        ServiceType::BusDepot => {
            let color = [0.50, 0.58, 0.65, 1.0];
            let hw = s * 0.45 * scale_x;
            let hh = s * 0.30;
            let hd = s * 0.4 * scale_z;
            // Open-sided garage structure (roof on columns)
            m.add_cuboid(0.0, hh * 2.0, 0.0, hw, s * 0.03, hd, darken(color, 0.85));
            // Columns
            m.add_cuboid(-hw, hh, -hd, s * 0.04, hh, s * 0.04, [0.5, 0.5, 0.52, 1.0]);
            m.add_cuboid(hw, hh, -hd, s * 0.04, hh, s * 0.04, [0.5, 0.5, 0.52, 1.0]);
            m.add_cuboid(-hw, hh, hd, s * 0.04, hh, s * 0.04, [0.5, 0.5, 0.52, 1.0]);
            m.add_cuboid(hw, hh, hd, s * 0.04, hh, s * 0.04, [0.5, 0.5, 0.52, 1.0]);
            // Back wall
            m.add_cuboid(0.0, hh, -hd, hw, hh, s * 0.03, color);
            // Parked bus shape
            m.add_cuboid(
                0.0,
                s * 0.12,
                0.0,
                s * 0.08,
                s * 0.10,
                s * 0.25,
                [0.2, 0.4, 0.7, 1.0],
            );
        }
        ServiceType::SubwayStation | ServiceType::TramDepot => {
            let color = [0.50, 0.60, 0.70, 1.0];
            let hw = s * 0.4 * scale_x;
            let hh = s * 0.20;
            let hd = s * 0.4 * scale_z;
            // Entrance building
            m.add_cuboid(0.0, hh, 0.0, hw, hh, hd, color);
            // Stairs indicator (recessed darker section)
            m.add_cuboid(
                0.0,
                hh * 0.5,
                hd + 0.05,
                hw * 0.4,
                hh * 0.5,
                0.08,
                darken(color, 0.45),
            );
            // Subway sign post
            m.add_cylinder(
                hw * 0.6,
                hh * 2.5,
                hd * 0.6,
                s * 0.02,
                s * 0.3,
                4,
                [0.5, 0.5, 0.55, 1.0],
            );
            m.add_cuboid(
                hw * 0.6,
                hh * 2.0 + s * 0.25,
                hd * 0.6,
                s * 0.06,
                s * 0.06,
                s * 0.02,
                lighten(color, 1.3),
            );
        }
        ServiceType::SmallAirstrip => {
            let color = [0.65, 0.65, 0.70, 1.0];
            let hw = s * 0.45 * scale_x;
            let hh = s * 0.15;
            let hd = s * 0.45 * scale_z;
            // Small terminal building
            m.add_cuboid(0.0, hh, 0.0, hw * 0.4, hh, hd * 0.3, color);
            // Runway
            m.add_cuboid(
                0.0,
                s * 0.01,
                hd * 0.3,
                hw * 0.12,
                s * 0.01,
                hd * 0.7,
                [0.3, 0.3, 0.35, 1.0],
            );
            // Runway center stripe
            m.add_cuboid(
                0.0,
                s * 0.015,
                hd * 0.3,
                hw * 0.01,
                s * 0.005,
                hd * 0.6,
                [1.0, 1.0, 1.0, 0.8],
            );
            // Windsock pole
            m.add_cylinder(
                hw * 0.5,
                hh * 2.0,
                -hd * 0.3,
                s * 0.02,
                s * 0.3,
                4,
                [0.5, 0.5, 0.55, 1.0],
            );
        }
        ServiceType::RegionalAirport => {
            let color = [0.60, 0.62, 0.68, 1.0];
            let hw = s * 0.45 * scale_x;
            let hh = s * 0.20;
            let hd = s * 0.45 * scale_z;
            // Terminal building
            m.add_cuboid(0.0, hh, -hd * 0.2, hw * 0.5, hh, hd * 0.35, color);
            // Terminal extension (gate concourse)
            m.add_cuboid(
                hw * 0.15,
                hh * 0.8,
                -hd * 0.55,
                hw * 0.2,
                hh * 0.6,
                hd * 0.1,
                darken(color, 0.9),
            );
            // Runway
            m.add_cuboid(
                0.0,
                s * 0.01,
                hd * 0.25,
                hw * 0.15,
                s * 0.01,
                hd * 0.75,
                [0.3, 0.3, 0.35, 1.0],
            );
            // Runway center stripe
            m.add_cuboid(
                0.0,
                s * 0.015,
                hd * 0.25,
                hw * 0.01,
                s * 0.005,
                hd * 0.65,
                [1.0, 1.0, 1.0, 0.8],
            );
            // Control tower
            m.add_cylinder(
                hw * 0.55,
                hh * 2.5,
                -hd * 0.3,
                s * 0.05,
                s * 0.45,
                8,
                [0.5, 0.5, 0.55, 1.0],
            );
            // Tower cab (observation deck)
            m.add_cuboid(
                hw * 0.55,
                hh * 2.5 + s * 0.25,
                -hd * 0.3,
                s * 0.08,
                s * 0.06,
                s * 0.08,
                [0.4, 0.6, 0.65, 1.0],
            );
        }
        ServiceType::InternationalAirport => {
            let color = [0.58, 0.60, 0.66, 1.0];
            let hw = s * 0.45 * scale_x;
            let hh = s * 0.22;
            let hd = s * 0.45 * scale_z;
            // Main terminal building (large)
            m.add_cuboid(0.0, hh, -hd * 0.15, hw * 0.6, hh, hd * 0.40, color);
            // Terminal wings (gate concourses on each side)
            m.add_cuboid(
                -hw * 0.35,
                hh * 0.7,
                -hd * 0.6,
                hw * 0.15,
                hh * 0.5,
                hd * 0.15,
                darken(color, 0.9),
            );
            m.add_cuboid(
                hw * 0.35,
                hh * 0.7,
                -hd * 0.6,
                hw * 0.15,
                hh * 0.5,
                hd * 0.15,
                darken(color, 0.9),
            );
            // Two parallel runways
            m.add_cuboid(
                -hw * 0.25,
                s * 0.01,
                hd * 0.3,
                hw * 0.12,
                s * 0.01,
                hd * 0.7,
                [0.3, 0.3, 0.35, 1.0],
            );
            m.add_cuboid(
                hw * 0.25,
                s * 0.01,
                hd * 0.3,
                hw * 0.12,
                s * 0.01,
                hd * 0.7,
                [0.3, 0.3, 0.35, 1.0],
            );
            // Runway center stripes
            m.add_cuboid(
                -hw * 0.25,
                s * 0.015,
                hd * 0.3,
                hw * 0.01,
                s * 0.005,
                hd * 0.6,
                [1.0, 1.0, 1.0, 0.8],
            );
            m.add_cuboid(
                hw * 0.25,
                s * 0.015,
                hd * 0.3,
                hw * 0.01,
                s * 0.005,
                hd * 0.6,
                [1.0, 1.0, 1.0, 0.8],
            );
            // Tall control tower
            m.add_cylinder(
                hw * 0.6,
                hh * 3.0,
                -hd * 0.25,
                s * 0.06,
                s * 0.65,
                8,
                [0.5, 0.5, 0.55, 1.0],
            );
            // Tower cab
            m.add_cuboid(
                hw * 0.6,
                hh * 3.0 + s * 0.35,
                -hd * 0.25,
                s * 0.10,
                s * 0.08,
                s * 0.10,
                [0.3, 0.55, 0.6, 1.0],
            );
            // Parking structure
            m.add_cuboid(
                0.0,
                hh * 0.5,
                hd * 0.15,
                hw * 0.3,
                hh * 0.3,
                hd * 0.1,
                darken(color, 0.7),
            );
        }
        ServiceType::FerryPier => {
            let color = [0.40, 0.55, 0.70, 1.0];
            m.add_cuboid(0.0, s * 0.08, 0.0, s * 0.4, s * 0.08, s * 0.15, color);
            m.add_cuboid(
                0.0,
                s * 0.04,
                s * 0.3,
                s * 0.1,
                s * 0.04,
                s * 0.2,
                darken(color, 0.7),
            );
        }
        _ => {}
    }
}
