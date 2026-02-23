//! Placement-oriented shortcut categories: roads, zones, utilities, emergency,
//! education, parks, landmarks, sanitation, transport, and telecom.

use bevy::prelude::*;
use rendering::input::ActiveTool;

use super::{ShortcutCategory, ShortcutItem};

pub(super) fn placement_categories() -> Vec<ShortcutCategory> {
    vec![
        ShortcutCategory {
            key: KeyCode::KeyR,
            label: "Roads",
            key_hint: "R",
            items: vec![
                ShortcutItem {
                    name: "Local Road",
                    tool: Some(ActiveTool::Road),
                    overlay: None,
                },
                ShortcutItem {
                    name: "Avenue",
                    tool: Some(ActiveTool::RoadAvenue),
                    overlay: None,
                },
                ShortcutItem {
                    name: "Boulevard",
                    tool: Some(ActiveTool::RoadBoulevard),
                    overlay: None,
                },
                ShortcutItem {
                    name: "Highway",
                    tool: Some(ActiveTool::RoadHighway),
                    overlay: None,
                },
                ShortcutItem {
                    name: "One-Way",
                    tool: Some(ActiveTool::RoadOneWay),
                    overlay: None,
                },
                ShortcutItem {
                    name: "Path",
                    tool: Some(ActiveTool::RoadPath),
                    overlay: None,
                },
            ],
        },
        ShortcutCategory {
            key: KeyCode::KeyZ,
            label: "Zones",
            key_hint: "Z",
            items: vec![
                ShortcutItem {
                    name: "Res Low",
                    tool: Some(ActiveTool::ZoneResidentialLow),
                    overlay: None,
                },
                ShortcutItem {
                    name: "Res Medium",
                    tool: Some(ActiveTool::ZoneResidentialMedium),
                    overlay: None,
                },
                ShortcutItem {
                    name: "Res High",
                    tool: Some(ActiveTool::ZoneResidentialHigh),
                    overlay: None,
                },
                ShortcutItem {
                    name: "Com Low",
                    tool: Some(ActiveTool::ZoneCommercialLow),
                    overlay: None,
                },
                ShortcutItem {
                    name: "Com High",
                    tool: Some(ActiveTool::ZoneCommercialHigh),
                    overlay: None,
                },
                ShortcutItem {
                    name: "Industrial",
                    tool: Some(ActiveTool::ZoneIndustrial),
                    overlay: None,
                },
                ShortcutItem {
                    name: "Office",
                    tool: Some(ActiveTool::ZoneOffice),
                    overlay: None,
                },
                ShortcutItem {
                    name: "Mixed-Use",
                    tool: Some(ActiveTool::ZoneMixedUse),
                    overlay: None,
                },
            ],
        },
        ShortcutCategory {
            key: KeyCode::KeyU,
            label: "Utilities",
            key_hint: "U",
            items: vec![
                ShortcutItem {
                    name: "Power Plant",
                    tool: Some(ActiveTool::PlacePowerPlant),
                    overlay: None,
                },
                ShortcutItem {
                    name: "Solar Farm",
                    tool: Some(ActiveTool::PlaceSolarFarm),
                    overlay: None,
                },
                ShortcutItem {
                    name: "Wind Turbine",
                    tool: Some(ActiveTool::PlaceWindTurbine),
                    overlay: None,
                },
                ShortcutItem {
                    name: "Nuclear Plant",
                    tool: Some(ActiveTool::PlaceNuclearPlant),
                    overlay: None,
                },
                ShortcutItem {
                    name: "Geothermal",
                    tool: Some(ActiveTool::PlaceGeothermal),
                    overlay: None,
                },
                ShortcutItem {
                    name: "Water Tower",
                    tool: Some(ActiveTool::PlaceWaterTower),
                    overlay: None,
                },
                ShortcutItem {
                    name: "Sewage Plant",
                    tool: Some(ActiveTool::PlaceSewagePlant),
                    overlay: None,
                },
                ShortcutItem {
                    name: "Pumping Station",
                    tool: Some(ActiveTool::PlacePumpingStation),
                    overlay: None,
                },
                ShortcutItem {
                    name: "Water Treatment",
                    tool: Some(ActiveTool::PlaceWaterTreatment),
                    overlay: None,
                },
            ],
        },
        ShortcutCategory {
            key: KeyCode::KeyE,
            label: "Emergency",
            key_hint: "E",
            items: vec![
                ShortcutItem {
                    name: "Fire House",
                    tool: Some(ActiveTool::PlaceFireHouse),
                    overlay: None,
                },
                ShortcutItem {
                    name: "Fire Station",
                    tool: Some(ActiveTool::PlaceFireStation),
                    overlay: None,
                },
                ShortcutItem {
                    name: "Fire HQ",
                    tool: Some(ActiveTool::PlaceFireHQ),
                    overlay: None,
                },
                ShortcutItem {
                    name: "Police Kiosk",
                    tool: Some(ActiveTool::PlacePoliceKiosk),
                    overlay: None,
                },
                ShortcutItem {
                    name: "Police Station",
                    tool: Some(ActiveTool::PlacePoliceStation),
                    overlay: None,
                },
                ShortcutItem {
                    name: "Police HQ",
                    tool: Some(ActiveTool::PlacePoliceHQ),
                    overlay: None,
                },
                ShortcutItem {
                    name: "Prison",
                    tool: Some(ActiveTool::PlacePrison),
                    overlay: None,
                },
                ShortcutItem {
                    name: "Medical Clinic",
                    tool: Some(ActiveTool::PlaceMedicalClinic),
                    overlay: None,
                },
                ShortcutItem {
                    name: "Hospital",
                    tool: Some(ActiveTool::PlaceHospital),
                    overlay: None,
                },
                ShortcutItem {
                    name: "Medical Center",
                    tool: Some(ActiveTool::PlaceMedicalCenter),
                    overlay: None,
                },
            ],
        },
        ShortcutCategory {
            key: KeyCode::KeyS,
            label: "Education",
            key_hint: "S",
            items: vec![
                ShortcutItem {
                    name: "Kindergarten",
                    tool: Some(ActiveTool::PlaceKindergarten),
                    overlay: None,
                },
                ShortcutItem {
                    name: "Elementary",
                    tool: Some(ActiveTool::PlaceElementarySchool),
                    overlay: None,
                },
                ShortcutItem {
                    name: "High School",
                    tool: Some(ActiveTool::PlaceHighSchool),
                    overlay: None,
                },
                ShortcutItem {
                    name: "University",
                    tool: Some(ActiveTool::PlaceUniversity),
                    overlay: None,
                },
                ShortcutItem {
                    name: "Library",
                    tool: Some(ActiveTool::PlaceLibrary),
                    overlay: None,
                },
            ],
        },
        ShortcutCategory {
            key: KeyCode::KeyK,
            label: "Parks",
            key_hint: "K",
            items: vec![
                ShortcutItem {
                    name: "Small Park",
                    tool: Some(ActiveTool::PlaceSmallPark),
                    overlay: None,
                },
                ShortcutItem {
                    name: "Large Park",
                    tool: Some(ActiveTool::PlaceLargePark),
                    overlay: None,
                },
                ShortcutItem {
                    name: "Playground",
                    tool: Some(ActiveTool::PlacePlayground),
                    overlay: None,
                },
                ShortcutItem {
                    name: "Plaza",
                    tool: Some(ActiveTool::PlacePlaza),
                    overlay: None,
                },
                ShortcutItem {
                    name: "Sports Field",
                    tool: Some(ActiveTool::PlaceSportsField),
                    overlay: None,
                },
                ShortcutItem {
                    name: "Stadium",
                    tool: Some(ActiveTool::PlaceStadium),
                    overlay: None,
                },
            ],
        },
        ShortcutCategory {
            key: KeyCode::KeyL,
            label: "Landmarks",
            key_hint: "L",
            items: vec![
                ShortcutItem {
                    name: "City Hall",
                    tool: Some(ActiveTool::PlaceCityHall),
                    overlay: None,
                },
                ShortcutItem {
                    name: "Museum",
                    tool: Some(ActiveTool::PlaceMuseum),
                    overlay: None,
                },
                ShortcutItem {
                    name: "Cathedral",
                    tool: Some(ActiveTool::PlaceCathedral),
                    overlay: None,
                },
                ShortcutItem {
                    name: "TV Station",
                    tool: Some(ActiveTool::PlaceTVStation),
                    overlay: None,
                },
            ],
        },
        ShortcutCategory {
            key: KeyCode::KeyG,
            label: "Sanitation",
            key_hint: "G",
            items: vec![
                ShortcutItem {
                    name: "Landfill",
                    tool: Some(ActiveTool::PlaceLandfill),
                    overlay: None,
                },
                ShortcutItem {
                    name: "Recycling Center",
                    tool: Some(ActiveTool::PlaceRecyclingCenter),
                    overlay: None,
                },
                ShortcutItem {
                    name: "Incinerator",
                    tool: Some(ActiveTool::PlaceIncinerator),
                    overlay: None,
                },
                ShortcutItem {
                    name: "Transfer Station",
                    tool: Some(ActiveTool::PlaceTransferStation),
                    overlay: None,
                },
                ShortcutItem {
                    name: "Cemetery",
                    tool: Some(ActiveTool::PlaceCemetery),
                    overlay: None,
                },
                ShortcutItem {
                    name: "Crematorium",
                    tool: Some(ActiveTool::PlaceCrematorium),
                    overlay: None,
                },
            ],
        },
        ShortcutCategory {
            key: KeyCode::KeyX,
            label: "Transport",
            key_hint: "X",
            items: vec![
                ShortcutItem {
                    name: "Bus Depot",
                    tool: Some(ActiveTool::PlaceBusDepot),
                    overlay: None,
                },
                ShortcutItem {
                    name: "Train Station",
                    tool: Some(ActiveTool::PlaceTrainStation),
                    overlay: None,
                },
                ShortcutItem {
                    name: "Subway",
                    tool: Some(ActiveTool::PlaceSubwayStation),
                    overlay: None,
                },
                ShortcutItem {
                    name: "Tram Depot",
                    tool: Some(ActiveTool::PlaceTramDepot),
                    overlay: None,
                },
                ShortcutItem {
                    name: "Ferry Pier",
                    tool: Some(ActiveTool::PlaceFerryPier),
                    overlay: None,
                },
                ShortcutItem {
                    name: "Small Airstrip",
                    tool: Some(ActiveTool::PlaceSmallAirstrip),
                    overlay: None,
                },
                ShortcutItem {
                    name: "Regional Airport",
                    tool: Some(ActiveTool::PlaceRegionalAirport),
                    overlay: None,
                },
                ShortcutItem {
                    name: "Int'l Airport",
                    tool: Some(ActiveTool::PlaceInternationalAirport),
                    overlay: None,
                },
            ],
        },
        ShortcutCategory {
            key: KeyCode::KeyN,
            label: "Telecom",
            key_hint: "N",
            items: vec![
                ShortcutItem {
                    name: "Cell Tower",
                    tool: Some(ActiveTool::PlaceCellTower),
                    overlay: None,
                },
                ShortcutItem {
                    name: "Data Center",
                    tool: Some(ActiveTool::PlaceDataCenter),
                    overlay: None,
                },
            ],
        },
    ]
}
