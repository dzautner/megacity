//! Serializable action types for the input recorder (STAB-03).
//!
//! Contains `RecordedAction` and bitcode-compatible mirrors of game enums.
//! These types are separate from the game's canonical enums because the
//! game types don't all derive `bitcode::Encode`/`Decode` (e.g. `Vec2`,
//! `SegmentId`).

use bitcode::{Decode, Encode};

use crate::grid::{RoadType, ZoneType};
use crate::services::ServiceType;
use crate::undo_redo::CityAction;
use crate::utilities::UtilityType;

// ---------------------------------------------------------------------------
// RecordedAction
// ---------------------------------------------------------------------------

/// A serializable representation of a player action.
///
/// This mirrors `CityAction` but uses only types that derive `Encode`/`Decode`.
/// Segment-level road operations store raw f32 coordinates and u32 IDs.
#[derive(Debug, Clone, PartialEq, Encode, Decode)]
pub enum RecordedAction {
    /// Road segment placed via the freeform drawing tool.
    PlaceRoadSegment {
        segment_id: u32,
        start_node: u32,
        end_node: u32,
        p0: [f32; 2],
        p1: [f32; 2],
        p2: [f32; 2],
        p3: [f32; 2],
        road_type: RecordedRoadType,
        cost: f64,
    },
    /// Legacy grid-snap road cell placed.
    PlaceGridRoad {
        x: usize,
        y: usize,
        road_type: RecordedRoadType,
        cost: f64,
    },
    /// One or more zone cells painted.
    PlaceZone {
        cells: Vec<(usize, usize, RecordedZoneType)>,
        cost: f64,
    },
    /// Service building placed.
    PlaceService {
        service_type: RecordedServiceType,
        grid_x: usize,
        grid_y: usize,
        cost: f64,
    },
    /// Utility building placed.
    PlaceUtility {
        utility_type: RecordedUtilityType,
        grid_x: usize,
        grid_y: usize,
        cost: f64,
    },
    /// Road cell bulldozed.
    BulldozeRoad {
        x: usize,
        y: usize,
        road_type: RecordedRoadType,
        refund: f64,
    },
    /// Zone cell bulldozed.
    BulldozeZone {
        x: usize,
        y: usize,
        zone: RecordedZoneType,
    },
    /// Service building bulldozed.
    BulldozeService {
        service_type: RecordedServiceType,
        grid_x: usize,
        grid_y: usize,
        refund: f64,
    },
    /// Utility building bulldozed.
    BulldozeUtility {
        utility_type: RecordedUtilityType,
        grid_x: usize,
        grid_y: usize,
        refund: f64,
    },
    /// Composite action (e.g. drag operation).
    Composite(Vec<RecordedAction>),
}

// ---------------------------------------------------------------------------
// Serializable mirrors of game enums
// ---------------------------------------------------------------------------

/// Bitcode-serializable mirror of `RoadType`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Encode, Decode)]
pub enum RecordedRoadType {
    Local,
    Avenue,
    Boulevard,
    Highway,
    OneWay,
    Path,
}

/// Bitcode-serializable mirror of `ZoneType`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Encode, Decode)]
pub enum RecordedZoneType {
    None,
    ResidentialLow,
    ResidentialMedium,
    ResidentialHigh,
    CommercialLow,
    CommercialHigh,
    Industrial,
    Office,
    MixedUse,
}

/// Bitcode-serializable mirror of `ServiceType`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Encode, Decode)]
pub enum RecordedServiceType {
    FireStation,
    PoliceStation,
    Hospital,
    ElementarySchool,
    HighSchool,
    University,
    Library,
    SmallPark,
    LargePark,
    Playground,
    Plaza,
    SportsField,
    Stadium,
    Landfill,
    RecyclingCenter,
    Incinerator,
    Cemetery,
    Crematorium,
    CityHall,
    Museum,
    Cathedral,
    TVStation,
    BusDepot,
    TrainStation,
    FireHouse,
    FireHQ,
    PoliceKiosk,
    PoliceHQ,
    Prison,
    MedicalClinic,
    MedicalCenter,
    Kindergarten,
    SubwayStation,
    TramDepot,
    FerryPier,
    SmallAirstrip,
    RegionalAirport,
    InternationalAirport,
    TransferStation,
    CellTower,
    DataCenter,
    HomelessShelter,
    WelfareOffice,
    PostOffice,
    MailSortingCenter,
    HeatingBoiler,
    DistrictHeatingPlant,
    GeothermalPlant,
    WaterTreatmentPlant,
    WellPump,
    Daycare,
    Eldercare,
    CommunityCenter,
    SubstanceAbuseTreatmentCenter,
    SeniorCenter,
    YouthCenter,
}

