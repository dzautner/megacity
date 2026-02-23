// ---------------------------------------------------------------------------
// Grid-related codecs: ZoneType, RoadType
// ---------------------------------------------------------------------------

use simulation::grid::{RoadType, ZoneType};

pub fn zone_type_to_u8(z: ZoneType) -> u8 {
    match z {
        ZoneType::None => 0,
        ZoneType::ResidentialLow => 1,
        ZoneType::ResidentialHigh => 2,
        ZoneType::CommercialLow => 3,
        ZoneType::CommercialHigh => 4,
        ZoneType::Industrial => 5,
        ZoneType::Office => 6,
        ZoneType::ResidentialMedium => 7,
        ZoneType::MixedUse => 8,
    }
}

pub fn u8_to_zone_type(v: u8) -> ZoneType {
    match v {
        1 => ZoneType::ResidentialLow,
        2 => ZoneType::ResidentialHigh,
        3 => ZoneType::CommercialLow,
        4 => ZoneType::CommercialHigh,
        5 => ZoneType::Industrial,
        6 => ZoneType::Office,
        7 => ZoneType::ResidentialMedium,
        8 => ZoneType::MixedUse,
        _ => ZoneType::None,
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
