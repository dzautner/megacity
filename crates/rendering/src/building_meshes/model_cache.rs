//! GLB model cache resource and the startup system that populates it.

use bevy::prelude::*;
use std::collections::HashMap;

use simulation::config::CELL_SIZE;
use simulation::grid::ZoneType;
use simulation::services::ServiceType;
use simulation::utilities::UtilityType;

use super::service_civic::generate_civic_mesh;
use super::service_education::generate_education_mesh;
use super::service_emergency::generate_emergency_mesh;
use super::service_infrastructure::generate_infrastructure_mesh;
use super::service_recreation::generate_recreation_mesh;
use super::service_transport::generate_transport_mesh;
use super::service_welfare::generate_welfare_mesh;
use super::utility_meshes::generate_utility_mesh;
use super::MeshData;

// ---------------------------------------------------------------------------
// Helper: dispatch ServiceType → category mesh builder
// ---------------------------------------------------------------------------

fn generate_service_mesh(service_type: ServiceType) -> Mesh {
    let mut m = MeshData::new();
    let s = CELL_SIZE;
    let (fw, fh) = simulation::services::ServiceBuilding::footprint(service_type);
    let scale_x = fw as f32;
    let scale_z = fh as f32;

    match service_type {
        // Emergency services
        ServiceType::FireStation
        | ServiceType::FireHouse
        | ServiceType::FireHQ
        | ServiceType::PoliceStation
        | ServiceType::PoliceKiosk
        | ServiceType::PoliceHQ
        | ServiceType::Prison
        | ServiceType::Hospital
        | ServiceType::MedicalClinic
        | ServiceType::MedicalCenter => {
            generate_emergency_mesh(&mut m, service_type, s, scale_x, scale_z);
        }

        // Education
        ServiceType::ElementarySchool
        | ServiceType::HighSchool
        | ServiceType::Kindergarten
        | ServiceType::University
        | ServiceType::Library => {
            generate_education_mesh(&mut m, service_type, s);
        }

        // Recreation / parks
        ServiceType::SmallPark
        | ServiceType::LargePark
        | ServiceType::Playground
        | ServiceType::SportsField
        | ServiceType::Plaza
        | ServiceType::Stadium => {
            generate_recreation_mesh(&mut m, service_type, s, scale_x, scale_z);
        }

        // Transport
        ServiceType::TrainStation
        | ServiceType::BusDepot
        | ServiceType::SubwayStation
        | ServiceType::TramDepot
        | ServiceType::SmallAirstrip
        | ServiceType::RegionalAirport
        | ServiceType::InternationalAirport
        | ServiceType::FerryPier => {
            generate_transport_mesh(&mut m, service_type, s, scale_x, scale_z);
        }

        // Civic / landmarks
        ServiceType::CellTower
        | ServiceType::DataCenter
        | ServiceType::TransferStation
        | ServiceType::CityHall
        | ServiceType::Cathedral
        | ServiceType::Museum
        | ServiceType::Cemetery
        | ServiceType::Crematorium => {
            generate_civic_mesh(&mut m, service_type, s, scale_x, scale_z);
        }

        // Welfare / social services
        ServiceType::HomelessShelter
        | ServiceType::WelfareOffice
        | ServiceType::PostOffice
        | ServiceType::MailSortingCenter => {
            generate_welfare_mesh(&mut m, service_type, s);
        }

        // Infrastructure (heating, water)
        ServiceType::HeatingBoiler
        | ServiceType::DistrictHeatingPlant
        | ServiceType::GeothermalPlant
        | ServiceType::WaterTreatmentPlant
        | ServiceType::WellPump => {
            generate_infrastructure_mesh(&mut m, service_type, s, scale_x, scale_z);
        }

        // Fallback for any unhandled service type
        _ => {
            let color = [0.91, 0.82, 0.38, 1.0];
            let hw = s * 0.4;
            let hh = s * 0.35;
            let hd = s * 0.4;
            m.add_cuboid(0.0, hh, 0.0, hw, hh, hd, color);
        }
    }

    m.into_mesh()
}

// ---------------------------------------------------------------------------
// BuildingModelCache
// ---------------------------------------------------------------------------

/// Pre-loaded scene handles for all building GLB models
#[derive(Resource)]
pub struct BuildingModelCache {
    /// Residential building scenes (suburban houses)
    pub residential: Vec<Handle<Scene>>,
    /// Commercial building scenes
    pub commercial: Vec<Handle<Scene>>,
    /// Commercial skyscraper scenes (tall buildings)
    pub skyscrapers: Vec<Handle<Scene>>,
    /// Industrial building scenes
    pub industrial: Vec<Handle<Scene>>,
    /// Vehicle scenes (sedan, SUV, van, truck, etc.)
    pub vehicles: Vec<Handle<Scene>>,
    /// Character scenes (male/female variants)
    pub characters: Vec<Handle<Scene>>,
    /// Tree/nature scenes
    pub trees: Vec<Handle<Scene>>,
    /// Urban prop scenes (benches, lights, etc.)
    pub props: Vec<Handle<Scene>>,