/// Bitcode-serializable mirror of `UtilityType`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Encode, Decode)]
pub enum RecordedUtilityType {
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
}

// ---------------------------------------------------------------------------
// Conversion: game types → recorded types
// ---------------------------------------------------------------------------

impl From<RoadType> for RecordedRoadType {
    fn from(rt: RoadType) -> Self {
        match rt {
            RoadType::Local => Self::Local,
            RoadType::Avenue => Self::Avenue,
            RoadType::Boulevard => Self::Boulevard,
            RoadType::Highway => Self::Highway,
            RoadType::OneWay => Self::OneWay,
            RoadType::Path => Self::Path,
        }
    }
}

impl From<ZoneType> for RecordedZoneType {
    fn from(zt: ZoneType) -> Self {
        match zt {
            ZoneType::None => Self::None,
            ZoneType::ResidentialLow => Self::ResidentialLow,
            ZoneType::ResidentialMedium => Self::ResidentialMedium,
            ZoneType::ResidentialHigh => Self::ResidentialHigh,
            ZoneType::CommercialLow => Self::CommercialLow,
            ZoneType::CommercialHigh => Self::CommercialHigh,
            ZoneType::Industrial => Self::Industrial,
            ZoneType::Office => Self::Office,
            ZoneType::MixedUse => Self::MixedUse,
        }
    }
}

impl From<ServiceType> for RecordedServiceType {
    fn from(st: ServiceType) -> Self {
        match st {
            ServiceType::FireStation => Self::FireStation,
            ServiceType::PoliceStation => Self::PoliceStation,
            ServiceType::Hospital => Self::Hospital,
            ServiceType::ElementarySchool => Self::ElementarySchool,
            ServiceType::HighSchool => Self::HighSchool,
            ServiceType::University => Self::University,
            ServiceType::Library => Self::Library,
            ServiceType::SmallPark => Self::SmallPark,
            ServiceType::LargePark => Self::LargePark,
            ServiceType::Playground => Self::Playground,
            ServiceType::Plaza => Self::Plaza,
            ServiceType::SportsField => Self::SportsField,
            ServiceType::Stadium => Self::Stadium,
            ServiceType::Landfill => Self::Landfill,
            ServiceType::RecyclingCenter => Self::RecyclingCenter,
            ServiceType::Incinerator => Self::Incinerator,
            ServiceType::Cemetery => Self::Cemetery,
            ServiceType::Crematorium => Self::Crematorium,
            ServiceType::CityHall => Self::CityHall,
            ServiceType::Museum => Self::Museum,
            ServiceType::Cathedral => Self::Cathedral,
            ServiceType::TVStation => Self::TVStation,
            ServiceType::BusDepot => Self::BusDepot,
            ServiceType::TrainStation => Self::TrainStation,
            ServiceType::FireHouse => Self::FireHouse,
            ServiceType::FireHQ => Self::FireHQ,
            ServiceType::PoliceKiosk => Self::PoliceKiosk,
            ServiceType::PoliceHQ => Self::PoliceHQ,
            ServiceType::Prison => Self::Prison,
            ServiceType::MedicalClinic => Self::MedicalClinic,
            ServiceType::MedicalCenter => Self::MedicalCenter,
            ServiceType::Kindergarten => Self::Kindergarten,
            ServiceType::SubwayStation => Self::SubwayStation,
            ServiceType::TramDepot => Self::TramDepot,
            ServiceType::FerryPier => Self::FerryPier,
            ServiceType::SmallAirstrip => Self::SmallAirstrip,
            ServiceType::RegionalAirport => Self::RegionalAirport,
            ServiceType::InternationalAirport => Self::InternationalAirport,
            ServiceType::TransferStation => Self::TransferStation,
            ServiceType::CellTower => Self::CellTower,
            ServiceType::DataCenter => Self::DataCenter,
            ServiceType::HomelessShelter => Self::HomelessShelter,
            ServiceType::WelfareOffice => Self::WelfareOffice,
            ServiceType::PostOffice => Self::PostOffice,
            ServiceType::MailSortingCenter => Self::MailSortingCenter,
            ServiceType::HeatingBoiler => Self::HeatingBoiler,
            ServiceType::DistrictHeatingPlant => Self::DistrictHeatingPlant,
            ServiceType::GeothermalPlant => Self::GeothermalPlant,
            ServiceType::WaterTreatmentPlant => Self::WaterTreatmentPlant,
            ServiceType::WellPump => Self::WellPump,
            ServiceType::Daycare => Self::Daycare,
            ServiceType::Eldercare => Self::Eldercare,
            ServiceType::CommunityCenter => Self::CommunityCenter,
            ServiceType::SubstanceAbuseTreatmentCenter => {
                Self::SubstanceAbuseTreatmentCenter
            }
            ServiceType::SeniorCenter => Self::SeniorCenter,
            ServiceType::YouthCenter => Self::YouthCenter,
        }
    }
}

