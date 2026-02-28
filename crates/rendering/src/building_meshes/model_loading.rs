//! Startup system that loads all GLB models into `BuildingModelCache`.

use bevy::prelude::*;
use std::collections::HashMap;

use simulation::services::ServiceType;
use simulation::utilities::UtilityType;

use super::model_cache::BuildingModelCache;
use super::service_scene_map::service_scene_path;
use super::utility_scene_map::utility_scene_path;

/// Startup system: load all GLB models from assets/models/ directory
pub fn load_building_models(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let load_scene = |path: String| -> Handle<Scene> {
        asset_server.load(bevy::gltf::GltfAssetLabel::Scene(0).from_asset(path))
    };

    let residential = load_residential(&load_scene);
    let (commercial, skyscrapers) = load_commercial(&load_scene);
    let industrial = load_industrial(&load_scene);
    let vehicles = load_vehicles(&load_scene);
    let characters = load_characters(&load_scene);
    let trees = load_trees(&load_scene);
    let props = load_props(&load_scene);
    let service_scenes = load_service_scenes(&load_scene);
    let utility_scenes = load_utility_scenes(&load_scene);

    let fallback_material = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        perceptual_roughness: 0.8,
        ..default()
    });

    commands.insert_resource(BuildingModelCache {
        residential,
        commercial,
        skyscrapers,
        industrial,
        vehicles,
        characters,
        trees,
        props,
        service_scenes,
        utility_scenes,
        service_meshes: HashMap::new(),
        utility_meshes: HashMap::new(),
        fallback_material,
    });
}

fn load_residential(load_scene: &dyn Fn(String) -> Handle<Scene>) -> Vec<Handle<Scene>> {
    let files = [
        "a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k", "l", "m", "n", "o", "p",
        "q", "r", "s", "t", "u",
    ];
    files
        .iter()
        .map(|letter| {
            load_scene(format!(
                "models/buildings/residential/building-type-{letter}.glb"
            ))
        })
        .collect()
}

fn load_commercial(
    load_scene: &dyn Fn(String) -> Handle<Scene>,
) -> (Vec<Handle<Scene>>, Vec<Handle<Scene>>) {
    let commercial_files = [
        "a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k", "l", "m", "n",
    ];
    let mut commercial: Vec<Handle<Scene>> = commercial_files
        .iter()
        .map(|letter| load_scene(format!("models/buildings/commercial/building-{letter}.glb")))
        .collect();

    let low_detail_files = [
        "a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k", "l", "m", "n",
    ];
    for letter in &low_detail_files {
        commercial.push(load_scene(format!(
            "models/buildings/commercial/low-detail-building-{letter}.glb"
        )));
    }

    let skyscraper_files = ["a", "b", "c", "d", "e"];
    let mut skyscrapers: Vec<Handle<Scene>> = skyscraper_files
        .iter()
        .map(|letter| {
            load_scene(format!(
                "models/buildings/skyscrapers/building-skyscraper-{letter}.glb"
            ))
        })
        .collect();
    // Extra CC0 skyline variants (Kenney via Poly Pizza) for more distinctive
    // high-density/office silhouettes.
    let vintage_files = ["a", "b", "c", "d", "e", "f"];
    for letter in &vintage_files {
        skyscrapers.push(load_scene(format!(
            "models/buildings/skyscrapers/vintage/skyscraper-vintage-{letter}.glb"
        )));
    }

    (commercial, skyscrapers)
}

fn load_industrial(load_scene: &dyn Fn(String) -> Handle<Scene>) -> Vec<Handle<Scene>> {
    let files = [
        "a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k", "l", "m", "n", "o", "p",
        "q", "r", "s", "t",
    ];
    files
        .iter()
        .map(|letter| load_scene(format!("models/buildings/industrial/building-{letter}.glb")))
        .collect()
}

fn load_vehicles(load_scene: &dyn Fn(String) -> Handle<Scene>) -> Vec<Handle<Scene>> {
    let files = [
        "sedan",
        "sedan-sports",
        "hatchback-sports",
        "suv",
        "suv-luxury",
        "van",
        "truck",
        "taxi",
        "police",
        "ambulance",
        "firetruck",
        "garbage-truck",
        "delivery",
        "delivery-flat",
        "truck-flat",
    ];
    files
        .iter()
        .map(|name| load_scene(format!("models/vehicles/{name}.glb")))
        .collect()
}