    /// Fallback procedural meshes for service/utility buildings that don't have GLB models
    pub service_meshes: HashMap<ServiceType, Handle<Mesh>>,
    pub utility_meshes: HashMap<UtilityType, Handle<Mesh>>,
    pub fallback_material: Handle<StandardMaterial>,
}

impl BuildingModelCache {
    /// Get a residential building scene handle based on a hash value for variation
    pub fn get_residential(&self, hash: usize) -> Handle<Scene> {
        if self.residential.is_empty() {
            return self.skyscrapers.first().cloned().unwrap_or_default();
        }
        self.residential[hash % self.residential.len()].clone()
    }

    /// Get a commercial building scene based on level and hash
    pub fn get_commercial(&self, level: u8, hash: usize) -> Handle<Scene> {
        if level >= 4 && !self.skyscrapers.is_empty() {
            return self.skyscrapers[hash % self.skyscrapers.len()].clone();
        }
        if self.commercial.is_empty() {
            return self.skyscrapers.first().cloned().unwrap_or_default();
        }
        self.commercial[hash % self.commercial.len()].clone()
    }

    /// Get an industrial building scene
    pub fn get_industrial(&self, hash: usize) -> Handle<Scene> {
        if self.industrial.is_empty() {
            return self.commercial.first().cloned().unwrap_or_default();
        }
        self.industrial[hash % self.industrial.len()].clone()
    }

    /// Get a building scene for any zone type and level
    pub fn get_zone_scene(&self, zone: ZoneType, level: u8, hash: usize) -> Handle<Scene> {
        match zone {
            ZoneType::ResidentialLow => self.get_residential(hash),
            ZoneType::ResidentialMedium => {
                if !self.commercial.is_empty() {
                    self.commercial[hash % self.commercial.len()].clone()
                } else {
                    self.get_residential(hash)
                }
            }
            ZoneType::ResidentialHigh => {
                if level >= 3 && !self.skyscrapers.is_empty() {
                    self.skyscrapers[hash % self.skyscrapers.len()].clone()
                } else if !self.commercial.is_empty() {
                    self.commercial[hash % self.commercial.len()].clone()
                } else {
                    self.get_residential(hash)
                }
            }
            ZoneType::CommercialLow => self.get_commercial(level, hash),
            ZoneType::CommercialHigh => self.get_commercial(level, hash),
            ZoneType::Industrial => self.get_industrial(hash),
            ZoneType::Office => {
                if level >= 3 && !self.skyscrapers.is_empty() {
                    self.skyscrapers[hash % self.skyscrapers.len()].clone()
                } else {
                    self.get_commercial(level, hash)
                }
            }
            ZoneType::MixedUse => {
                if level >= 3 && !self.skyscrapers.is_empty() {
                    self.skyscrapers[hash % self.skyscrapers.len()].clone()
                } else {
                    self.get_commercial(level, hash)
                }
            }
            ZoneType::None => self.get_residential(hash),
        }
    }

    /// Get a vehicle scene
    pub fn get_vehicle(&self, hash: usize) -> Handle<Scene> {
        if self.vehicles.is_empty() {
            return Handle::default();
        }
        self.vehicles[hash % self.vehicles.len()].clone()
    }

    /// Get a character scene
    pub fn get_character(&self, hash: usize) -> Handle<Scene> {
        if self.characters.is_empty() {
            return Handle::default();
        }
        self.characters[hash % self.characters.len()].clone()
    }

    /// Get a tree scene
    pub fn get_tree(&self, hash: usize) -> Handle<Scene> {
        if self.trees.is_empty() {
            return Handle::default();
        }
        self.trees[hash % self.trees.len()].clone()
    }

    /// Get a prop scene
    pub fn get_prop(&self, hash: usize) -> Handle<Scene> {
        if self.props.is_empty() {
            return Handle::default();
        }
        self.props[hash % self.props.len()].clone()
    }

    pub fn get_or_create_service_mesh(
        &mut self,
        service_type: ServiceType,
        meshes: &mut Assets<Mesh>,
    ) -> Handle<Mesh> {
        self.service_meshes
            .entry(service_type)
            .or_insert_with(|| meshes.add(generate_service_mesh(service_type)))
            .clone()
    }

    pub fn get_or_create_utility_mesh(
        &mut self,
        utility_type: UtilityType,
        meshes: &mut Assets<Mesh>,
    ) -> Handle<Mesh> {
        self.utility_meshes
            .entry(utility_type)
            .or_insert_with(|| meshes.add(generate_utility_mesh(utility_type)))
            .clone()
    }
}

// ---------------------------------------------------------------------------
// Startup system: load all GLB models
// ---------------------------------------------------------------------------

