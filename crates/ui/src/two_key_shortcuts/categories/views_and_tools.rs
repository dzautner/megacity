//! Non-placement shortcut categories: overlay views, environment, terrain,
//! districts, and general tools.

use bevy::prelude::*;
use rendering::input::ActiveTool;
use rendering::overlay::OverlayMode;

use super::{ShortcutCategory, ShortcutItem};

pub(super) fn views_and_tools_categories() -> Vec<ShortcutCategory> {
    vec![
        ShortcutCategory {
            key: KeyCode::KeyV,
            label: "Views",
            key_hint: "V",
            items: vec![
                ShortcutItem {
                    name: "Power",
                    tool: None,
                    overlay: Some(OverlayMode::Power),
                },
                ShortcutItem {
                    name: "Water",
                    tool: None,
                    overlay: Some(OverlayMode::Water),
                },
                ShortcutItem {
                    name: "Traffic",
                    tool: None,
                    overlay: Some(OverlayMode::Traffic),
                },
                ShortcutItem {
                    name: "Pollution",
                    tool: None,
                    overlay: Some(OverlayMode::Pollution),
                },
                ShortcutItem {
                    name: "Land Value",
                    tool: None,
                    overlay: Some(OverlayMode::LandValue),
                },
                ShortcutItem {
                    name: "Education",
                    tool: None,
                    overlay: Some(OverlayMode::Education),
                },
                ShortcutItem {
                    name: "Garbage",
                    tool: None,
                    overlay: Some(OverlayMode::Garbage),
                },
                ShortcutItem {
                    name: "Noise",
                    tool: None,
                    overlay: Some(OverlayMode::Noise),
                },
                ShortcutItem {
                    name: "Water Pollution",
                    tool: None,
                    overlay: Some(OverlayMode::WaterPollution),
                },
                ShortcutItem {
                    name: "GW Level",
                    tool: None,
                    overlay: Some(OverlayMode::GroundwaterLevel),
                },
            ],
        },
        ShortcutCategory {
            key: KeyCode::KeyF,
            label: "Environment",
            key_hint: "F",
            items: vec![
                ShortcutItem {
                    name: "Plant Tree",
                    tool: Some(ActiveTool::TreePlant),
                    overlay: None,
                },
                ShortcutItem {
                    name: "Remove Tree",
                    tool: Some(ActiveTool::TreeRemove),
                    overlay: None,
                },
            ],
        },
        ShortcutCategory {
            key: KeyCode::KeyW,
            label: "Terrain",
            key_hint: "W",
            items: vec![
                ShortcutItem {
                    name: "Raise",
                    tool: Some(ActiveTool::TerrainRaise),
                    overlay: None,
                },
                ShortcutItem {
                    name: "Lower",
                    tool: Some(ActiveTool::TerrainLower),
                    overlay: None,
                },
                ShortcutItem {
                    name: "Flatten",
                    tool: Some(ActiveTool::TerrainLevel),
                    overlay: None,
                },
                ShortcutItem {
                    name: "Water",
                    tool: Some(ActiveTool::TerrainWater),
                    overlay: None,
                },
            ],
        },
        ShortcutCategory {
            key: KeyCode::KeyD,
            label: "Districts",
            key_hint: "D",
            items: vec![
                ShortcutItem {
                    name: "Downtown",
                    tool: Some(ActiveTool::DistrictPaint(0)),
                    overlay: None,
                },
                ShortcutItem {
                    name: "Suburbs",
                    tool: Some(ActiveTool::DistrictPaint(1)),
                    overlay: None,
                },
                ShortcutItem {
                    name: "Industrial",
                    tool: Some(ActiveTool::DistrictPaint(2)),
                    overlay: None,
                },
                ShortcutItem {
                    name: "Waterfront",
                    tool: Some(ActiveTool::DistrictPaint(3)),
                    overlay: None,
                },
                ShortcutItem {
                    name: "Historic",
                    tool: Some(ActiveTool::DistrictPaint(4)),
                    overlay: None,
                },
                ShortcutItem {
                    name: "University",
                    tool: Some(ActiveTool::DistrictPaint(5)),
                    overlay: None,
                },
                ShortcutItem {
                    name: "Arts",
                    tool: Some(ActiveTool::DistrictPaint(6)),
                    overlay: None,
                },
                ShortcutItem {
                    name: "Tech Park",
                    tool: Some(ActiveTool::DistrictPaint(7)),
                    overlay: None,
                },
                ShortcutItem {
                    name: "Erase District",
                    tool: Some(ActiveTool::DistrictErase),
                    overlay: None,
                },
            ],
        },
        ShortcutCategory {
            key: KeyCode::KeyT,
            label: "Tools",
            key_hint: "T",
            items: vec![
                ShortcutItem {
                    name: "Bulldoze",
                    tool: Some(ActiveTool::Bulldoze),
                    overlay: None,
                },
                ShortcutItem {
                    name: "Inspect",
                    tool: Some(ActiveTool::Inspect),
                    overlay: None,
                },
            ],
        },
    ]
}