impl From<UtilityType> for RecordedUtilityType {
    fn from(ut: UtilityType) -> Self {
        match ut {
            UtilityType::PowerPlant => Self::PowerPlant,
            UtilityType::SolarFarm => Self::SolarFarm,
            UtilityType::WindTurbine => Self::WindTurbine,
            UtilityType::WaterTower => Self::WaterTower,
            UtilityType::SewagePlant => Self::SewagePlant,
            UtilityType::NuclearPlant => Self::NuclearPlant,
            UtilityType::Geothermal => Self::Geothermal,
            UtilityType::PumpingStation => Self::PumpingStation,
            UtilityType::WaterTreatment => Self::WaterTreatment,
            UtilityType::HydroDam => Self::HydroDam,
            UtilityType::OilPlant => Self::OilPlant,
            UtilityType::GasPlant => Self::GasPlant,
        }
    }
}

// ---------------------------------------------------------------------------
// CityAction → RecordedAction conversion
// ---------------------------------------------------------------------------

impl RecordedAction {
    /// Convert a `CityAction` into a `RecordedAction`.
    pub fn from_city_action(action: &CityAction) -> Self {
        match action {
            CityAction::PlaceRoadSegment {
                segment_id,
                start_node,
                end_node,
                p0,
                p1,
                p2,
                p3,
                road_type,
                cost,
                ..
            } => Self::PlaceRoadSegment {
                segment_id: segment_id.0,
                start_node: start_node.0,
                end_node: end_node.0,
                p0: [p0.x, p0.y],
                p1: [p1.x, p1.y],
                p2: [p2.x, p2.y],
                p3: [p3.x, p3.y],
                road_type: (*road_type).into(),
                cost: *cost,
            },
            CityAction::PlaceGridRoad {
                x,
                y,
                road_type,
                cost,
            } => Self::PlaceGridRoad {
                x: *x,
                y: *y,
                road_type: (*road_type).into(),
                cost: *cost,
            },
            CityAction::PlaceZone { cells, cost } => Self::PlaceZone {
                cells: cells
                    .iter()
                    .map(|(x, y, zt)| (*x, *y, (*zt).into()))
                    .collect(),
                cost: *cost,
            },
            CityAction::PlaceService {
                service_type,
                grid_x,
                grid_y,
                cost,
            } => Self::PlaceService {
                service_type: (*service_type).into(),
                grid_x: *grid_x,
                grid_y: *grid_y,
                cost: *cost,
            },
            CityAction::PlaceUtility {
                utility_type,
                grid_x,
                grid_y,
                cost,
            } => Self::PlaceUtility {
                utility_type: (*utility_type).into(),
                grid_x: *grid_x,
                grid_y: *grid_y,
                cost: *cost,
            },
            CityAction::BulldozeRoad {
                x,
                y,
                road_type,
                refund,
            } => Self::BulldozeRoad {
                x: *x,
                y: *y,
                road_type: (*road_type).into(),
                refund: *refund,
            },
            CityAction::BulldozeZone { x, y, zone } => Self::BulldozeZone {
                x: *x,
                y: *y,
                zone: (*zone).into(),
            },
            CityAction::BulldozeService {
                service_type,
                grid_x,
                grid_y,
                refund,
            } => Self::BulldozeService {
                service_type: (*service_type).into(),
                grid_x: *grid_x,
                grid_y: *grid_y,
                refund: *refund,
            },
            CityAction::BulldozeUtility {
                utility_type,
                grid_x,
                grid_y,
                refund,
            } => Self::BulldozeUtility {
                utility_type: (*utility_type).into(),
                grid_x: *grid_x,
                grid_y: *grid_y,
                refund: *refund,
            },
            CityAction::Composite(actions) => {
                Self::Composite(actions.iter().map(Self::from_city_action).collect())
            }
        }
    }
}
