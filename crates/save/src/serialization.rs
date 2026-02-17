use bitcode::{Decode, Encode};
use serde::{Deserialize, Serialize};

use simulation::buildings::Building;
use simulation::citizen::{CitizenDetails, CitizenState};
use simulation::economy::CityBudget;
use simulation::grid::{RoadType, WorldGrid};
use simulation::road_segments::{
    RoadSegment, RoadSegmentStore, SegmentId, SegmentNode, SegmentNodeId,
};
use simulation::roads::RoadNetwork;
use simulation::services::{ServiceBuilding, ServiceType};
use simulation::time_of_day::GameClock;
use simulation::utilities::{UtilitySource, UtilityType};
use simulation::zones::ZoneDemand;

// ---------------------------------------------------------------------------
// Encoding helpers
// ---------------------------------------------------------------------------

pub fn zone_type_to_u8(z: simulation::grid::ZoneType) -> u8 {
    match z {
        simulation::grid::ZoneType::None => 0,
        simulation::grid::ZoneType::ResidentialLow => 1,
        simulation::grid::ZoneType::ResidentialHigh => 2,
        simulation::grid::ZoneType::CommercialLow => 3,
        simulation::grid::ZoneType::CommercialHigh => 4,
        simulation::grid::ZoneType::Industrial => 5,
        simulation::grid::ZoneType::Office => 6,
    }
}

pub fn u8_to_zone_type(v: u8) -> simulation::grid::ZoneType {
    match v {
        1 => simulation::grid::ZoneType::ResidentialLow,
        2 => simulation::grid::ZoneType::ResidentialHigh,
        3 => simulation::grid::ZoneType::CommercialLow,
        4 => simulation::grid::ZoneType::CommercialHigh,
        5 => simulation::grid::ZoneType::Industrial,
        6 => simulation::grid::ZoneType::Office,
        _ => simulation::grid::ZoneType::None,
    }
}

pub fn utility_type_to_u8(u: UtilityType) -> u8 {
    match u {
        UtilityType::PowerPlant => 0,
        UtilityType::SolarFarm => 1,
        UtilityType::WindTurbine => 2,
        UtilityType::WaterTower => 3,
        UtilityType::SewagePlant => 4,
        UtilityType::NuclearPlant => 5,
        UtilityType::Geothermal => 6,
        UtilityType::PumpingStation => 7,
        UtilityType::WaterTreatment => 8,
    }
}

pub fn u8_to_utility_type(v: u8) -> UtilityType {
    match v {
        0 => UtilityType::PowerPlant,
        1 => UtilityType::SolarFarm,
        2 => UtilityType::WindTurbine,
        3 => UtilityType::WaterTower,
        4 => UtilityType::SewagePlant,
        5 => UtilityType::NuclearPlant,
        6 => UtilityType::Geothermal,
        7 => UtilityType::PumpingStation,
        8 => UtilityType::WaterTreatment,
        _ => UtilityType::PowerPlant, // fallback
    }
}

pub fn service_type_to_u8(s: ServiceType) -> u8 {
    match s {
        ServiceType::FireStation => 0,
        ServiceType::PoliceStation => 1,
        ServiceType::Hospital => 2,
        ServiceType::ElementarySchool => 3,
        ServiceType::HighSchool => 4,
        ServiceType::University => 5,
        ServiceType::Library => 6,
        ServiceType::SmallPark => 7,
        ServiceType::LargePark => 8,
        ServiceType::Playground => 9,
        ServiceType::Plaza => 10,
        ServiceType::SportsField => 11,
        ServiceType::Stadium => 12,
        ServiceType::Landfill => 13,
        ServiceType::RecyclingCenter => 14,
        ServiceType::Incinerator => 15,
        ServiceType::Cemetery => 16,
        ServiceType::Crematorium => 17,
        ServiceType::CityHall => 18,
        ServiceType::Museum => 19,
        ServiceType::Cathedral => 20,
        ServiceType::TVStation => 21,
        ServiceType::BusDepot => 22,
        ServiceType::TrainStation => 23,
        ServiceType::FireHouse => 24,
        ServiceType::FireHQ => 25,
        ServiceType::PoliceKiosk => 26,
        ServiceType::PoliceHQ => 27,
        ServiceType::Prison => 28,
        ServiceType::MedicalClinic => 29,
        ServiceType::MedicalCenter => 30,
        ServiceType::Kindergarten => 31,
        ServiceType::SubwayStation => 32,
        ServiceType::TramDepot => 33,
        ServiceType::FerryPier => 34,
        ServiceType::SmallAirport => 35,
        ServiceType::InternationalAirport => 36,
        ServiceType::TransferStation => 37,
        ServiceType::CellTower => 38,
        ServiceType::DataCenter => 39,
    }
}

