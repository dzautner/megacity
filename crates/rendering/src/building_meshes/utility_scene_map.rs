//! Mapping from `UtilityType` → GLB asset path for 3D scene models.
//!
//! All utility models come from the Kenney industrial kit and live under
//! `assets/models/buildings/utilities/` with a shared industrial colormap.

use simulation::utilities::UtilityType;

/// Returns the asset path (relative to `assets/`) for the given utility type,
/// or `None` if no GLB model has been assigned yet (fallback to procedural mesh).
pub fn utility_scene_path(utility_type: UtilityType) -> Option<&'static str> {
    let path = match utility_type {
        UtilityType::PowerPlant => "models/buildings/utilities/power-plant.glb",
        UtilityType::SolarFarm => "models/buildings/utilities/solar-farm.glb",
        UtilityType::WindTurbine => "models/buildings/utilities/wind-turbine.glb",
        UtilityType::WaterTower => "models/buildings/utilities/water-tower.glb",
        UtilityType::SewagePlant => "models/buildings/utilities/sewage-plant.glb",
        UtilityType::NuclearPlant => "models/buildings/utilities/nuclear-plant.glb",
        UtilityType::Geothermal => "models/buildings/utilities/geothermal.glb",
        UtilityType::PumpingStation => "models/buildings/utilities/pumping-station.glb",
        UtilityType::WaterTreatment => "models/buildings/utilities/water-treatment.glb",
        UtilityType::HydroDam => "models/buildings/utilities/hydro-dam.glb",
        UtilityType::OilPlant => "models/buildings/utilities/oil-plant.glb",
        UtilityType::GasPlant => "models/buildings/utilities/gas-plant.glb",
    };
    Some(path)
}

#[cfg(test)]
mod tests {
    use super::utility_scene_path;
    use simulation::utilities::UtilityType;
    use std::path::PathBuf;

    fn asset_path(rel: &str) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../app/assets")
            .join(rel)
    }

    #[test]
    fn all_utility_scene_paths_exist() {
        let all = [
            UtilityType::PowerPlant,
            UtilityType::SolarFarm,
            UtilityType::WindTurbine,
            UtilityType::WaterTower,
            UtilityType::SewagePlant,
            UtilityType::NuclearPlant,
            UtilityType::Geothermal,
            UtilityType::PumpingStation,
            UtilityType::WaterTreatment,
            UtilityType::HydroDam,
            UtilityType::OilPlant,
            UtilityType::GasPlant,
        ];

        for ty in all {
            let rel = utility_scene_path(ty).unwrap_or_else(|| panic!("missing scene path for {ty:?}"));
            let abs = asset_path(rel);
            assert!(
                abs.exists(),
                "utility scene path does not exist for {ty:?}: {}",
                abs.display()
            );
        }
    }
}
