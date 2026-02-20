//! Two-key tool shortcuts (UX-014).
//!
//! Press a category key (R for roads, Z for zones, etc.) to open a numbered
//! popup listing the sub-tools for that category.  Then press a digit key
//! (1-9, 0) to select the corresponding sub-tool.  The popup auto-closes
//! after a 2-second timeout or when Escape is pressed.

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use rendering::input::ActiveTool;
use rendering::overlay::{OverlayMode, OverlayState};

use crate::toolbar::OpenCategory;

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

pub struct TwoKeyShortcutPlugin;

impl Plugin for TwoKeyShortcutPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TwoKeyShortcutState>()
            .add_systems(Update, (two_key_input_system, two_key_popup_ui));
    }
}

// ---------------------------------------------------------------------------
// Category definition (mirrors toolbar order with keyboard-friendly keys)
// ---------------------------------------------------------------------------

/// A shortcut category: a trigger key and the list of sub-tools.
struct ShortcutCategory {
    key: KeyCode,
    label: &'static str,
    /// Short string shown in the popup header, e.g. "R" for roads.
    key_hint: &'static str,
    items: Vec<ShortcutItem>,
}

struct ShortcutItem {
    name: &'static str,
    tool: Option<ActiveTool>,
    overlay: Option<OverlayMode>,
}

fn build_shortcut_categories() -> Vec<ShortcutCategory> {
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

// ---------------------------------------------------------------------------
// State resource
// ---------------------------------------------------------------------------

/// Tracks the two-key shortcut state machine.
///
/// When the user presses a category key (e.g. R), `pending_category` is set to
/// the index into `build_shortcut_categories()`.  A timer starts counting down
/// from `TIMEOUT_SECS`.  If a digit key is pressed before the timer expires,
/// the corresponding sub-tool is activated.  If the timer expires or Escape is
/// pressed the pending state is cleared.
#[derive(Resource, Default)]
pub struct TwoKeyShortcutState {
    /// Index into the categories vec, or `None` if no category is pending.
    pub pending_category: Option<usize>,
    /// Remaining seconds before the popup auto-closes.
    pub timer: f32,
}

const TIMEOUT_SECS: f32 = 2.0;

// ---------------------------------------------------------------------------
// Input system
// ---------------------------------------------------------------------------

/// Maps digit key-codes to a 0-based sub-tool index (1->0, 2->1, ..., 9->8, 0->9).
fn digit_key_to_index(key: KeyCode) -> Option<usize> {
    match key {
        KeyCode::Digit1 => Some(0),
        KeyCode::Digit2 => Some(1),
        KeyCode::Digit3 => Some(2),
        KeyCode::Digit4 => Some(3),
        KeyCode::Digit5 => Some(4),
        KeyCode::Digit6 => Some(5),
        KeyCode::Digit7 => Some(6),
        KeyCode::Digit8 => Some(7),
        KeyCode::Digit9 => Some(8),
        KeyCode::Digit0 => Some(9),
        _ => None,
    }
}

/// Handles category-key and digit-key presses, plus timeout and Escape.
#[allow(clippy::too_many_arguments)]
fn two_key_input_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut state: ResMut<TwoKeyShortcutState>,
    mut tool: ResMut<ActiveTool>,
    mut overlay: ResMut<OverlayState>,
    mut open_cat: ResMut<OpenCategory>,
    mut contexts: EguiContexts,
) {
    // Skip when egui wants keyboard (text fields, etc.)
    if contexts.ctx_mut().wants_keyboard_input() {
        return;
    }

    let categories = build_shortcut_categories();

    // --- If a category is already pending, handle sub-key / timeout / escape ---
    if let Some(cat_idx) = state.pending_category {
        // Escape cancels
        if keyboard.just_pressed(KeyCode::Escape) {
            state.pending_category = None;
            state.timer = 0.0;
            return;
        }

        // Check digit keys
        let digit_keys = [
            KeyCode::Digit1,
            KeyCode::Digit2,
            KeyCode::Digit3,
            KeyCode::Digit4,
            KeyCode::Digit5,
            KeyCode::Digit6,
            KeyCode::Digit7,
            KeyCode::Digit8,
            KeyCode::Digit9,
            KeyCode::Digit0,
        ];

        for &dk in &digit_keys {
            if keyboard.just_pressed(dk) {
                if let Some(sub_idx) = digit_key_to_index(dk) {
                    if cat_idx < categories.len() {
                        let cat = &categories[cat_idx];
                        if sub_idx < cat.items.len() {
                            let item = &cat.items[sub_idx];
                            if let Some(t) = item.tool {
                                *tool = t;
                            } else if let Some(ov) = item.overlay {
                                overlay.mode = if overlay.mode == ov {
                                    OverlayMode::None
                                } else {
                                    ov
                                };
                            }
                        }
                    }
                }
                // Close popup after selection (or invalid digit)
                state.pending_category = None;
                state.timer = 0.0;
                return;
            }
        }

        // Pressing a different category key switches category
        for (idx, cat) in categories.iter().enumerate() {
            if keyboard.just_pressed(cat.key) && idx != cat_idx {
                state.pending_category = Some(idx);
                state.timer = TIMEOUT_SECS;
                // Also open the toolbar category popup for consistency
                open_cat.0 = Some(idx);
                return;
            }
        }

        // Timer countdown
        state.timer -= time.delta_secs();
        if state.timer <= 0.0 {
            state.pending_category = None;
            state.timer = 0.0;
        }

        return;
    }

    // --- No pending category: check for category key presses ---
    for (idx, cat) in categories.iter().enumerate() {
        if keyboard.just_pressed(cat.key) {
            state.pending_category = Some(idx);
            state.timer = TIMEOUT_SECS;
            // Also open the matching toolbar category popup for consistency
            open_cat.0 = Some(idx);
            return;
        }
    }
}