/// Startup system: load all GLB models from assets/models/ directory
pub fn load_building_models(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let load_scene = |path: String| -> Handle<Scene> {
        asset_server.load(bevy::gltf::GltfAssetLabel::Scene(0).from_asset(path))
    };

    let residential_files = [
        "a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k", "l", "m", "n", "o", "p", "q", "r",
        "s", "t", "u",
    ];
    let residential: Vec<Handle<Scene>> = residential_files
        .iter()
        .map(|letter| {
            load_scene(format!(
                "models/buildings/residential/building-type-{letter}.glb"
            ))
        })
        .collect();

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
    let skyscrapers: Vec<Handle<Scene>> = skyscraper_files
        .iter()
        .map(|letter| {
            load_scene(format!(
                "models/buildings/skyscrapers/building-skyscraper-{letter}.glb"
            ))
        })
        .collect();

    let industrial_files = [
        "a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k", "l", "m", "n", "o", "p", "q", "r",
        "s", "t",
    ];
    let industrial: Vec<Handle<Scene>> = industrial_files
        .iter()
        .map(|letter| load_scene(format!("models/buildings/industrial/building-{letter}.glb")))
        .collect();

    let vehicle_files = [
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
    let vehicles: Vec<Handle<Scene>> = vehicle_files
        .iter()
        .map(|name| load_scene(format!("models/vehicles/{name}.glb")))
        .collect();

    let character_files = [
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
    let characters: Vec<Handle<Scene>> = character_files
        .iter()
        .map(|name| load_scene(format!("models/characters/{name}.glb")))
        .collect();

    let tree_files = [
        "tree-suburban",
        "tree-retro-large",
        "tree-park-large",
        "tree-park-pine-large",
    ];
    let mut trees: Vec<Handle<Scene>> = tree_files
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

    let prop_files = [
        "detail-bench",
        "detail-light-single",
        "detail-light-double",
        "detail-barrier-type-a",
        "detail-dumpster-closed",
        "planter",
    ];
    let props: Vec<Handle<Scene>> = prop_files
        .iter()
        .map(|name| load_scene(format!("models/props/{name}.glb")))
        .collect();

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
        service_meshes: HashMap::new(),
        utility_meshes: HashMap::new(),
        fallback_material,
    });
}

// ---------------------------------------------------------------------------
// Building scale
// ---------------------------------------------------------------------------

/// Get the scale factor for a building based on zone type and level.
/// Kenney city-kit models are ~1-3 units wide. CELL_SIZE = 16 units.
/// Buildings should fill their cell in dense areas for a Manhattan-like look.
pub fn building_scale(zone: ZoneType, level: u8) -> f32 {
    let base: f32 = 7.5;
    match (zone, level) {
        (ZoneType::ResidentialLow, 1) => base * 0.65,
        (ZoneType::ResidentialLow, 2) => base * 0.75,
        (ZoneType::ResidentialLow, _) => base * 0.85,

        (ZoneType::ResidentialMedium, 1) => base * 0.75,
        (ZoneType::ResidentialMedium, 2) => base * 0.9,
        (ZoneType::ResidentialMedium, 3) => base * 1.05,
        (ZoneType::ResidentialMedium, _) => base * 1.2,

        (ZoneType::ResidentialHigh, 1) => base * 0.85,
        (ZoneType::ResidentialHigh, 2) => base * 1.0,
        (ZoneType::ResidentialHigh, 3) => base * 1.15,
        (ZoneType::ResidentialHigh, 4) => base * 1.3,
        (ZoneType::ResidentialHigh, _) => base * 1.5,

        (ZoneType::CommercialLow, 1) => base * 0.75,
        (ZoneType::CommercialLow, 2) => base * 0.9,
        (ZoneType::CommercialLow, _) => base * 1.0,

        (ZoneType::CommercialHigh, 1) => base * 0.9,
        (ZoneType::CommercialHigh, 2) => base * 1.1,
        (ZoneType::CommercialHigh, 3) => base * 1.3,
        (ZoneType::CommercialHigh, 4) => base * 1.5,
        (ZoneType::CommercialHigh, _) => base * 1.8,

        (ZoneType::Industrial, 1) => base * 0.8,
        (ZoneType::Industrial, 2) => base * 0.95,
        (ZoneType::Industrial, 3) => base * 1.1,
        (ZoneType::Industrial, _) => base * 1.2,

        (ZoneType::Office, 1) => base * 0.85,
        (ZoneType::Office, 2) => base * 1.05,
        (ZoneType::Office, 3) => base * 1.3,
        (ZoneType::Office, 4) => base * 1.5,
        (ZoneType::Office, _) => base * 1.8,

        (ZoneType::MixedUse, 1) => base * 0.8,
        (ZoneType::MixedUse, 2) => base * 1.0,
        (ZoneType::MixedUse, 3) => base * 1.2,
        (ZoneType::MixedUse, 4) => base * 1.4,
        (ZoneType::MixedUse, _) => base * 1.6,

        _ => base,
    }
}
