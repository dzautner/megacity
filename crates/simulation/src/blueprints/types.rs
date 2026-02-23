//! Blueprint type definitions and conversions.
//!
//! Contains the serializable mirror types (`BlueprintRoadType`, `BlueprintZoneType`)
//! and their conversions to/from the grid types, plus the data structs for
//! segments, zone cells, and placement results.

use bitcode::{Decode, Encode};

use crate::grid::{RoadType, ZoneType};

// =============================================================================
// Segment & zone-cell data
// =============================================================================

/// A road segment stored relative to the blueprint origin.
#[derive(Debug, Clone, Encode, Decode)]
pub struct BlueprintSegment {
    /// Control points relative to the blueprint origin (world units).
    pub p0: [f32; 2],
    pub p1: [f32; 2],
    pub p2: [f32; 2],
    pub p3: [f32; 2],
    pub road_type: BlueprintRoadType,
}

/// A zone cell stored relative to the blueprint origin.
#[derive(Debug, Clone, Encode, Decode)]
pub struct BlueprintZoneCell {
    /// Offset from the blueprint origin in grid cells.
    pub dx: i32,
    pub dy: i32,
    pub zone_type: BlueprintZoneType,
}

/// Result of placing a blueprint on the map.
#[derive(Debug, Clone, Copy)]
pub struct PlaceResult {
    pub segments_placed: u32,
    pub zones_placed: u32,
}

// =============================================================================
// BlueprintRoadType
// =============================================================================

/// Serializable mirror of `RoadType` for bitcode encoding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Encode, Decode)]
pub enum BlueprintRoadType {
    Local,
    Avenue,
    Boulevard,
    Highway,
    OneWay,
    Path,
}

impl From<RoadType> for BlueprintRoadType {
    fn from(rt: RoadType) -> Self {
        match rt {
            RoadType::Local => BlueprintRoadType::Local,
            RoadType::Avenue => BlueprintRoadType::Avenue,
            RoadType::Boulevard => BlueprintRoadType::Boulevard,
            RoadType::Highway => BlueprintRoadType::Highway,
            RoadType::OneWay => BlueprintRoadType::OneWay,
            RoadType::Path => BlueprintRoadType::Path,
        }
    }
}

impl From<BlueprintRoadType> for RoadType {
    fn from(brt: BlueprintRoadType) -> Self {
        match brt {
            BlueprintRoadType::Local => RoadType::Local,
            BlueprintRoadType::Avenue => RoadType::Avenue,
            BlueprintRoadType::Boulevard => RoadType::Boulevard,
            BlueprintRoadType::Highway => RoadType::Highway,
            BlueprintRoadType::OneWay => RoadType::OneWay,
            BlueprintRoadType::Path => RoadType::Path,
        }
    }
}

// =============================================================================
// BlueprintZoneType
// =============================================================================

/// Serializable mirror of `ZoneType` for bitcode encoding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Encode, Decode)]
pub enum BlueprintZoneType {
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

impl From<ZoneType> for BlueprintZoneType {
    fn from(zt: ZoneType) -> Self {
        match zt {
            ZoneType::None => BlueprintZoneType::None,
            ZoneType::ResidentialLow => BlueprintZoneType::ResidentialLow,
            ZoneType::ResidentialMedium => BlueprintZoneType::ResidentialMedium,
            ZoneType::ResidentialHigh => BlueprintZoneType::ResidentialHigh,
            ZoneType::CommercialLow => BlueprintZoneType::CommercialLow,
            ZoneType::CommercialHigh => BlueprintZoneType::CommercialHigh,
            ZoneType::Industrial => BlueprintZoneType::Industrial,
            ZoneType::Office => BlueprintZoneType::Office,
            ZoneType::MixedUse => BlueprintZoneType::MixedUse,
        }
    }
}

impl From<BlueprintZoneType> for ZoneType {
    fn from(bzt: BlueprintZoneType) -> Self {
        match bzt {
            BlueprintZoneType::None => ZoneType::None,
            BlueprintZoneType::ResidentialLow => ZoneType::ResidentialLow,
            BlueprintZoneType::ResidentialMedium => ZoneType::ResidentialMedium,
            BlueprintZoneType::ResidentialHigh => ZoneType::ResidentialHigh,
            BlueprintZoneType::CommercialLow => ZoneType::CommercialLow,
            BlueprintZoneType::CommercialHigh => ZoneType::CommercialHigh,
            BlueprintZoneType::Industrial => ZoneType::Industrial,
            BlueprintZoneType::Office => ZoneType::Office,
            BlueprintZoneType::MixedUse => ZoneType::MixedUse,
        }
    }
}