pub fn u8_to_service_type(v: u8) -> Option<ServiceType> {
    match v {
        0 => Some(ServiceType::FireStation),
        1 => Some(ServiceType::PoliceStation),
        2 => Some(ServiceType::Hospital),
        3 => Some(ServiceType::ElementarySchool),
        4 => Some(ServiceType::HighSchool),
        5 => Some(ServiceType::University),
        6 => Some(ServiceType::Library),
        7 => Some(ServiceType::SmallPark),
        8 => Some(ServiceType::LargePark),
        9 => Some(ServiceType::Playground),
        10 => Some(ServiceType::Plaza),
        11 => Some(ServiceType::SportsField),
        12 => Some(ServiceType::Stadium),
        13 => Some(ServiceType::Landfill),
        14 => Some(ServiceType::RecyclingCenter),
        15 => Some(ServiceType::Incinerator),
        16 => Some(ServiceType::Cemetery),
        17 => Some(ServiceType::Crematorium),
        18 => Some(ServiceType::CityHall),
        19 => Some(ServiceType::Museum),
        20 => Some(ServiceType::Cathedral),
        21 => Some(ServiceType::TVStation),
        22 => Some(ServiceType::BusDepot),
        23 => Some(ServiceType::TrainStation),
        24 => Some(ServiceType::FireHouse),
        25 => Some(ServiceType::FireHQ),
        26 => Some(ServiceType::PoliceKiosk),
        27 => Some(ServiceType::PoliceHQ),
        28 => Some(ServiceType::Prison),
        29 => Some(ServiceType::MedicalClinic),
        30 => Some(ServiceType::MedicalCenter),
        31 => Some(ServiceType::Kindergarten),
        32 => Some(ServiceType::SubwayStation),
        33 => Some(ServiceType::TramDepot),
        34 => Some(ServiceType::FerryPier),
        35 => Some(ServiceType::SmallAirport),
        36 => Some(ServiceType::InternationalAirport),
        37 => Some(ServiceType::TransferStation),
        38 => Some(ServiceType::CellTower),
        39 => Some(ServiceType::DataCenter),
        _ => None,
    }
}

pub fn road_type_to_u8(r: RoadType) -> u8 {
    match r {
        RoadType::Local => 0,
        RoadType::Avenue => 1,
        RoadType::Boulevard => 2,
        RoadType::Highway => 3,
        RoadType::OneWay => 4,
        RoadType::Path => 5,
    }
}

pub fn u8_to_road_type(v: u8) -> RoadType {
    match v {
        0 => RoadType::Local,
        1 => RoadType::Avenue,
        2 => RoadType::Boulevard,
        3 => RoadType::Highway,
        4 => RoadType::OneWay,
        5 => RoadType::Path,
        _ => RoadType::Local,
    }
}

// ---------------------------------------------------------------------------
// Save structs
// ---------------------------------------------------------------------------

#[derive(Serialize, Deserialize, Encode, Decode)]
pub struct SaveSegmentNode {
    pub id: u32,
    pub x: f32,
    pub y: f32,
    pub connected_segments: Vec<u32>,
}

#[derive(Serialize, Deserialize, Encode, Decode)]
pub struct SaveRoadSegment {
    pub id: u32,
    pub start_node: u32,
    pub end_node: u32,
    pub p0_x: f32,
    pub p0_y: f32,
    pub p1_x: f32,
    pub p1_y: f32,
    pub p2_x: f32,
    pub p2_y: f32,
    pub p3_x: f32,
    pub p3_y: f32,
    pub road_type: u8,
}