fn load_characters(load_scene: &dyn Fn(String) -> Handle<Scene>) -> Vec<Handle<Scene>> {
    let files = [
        "character-female-a",
        "character-female-b",
        "character-female-c",
        "character-female-d",
        "character-female-e",
        "character-female-f",
        "character-male-a",
        "character-male-b",
        "character-male-c",
        "character-male-d",
        "character-male-e",
        "character-male-f",
    ];
    files
        .iter()
        .map(|name| load_scene(format!("models/characters/{name}.glb")))
        .collect()
}

fn load_trees(load_scene: &dyn Fn(String) -> Handle<Scene>) -> Vec<Handle<Scene>> {
    let prop_tree_files = [
        "tree-suburban",
        "tree-retro-large",
        "tree-park-large",
        "tree-park-pine-large",
    ];
    let mut trees: Vec<Handle<Scene>> = prop_tree_files
        .iter()
        .map(|name| load_scene(format!("models/props/{name}.glb")))
        .collect();

    let nature_tree_files = [
        "tree_cone_fall",
        "tree_pineRoundA",
        "tree_pineRoundB",
        "tree_pineRoundC",
        "tree_tall_dark",
        "tree_palmTall",
        "tree_palmDetailedShort",
    ];
    for name in &nature_tree_files {
        trees.push(load_scene(format!("models/nature/{name}.glb")));
    }
    trees
}

fn load_props(load_scene: &dyn Fn(String) -> Handle<Scene>) -> Vec<Handle<Scene>> {
    let files = [
        "detail-bench",
        "detail-light-single",
        "detail-light-double",
        "detail-barrier-type-a",
        "detail-dumpster-closed",
        "planter",
    ];
    files
        .iter()
        .map(|name| load_scene(format!("models/props/{name}.glb")))
        .collect()
}

/// Load GLB scene handles for all service types that have a model mapping.
fn load_service_scenes(
    load_scene: &dyn Fn(String) -> Handle<Scene>,
) -> HashMap<ServiceType, Handle<Scene>> {
    use ServiceType::*;
    let all_types = [
        FireStation,
        FireHouse,
        FireHQ,
        PoliceStation,
        PoliceKiosk,
        PoliceHQ,
        Prison,
        Hospital,
        MedicalClinic,
        MedicalCenter,
        ElementarySchool,
        HighSchool,
        Kindergarten,
        University,
        Library,
        CityHall,
        Museum,
        Cathedral,
        CellTower,
        Crematorium,
        TrainStation,
        BusDepot,
        SubwayStation,
        TramDepot,
        DataCenter,
        TransferStation,
        Incinerator,
        RecyclingCenter,
        Landfill,
        WaterTreatmentPlant,
        HeatingBoiler,
        DistrictHeatingPlant,
        GeothermalPlant,
        WellPump,
        PostOffice,
        WelfareOffice,
        HomelessShelter,
        MailSortingCenter,
        Daycare,
        Eldercare,
        SeniorCenter,
        YouthCenter,
        CommunityCenter,
        SubstanceAbuseTreatmentCenter,
        Stadium,
    ];
    let mut map = HashMap::new();
    for st in all_types {
        if let Some(path) = service_scene_path(st) {
            map.insert(st, load_scene(path.to_string()));
        }
    }
    map
}

/// Load GLB scene handles for all utility types that have a model mapping.
fn load_utility_scenes(
    load_scene: &dyn Fn(String) -> Handle<Scene>,
) -> HashMap<UtilityType, Handle<Scene>> {
    use UtilityType::*;
    let all_types = [
        PowerPlant,
        SolarFarm,
        WindTurbine,
        WaterTower,
        SewagePlant,
        NuclearPlant,
        Geothermal,
        PumpingStation,
        WaterTreatment,
        HydroDam,
        OilPlant,
        GasPlant,
    ];
    let mut map = HashMap::new();
    for ut in all_types {
        if let Some(path) = utility_scene_path(ut) {
            map.insert(ut, load_scene(path.to_string()));
        }
    }
    map
}
