//! Procedural meshes for utility buildings:
//! power plants, solar farms, wind turbines, water towers, sewage plants,
//! nuclear plants, geothermal plants, pumping stations, and water treatment.

use simulation::utilities::UtilityType;

use super::mesh_data::darken;
use super::MeshData;

/// Build a procedural `Mesh` for the given utility type.
pub(crate) fn generate_utility_mesh(utility_type: UtilityType) -> bevy::prelude::Mesh {
    let mut m = MeshData::new();
    let s = simulation::config::CELL_SIZE;

    match utility_type {
        UtilityType::PowerPlant => {
            let color = [0.9, 0.5, 0.1, 1.0];
            m.add_cuboid(0.0, s * 0.25, 0.0, s * 0.4, s * 0.25, s * 0.35, color);
            m.add_cylinder(
                s * 0.2,
                s * 0.45,
                s * 0.15,
                s * 0.1,
                s * 0.4,
                8,
                [0.6, 0.6, 0.6, 1.0],
            );
            m.add_cylinder(
                -s * 0.15,
                s * 0.45,
                -s * 0.1,
                s * 0.08,
                s * 0.35,
                8,
                [0.6, 0.6, 0.6, 1.0],
            );
        }
        UtilityType::SolarFarm => {
            let color = [0.2, 0.25, 0.4, 1.0];
            for i in 0..3 {
                let z = (i as f32 - 1.0) * s * 0.25;
                m.add_cuboid(0.0, s * 0.1, z, s * 0.35, s * 0.01, s * 0.08, color);
            }
            m.add_cuboid(
                0.0,
                s * 0.05,
                0.0,
                s * 0.02,
                s * 0.05,
                s * 0.02,
                [0.5, 0.5, 0.5, 1.0],
            );
        }
        UtilityType::WindTurbine => {
            let color = [0.85, 0.88, 0.9, 1.0];
            m.add_cylinder(0.0, s * 0.5, 0.0, s * 0.03, s * 1.0, 6, color);
            m.add_cuboid(
                0.0,
                s * 1.0,
                0.0,
                s * 0.05,
                s * 0.04,
                s * 0.04,
                [0.7, 0.7, 0.7, 1.0],
            );
            m.add_cuboid(
                0.0,
                s * 1.0 + s * 0.2,
                s * 0.02,
                s * 0.015,
                s * 0.25,
                s * 0.015,
                color,
            );
            m.add_cuboid(
                s * 0.17,
                s * 1.0 - s * 0.12,
                s * 0.02,
                s * 0.015,
                s * 0.12,
                s * 0.015,
                color,
            );
            m.add_cuboid(
                -s * 0.17,
                s * 1.0 - s * 0.12,
                s * 0.02,
                s * 0.015,
                s * 0.12,
                s * 0.015,
                color,
            );
        }
        UtilityType::WaterTower => {
            let color = [0.2, 0.7, 0.85, 1.0];
            for (dx, dz) in &[(0.08, 0.08), (-0.08, 0.08), (0.08, -0.08), (-0.08, -0.08)] {
                m.add_cylinder(
                    dx * s,
                    s * 0.2,
                    dz * s,
                    s * 0.02,
                    s * 0.4,
                    4,
                    [0.5, 0.5, 0.5, 1.0],
                );
            }
            m.add_cylinder(0.0, s * 0.5, 0.0, s * 0.15, s * 0.2, 8, color);
        }
        UtilityType::SewagePlant => {
            let color = [0.45, 0.55, 0.40, 1.0];
            m.add_cuboid(0.0, s * 0.15, 0.0, s * 0.4, s * 0.15, s * 0.35, color);
            m.add_cylinder(
                s * 0.15,
                s * 0.32,
                s * 0.12,
                s * 0.1,
                s * 0.02,
                8,
                darken(color, 0.6),
            );
        }
        UtilityType::NuclearPlant => {
            let color = [0.7, 0.7, 0.75, 1.0];
            m.add_cuboid(s * 0.15, s * 0.25, 0.0, s * 0.25, s * 0.25, s * 0.3, color);
            m.add_cylinder(
                -s * 0.15,
                s * 0.35,
                0.0,
                s * 0.18,
                s * 0.3,
                12,
                [0.75, 0.75, 0.8, 1.0],
            );
        }
        UtilityType::Geothermal => {
            let color = [0.65, 0.45, 0.30, 1.0];
            m.add_cuboid(0.0, s * 0.2, 0.0, s * 0.35, s * 0.2, s * 0.35, color);
            m.add_cylinder(
                s * 0.2,
                s * 0.4,
                s * 0.2,
                s * 0.04,
                s * 0.3,
                6,
                [0.5, 0.5, 0.5, 1.0],
            );
        }
        UtilityType::PumpingStation => {
            let color = [0.3, 0.6, 0.8, 1.0];
            m.add_cuboid(0.0, s * 0.15, 0.0, s * 0.3, s * 0.15, s * 0.3, color);
        }
        UtilityType::WaterTreatment => {
            let color = [0.25, 0.55, 0.75, 1.0];
            m.add_cuboid(0.0, s * 0.2, 0.0, s * 0.4, s * 0.2, s * 0.35, color);
            m.add_cylinder(
                s * 0.15,
                s * 0.42,
                s * 0.1,
                s * 0.12,
                s * 0.02,
                8,
                darken(color, 0.6),
            );
        }
    }

    m.into_mesh()
}