#[derive(Serialize, Deserialize, Encode, Decode, Default)]
pub struct SaveRoadSegmentStore {
    pub nodes: Vec<SaveSegmentNode>,
    pub segments: Vec<SaveRoadSegment>,
}

#[derive(Serialize, Deserialize, Encode, Decode)]
pub struct SaveData {
    pub grid: SaveGrid,
    pub roads: SaveRoadNetwork,
    pub clock: SaveClock,
    pub budget: SaveBudget,
    pub demand: SaveDemand,
    pub buildings: Vec<SaveBuilding>,
    pub citizens: Vec<SaveCitizen>,
    pub utility_sources: Vec<SaveUtilitySource>,
    pub service_buildings: Vec<SaveServiceBuilding>,
    #[serde(default)]
    pub road_segments: Option<SaveRoadSegmentStore>,
}

#[derive(Serialize, Deserialize, Encode, Decode)]
pub struct SaveGrid {
    pub cells: Vec<SaveCell>,
    pub width: usize,
    pub height: usize,
}

#[derive(Serialize, Deserialize, Encode, Decode)]
pub struct SaveCell {
    pub elevation: f32,
    pub cell_type: u8,
    pub zone: u8,
    pub road_type: u8,
    pub has_power: bool,
    pub has_water: bool,
}

#[derive(Serialize, Deserialize, Encode, Decode)]
pub struct SaveRoadNetwork {
    pub road_positions: Vec<(usize, usize)>,
}

#[derive(Serialize, Deserialize, Encode, Decode)]
pub struct SaveClock {
    pub day: u32,
    pub hour: f32,
    pub speed: f32,
}

#[derive(Serialize, Deserialize, Encode, Decode)]
pub struct SaveBudget {
    pub treasury: f64,
    pub tax_rate: f32,
    pub last_collection_day: u32,
}

#[derive(Serialize, Deserialize, Encode, Decode)]
pub struct SaveDemand {
    pub residential: f32,
    pub commercial: f32,
    pub industrial: f32,
    pub office: f32,
}

#[derive(Serialize, Deserialize, Encode, Decode)]
pub struct SaveBuilding {
    pub zone_type: u8,
    pub level: u8,
    pub grid_x: usize,
    pub grid_y: usize,
    pub capacity: u32,
    pub occupants: u32,
}

#[derive(Serialize, Deserialize, Encode, Decode)]
pub struct SaveCitizen {
    pub age: u8,
    pub happiness: f32,
    pub education: u8,
    pub state: u8,
    pub home_x: usize,
    pub home_y: usize,
    pub work_x: usize,
    pub work_y: usize,
}

#[derive(Serialize, Deserialize, Encode, Decode)]
pub struct SaveUtilitySource {
    pub utility_type: u8,
    pub grid_x: usize,
    pub grid_y: usize,
    pub range: u32,
}

#[derive(Serialize, Deserialize, Encode, Decode)]
pub struct SaveServiceBuilding {
    pub service_type: u8,
    pub grid_x: usize,
    pub grid_y: usize,
    pub radius_cells: u32,
}