// ---------------------------------------------------------------------------
// Popup UI
// ---------------------------------------------------------------------------

/// Draws the numbered sub-tool popup when a category key is pending.
fn two_key_popup_ui(state: Res<TwoKeyShortcutState>, mut contexts: EguiContexts) {
    let cat_idx = match state.pending_category {
        Some(idx) => idx,
        None => return,
    };

    let categories = build_shortcut_categories();
    if cat_idx >= categories.len() {
        return;
    }
    let cat = &categories[cat_idx];

    let screen = contexts.ctx_mut().screen_rect();

    egui::Area::new(egui::Id::new("two_key_shortcut_popup"))
        .fixed_pos(egui::pos2(
            screen.center().x - 120.0,
            screen.center().y - 100.0,
        ))
        .order(egui::Order::Foreground)
        .show(contexts.ctx_mut(), |ui| {
            egui::Frame::popup(ui.style())
                .fill(egui::Color32::from_rgba_premultiplied(30, 30, 40, 240))
                .corner_radius(8.0)
                .inner_margin(12.0)
                .show(ui, |ui| {
                    ui.set_min_width(240.0);

                    // Header with category name and key hint
                    ui.horizontal(|ui| {
                        ui.heading(
                            egui::RichText::new(format!("[{}] {}", cat.key_hint, cat.label))
                                .strong()
                                .color(egui::Color32::from_rgb(140, 200, 255)),
                        );

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            // Timer indicator
                            let pct = (state.timer / TIMEOUT_SECS).clamp(0.0, 1.0);
                            let bar_color = if pct > 0.3 {
                                egui::Color32::from_rgb(100, 200, 100)
                            } else {
                                egui::Color32::from_rgb(220, 100, 60)
                            };
                            let (rect, _) =
                                ui.allocate_exact_size(egui::vec2(40.0, 6.0), egui::Sense::hover());
                            ui.painter()
                                .rect_filled(rect, 3.0, egui::Color32::from_gray(50));
                            let filled = egui::Rect::from_min_size(
                                rect.min,
                                egui::vec2(rect.width() * pct, rect.height()),
                            );
                            ui.painter().rect_filled(filled, 3.0, bar_color);
                        });
                    });

                    ui.separator();

                    // Numbered sub-tool list
                    for (i, item) in cat.items.iter().enumerate() {
                        let number = if i < 9 { i + 1 } else { 0 }; // 1-9, then 0
                        if i >= 10 {
                            break; // max 10 items (digits 1-9 + 0)
                        }

                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new(format!("[{}]", number))
                                    .strong()
                                    .color(egui::Color32::from_rgb(255, 220, 100))
                                    .monospace(),
                            );
                            ui.label(
                                egui::RichText::new(item.name)
                                    .color(egui::Color32::from_rgb(220, 220, 220)),
                            );
                        });
                    }

                    ui.add_space(4.0);
                    ui.label(
                        egui::RichText::new("Press number to select, Esc to cancel")
                            .small()
                            .color(egui::Color32::from_gray(140)),
                    );
                });
        });
}
