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
