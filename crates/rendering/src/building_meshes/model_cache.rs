//! `BuildingModelCache` resource: holds pre-loaded GLB scene handles and
//! procedural-mesh fallbacks for every building category.

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

    /// GLB scenes for service buildings (keyed by ServiceType)
    pub service_scenes: HashMap<ServiceType, Handle<Scene>>,
    /// GLB scenes for utility buildings (keyed by UtilityType)
    pub utility_scenes: HashMap<UtilityType, Handle<Scene>>,

    /// Fallback procedural meshes for service/utility buildings without GLB models
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

    /// Get a GLB scene for a service building, if one exists.
    pub fn get_service_scene(&self, service_type: ServiceType) -> Option<Handle<Scene>> {
        self.service_scenes.get(&service_type).cloned()
    }

    /// Get a GLB scene for a utility building, if one exists.
    pub fn get_utility_scene(&self, utility_type: UtilityType) -> Option<Handle<Scene>> {
        self.utility_scenes.get(&utility_type).cloned()
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

/// Scale factor for service/utility GLB scenes.
/// Kenney models are ~1 unit wide; CELL_SIZE = 16 units.
/// Single-cell buildings fill ~80% of a cell; multi-cell buildings scale up.
pub fn service_building_scale(footprint_w: usize, footprint_h: usize) -> f32 {
    let base = CELL_SIZE * 0.8;
    let cells = footprint_w.max(footprint_h) as f32;
    base * cells
}