impl SaveData {
    pub fn encode(&self) -> Vec<u8> {
        bitcode::encode(self)
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, bitcode::Error> {
        bitcode::decode(bytes)
    }
}

#[allow(clippy::too_many_arguments)]
pub fn create_save_data(
    grid: &WorldGrid,
    roads: &RoadNetwork,
    clock: &GameClock,
    budget: &CityBudget,
    demand: &ZoneDemand,
    buildings: &[(Building,)],
    citizens: &[(CitizenDetails, CitizenState, usize, usize, usize, usize)],
    utility_sources: &[UtilitySource],
    service_buildings: &[(ServiceBuilding,)],
    segment_store: Option<&RoadSegmentStore>,
) -> SaveData {
    let save_cells: Vec<SaveCell> = grid
        .cells
        .iter()
        .map(|c| SaveCell {
            elevation: c.elevation,
            cell_type: match c.cell_type {
                simulation::grid::CellType::Grass => 0,
                simulation::grid::CellType::Water => 1,
                simulation::grid::CellType::Road => 2,
            },
            zone: zone_type_to_u8(c.zone),
            road_type: road_type_to_u8(c.road_type),
            has_power: c.has_power,
            has_water: c.has_water,
        })
        .collect();

    SaveData {
        grid: SaveGrid {
            cells: save_cells,
            width: grid.width,
            height: grid.height,
        },
        roads: SaveRoadNetwork {
            road_positions: roads
                .edges
                .keys()
                .map(|n| (n.0, n.1))
                .collect(),
        },
        clock: SaveClock {
            day: clock.day,
            hour: clock.hour,
            speed: clock.speed,
        },
        budget: SaveBudget {
            treasury: budget.treasury,
            tax_rate: budget.tax_rate,
            last_collection_day: budget.last_collection_day,
        },
        demand: SaveDemand {
            residential: demand.residential,
            commercial: demand.commercial,
            industrial: demand.industrial,
            office: demand.office,
        },
        buildings: buildings
            .iter()
            .map(|(b,)| SaveBuilding {
                zone_type: zone_type_to_u8(b.zone_type),
                level: b.level,
                grid_x: b.grid_x,
                grid_y: b.grid_y,
                capacity: b.capacity,
                occupants: b.occupants,
            })
            .collect(),
        citizens: citizens
            .iter()
            .map(|(d, state, hx, hy, wx, wy)| SaveCitizen {
                age: d.age,
                happiness: d.happiness,
                education: d.education,
                state: match state {
                    CitizenState::AtHome => 0,
                    CitizenState::CommutingToWork => 1,
                    CitizenState::Working => 2,
                    CitizenState::CommutingHome => 3,
                    CitizenState::CommutingToShop => 4,
                    CitizenState::Shopping => 5,
                    CitizenState::CommutingToLeisure => 6,
                    CitizenState::AtLeisure => 7,
                    CitizenState::CommutingToSchool => 8,
                    CitizenState::AtSchool => 9,
                },
                home_x: *hx,
                home_y: *hy,
                work_x: *wx,
                work_y: *wy,
            })
            .collect(),
        utility_sources: utility_sources
            .iter()
            .map(|u| SaveUtilitySource {
                utility_type: utility_type_to_u8(u.utility_type),
                grid_x: u.grid_x,
                grid_y: u.grid_y,
                range: u.range,
            })
            .collect(),
        service_buildings: service_buildings
            .iter()
            .map(|(sb,)| SaveServiceBuilding {
                service_type: service_type_to_u8(sb.service_type),
                grid_x: sb.grid_x,
                grid_y: sb.grid_y,
                radius_cells: (sb.radius / simulation::config::CELL_SIZE) as u32,
            })
            .collect(),
        road_segments: segment_store.map(|store| SaveRoadSegmentStore {
            nodes: store
                .nodes
                .iter()
                .map(|n| SaveSegmentNode {
                    id: n.id.0,
                    x: n.position.x,
                    y: n.position.y,
                    connected_segments: n.connected_segments.iter().map(|s| s.0).collect(),
                })
                .collect(),
            segments: store
                .segments
                .iter()
                .map(|s| SaveRoadSegment {
                    id: s.id.0,
                    start_node: s.start_node.0,
                    end_node: s.end_node.0,
                    p0_x: s.p0.x,
                    p0_y: s.p0.y,
                    p1_x: s.p1.x,
                    p1_y: s.p1.y,
                    p2_x: s.p2.x,
                    p2_y: s.p2.y,
                    p3_x: s.p3.x,
                    p3_y: s.p3.y,
                    road_type: road_type_to_u8(s.road_type),
                })
                .collect(),
        }),
    }
}

/// Reconstruct a `RoadSegmentStore` from saved data.
/// After calling this, call `store.rasterize_all(grid, roads)` to rebuild grid cells.
pub fn restore_road_segment_store(save: &SaveRoadSegmentStore) -> RoadSegmentStore {
    use bevy::math::Vec2;

    let nodes: Vec<SegmentNode> = save
        .nodes
        .iter()
        .map(|n| SegmentNode {
            id: SegmentNodeId(n.id),
            position: Vec2::new(n.x, n.y),
            connected_segments: n.connected_segments.iter().map(|&s| SegmentId(s)).collect(),
        })
        .collect();

    let segments: Vec<RoadSegment> = save
        .segments
        .iter()
        .map(|s| RoadSegment {
            id: SegmentId(s.id),
            start_node: SegmentNodeId(s.start_node),
            end_node: SegmentNodeId(s.end_node),
            p0: Vec2::new(s.p0_x, s.p0_y),
            p1: Vec2::new(s.p1_x, s.p1_y),
            p2: Vec2::new(s.p2_x, s.p2_y),
            p3: Vec2::new(s.p3_x, s.p3_y),
            road_type: u8_to_road_type(s.road_type),
            arc_length: 0.0,
            rasterized_cells: Vec::new(),
        })
        .collect();

    RoadSegmentStore::from_parts(nodes, segments)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_roundtrip_serialization() {
        let mut grid = WorldGrid::new(16, 16);
        simulation::terrain::generate_terrain(&mut grid, 42);

        // Set some zones to test the new types
        grid.get_mut(5, 5).zone = simulation::grid::ZoneType::ResidentialLow;
        grid.get_mut(6, 6).zone = simulation::grid::ZoneType::ResidentialHigh;
        grid.get_mut(7, 7).zone = simulation::grid::ZoneType::CommercialLow;
        grid.get_mut(8, 8).zone = simulation::grid::ZoneType::CommercialHigh;
        grid.get_mut(9, 9).zone = simulation::grid::ZoneType::Office;

        let roads = RoadNetwork::default();
        let clock = GameClock::default();
        let budget = CityBudget::default();
        let demand = ZoneDemand::default();

        let save = create_save_data(&grid, &roads, &clock, &budget, &demand, &[], &[], &[], &[], None);
        let bytes = save.encode();
        let restored = SaveData::decode(&bytes).expect("decode should succeed");

        assert_eq!(restored.grid.width, 16);
        assert_eq!(restored.grid.height, 16);
        assert_eq!(restored.grid.cells.len(), 256);
        assert_eq!(restored.clock.day, clock.day);
        assert!((restored.budget.treasury - budget.treasury).abs() < 0.01);

        // Verify zone roundtrip
        let idx55 = 5 * 16 + 5;
        assert_eq!(restored.grid.cells[idx55].zone, 1); // ResidentialLow
        let idx66 = 6 * 16 + 6;
        assert_eq!(restored.grid.cells[idx66].zone, 2); // ResidentialHigh
        let idx77 = 7 * 16 + 7;
        assert_eq!(restored.grid.cells[idx77].zone, 3); // CommercialLow
        let idx88 = 8 * 16 + 8;
        assert_eq!(restored.grid.cells[idx88].zone, 4); // CommercialHigh
        let idx99 = 9 * 16 + 9;
        assert_eq!(restored.grid.cells[idx99].zone, 6); // Office
    }

    #[test]
    fn test_zone_type_roundtrip() {
        use simulation::grid::ZoneType;
        let types = [
            ZoneType::None,
            ZoneType::ResidentialLow,
            ZoneType::ResidentialHigh,
            ZoneType::CommercialLow,
            ZoneType::CommercialHigh,
            ZoneType::Industrial,
            ZoneType::Office,
        ];
        for zt in &types {
            let encoded = zone_type_to_u8(*zt);
            let decoded = u8_to_zone_type(encoded);
            assert_eq!(*zt, decoded);
        }
    }

    #[test]
    fn test_utility_type_roundtrip() {
        let types = [
            UtilityType::PowerPlant,
            UtilityType::SolarFarm,
            UtilityType::WindTurbine,
            UtilityType::WaterTower,
            UtilityType::SewagePlant,
        ];
        for ut in &types {
            let encoded = utility_type_to_u8(*ut);
            let decoded = u8_to_utility_type(encoded);
            assert_eq!(*ut, decoded);
        }
    }

    #[test]
    fn test_service_type_roundtrip() {
        for i in 0..40u8 {
            let st = u8_to_service_type(i).expect("valid service type");
            let encoded = service_type_to_u8(st);
            assert_eq!(i, encoded);
        }
        assert!(u8_to_service_type(40).is_none());
    }
}
