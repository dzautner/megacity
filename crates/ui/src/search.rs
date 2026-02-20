//! UX-044: Search/Filter for Buildings and Citizens.
//!
//! Provides a search bar (toggled via Ctrl+F) to find buildings and citizens.
//! Buildings can be searched by zone type, level, or status (abandoned, under construction).
//! Citizens can be searched by name, age, or occupation (education level).
//! Results are displayed in a scrollable list; clicking a result jumps the camera.

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use rendering::camera::OrbitCamera;
use simulation::abandonment::Abandoned;
use simulation::buildings::{Building, UnderConstruction};
use simulation::citizen::{Citizen, CitizenDetails, Gender, HomeLocation, Position, WorkLocation};
use simulation::config::CELL_SIZE;
use simulation::grid::ZoneType;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Maximum number of results to display per category.
const MAX_RESULTS: usize = 50;

// ---------------------------------------------------------------------------
// Resources
// ---------------------------------------------------------------------------

/// Tracks the search panel state.
#[derive(Resource)]
pub struct SearchState {
    /// Whether the search panel is visible.
    pub visible: bool,
    /// The current search query text.
    pub query: String,
    /// Whether to search buildings.
    pub search_buildings: bool,
    /// Whether to search citizens.
    pub search_citizens: bool,
    /// Cached building results: (entity, zone_label, level, status, grid_x, grid_y).
    pub building_results: Vec<BuildingResult>,
    /// Cached citizen results: (entity, name, age, education_label, grid_x, grid_y).
    pub citizen_results: Vec<CitizenResult>,
    /// Whether results need to be refreshed.
    pub dirty: bool,
    /// Track the previous query to detect changes.
    prev_query: String,
    /// Whether the text field should request focus on the next frame.
    request_focus: bool,
}

#[derive(Clone)]
pub struct BuildingResult {
    pub entity: Entity,
    pub zone_label: String,
    pub level: u8,
    pub status: &'static str,
    pub grid_x: usize,
    pub grid_y: usize,
}

#[derive(Clone)]
pub struct CitizenResult {
    pub entity: Entity,
    pub name: String,
    pub age: u8,
    pub education: &'static str,
    pub grid_x: f32,
    pub grid_y: f32,
}

impl Default for SearchState {
    fn default() -> Self {
        Self {
            visible: false,
            query: String::new(),
            search_buildings: true,
            search_citizens: true,
            building_results: Vec::new(),
            citizen_results: Vec::new(),
            dirty: false,
            prev_query: String::new(),
            request_focus: false,
        }
    }
}

// ---------------------------------------------------------------------------
// Name generation (matching citizen_info.rs)
// ---------------------------------------------------------------------------

const FIRST_NAMES_M: &[&str] = &[
    "James", "John", "Robert", "Michael", "David", "William", "Richard", "Joseph", "Thomas",
    "Daniel", "Matthew", "Anthony", "Mark", "Steven", "Paul", "Andrew", "Joshua", "Kenneth",
    "Kevin", "Brian", "George", "Timothy", "Ronald", "Edward", "Jason", "Jeffrey", "Ryan", "Jacob",
    "Gary", "Nicholas", "Eric", "Jonathan",
];

const FIRST_NAMES_F: &[&str] = &[
    "Mary",
    "Patricia",
    "Jennifer",
    "Linda",
    "Barbara",
    "Elizabeth",
    "Susan",
    "Jessica",
    "Sarah",
    "Karen",
    "Lisa",
    "Nancy",
    "Betty",
    "Margaret",
    "Sandra",
    "Ashley",
    "Dorothy",
    "Kimberly",
    "Emily",
    "Donna",
    "Michelle",
    "Carol",
    "Amanda",
    "Melissa",
    "Deborah",
    "Stephanie",
    "Rebecca",
    "Sharon",
    "Laura",
    "Cynthia",
    "Kathleen",
    "Amy",
];

const LAST_NAMES: &[&str] = &[
    "Smith",
    "Johnson",
    "Williams",
    "Brown",
    "Jones",
    "Garcia",
    "Miller",
    "Davis",
    "Rodriguez",
    "Martinez",
    "Hernandez",
    "Lopez",
    "Gonzalez",
    "Wilson",
    "Anderson",
    "Thomas",
    "Taylor",
    "Moore",
    "Jackson",
    "Martin",
    "Lee",
    "Thompson",
    "White",
    "Harris",
    "Clark",
    "Lewis",
    "Robinson",
    "Walker",
    "Young",
    "Allen",
    "King",
    "Wright",
    "Hill",
];

fn citizen_name(entity: Entity, gender: Gender) -> String {
    let idx = entity.index() as usize;
    let first = match gender {
        Gender::Male => FIRST_NAMES_M[idx % FIRST_NAMES_M.len()],
        Gender::Female => FIRST_NAMES_F[idx % FIRST_NAMES_F.len()],
    };
    let last = LAST_NAMES[(idx / 31) % LAST_NAMES.len()];
    format!("{} {}", first, last)
}

// ---------------------------------------------------------------------------
// Display helpers
// ---------------------------------------------------------------------------

fn zone_label(zone: ZoneType) -> &'static str {
    match zone {
        ZoneType::None => "None",
        ZoneType::ResidentialLow => "Residential (Low)",
        ZoneType::ResidentialMedium => "Residential (Med)",
        ZoneType::ResidentialHigh => "Residential (High)",
        ZoneType::CommercialLow => "Commercial (Low)",
        ZoneType::CommercialHigh => "Commercial (High)",
        ZoneType::Industrial => "Industrial",
        ZoneType::Office => "Office",
        ZoneType::MixedUse => "Mixed Use",
    }
}

fn education_label(education: u8) -> &'static str {
    match education {
        0 => "None",
        1 => "Elementary",
        2 => "High School",
        3 => "University",
        _ => "Advanced",
    }
}

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

/// Toggle search panel visibility with Ctrl+F.
pub fn search_keybind(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut contexts: EguiContexts,
    mut state: ResMut<SearchState>,
) {
    // Don't intercept when egui already wants keyboard (except for our own search field)
    if contexts.ctx_mut().wants_keyboard_input() && !state.visible {
        return;
    }

    let ctrl = keyboard.pressed(KeyCode::ControlLeft) || keyboard.pressed(KeyCode::ControlRight);
    if ctrl && keyboard.just_pressed(KeyCode::KeyF) {
        state.visible = !state.visible;
        if state.visible {
            state.request_focus = true;
            state.dirty = true;
        }
    }

    // Also close on Escape
    if state.visible && keyboard.just_pressed(KeyCode::Escape) {
        state.visible = false;
    }
}

/// Refresh search results when the query changes or the dirty flag is set.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub fn update_search_results(
    mut state: ResMut<SearchState>,
    buildings: Query<(
        Entity,
        &Building,
        Option<&Abandoned>,
        Option<&UnderConstruction>,
    )>,
    citizens: Query<
        (
            Entity,
            &CitizenDetails,
            Option<&Position>,
            Option<&HomeLocation>,
            Option<&WorkLocation>,
        ),
        With<Citizen>,
    >,
) {
    if !state.visible {
        return;
    }

    // Detect query change
    if state.query != state.prev_query {
        state.dirty = true;
        state.prev_query = state.query.clone();
    }

    if !state.dirty {
        return;
    }
    state.dirty = false;

    let query_lower = state.query.to_lowercase();
    let query_lower = query_lower.trim();

    // --- Building search ---
    state.building_results.clear();
    if state.search_buildings && !query_lower.is_empty() {
        for (entity, building, abandoned, under_construction) in &buildings {
            let zone_str = zone_label(building.zone_type);
            let status = if abandoned.is_some() {
                "Abandoned"
            } else if under_construction.is_some() {
                "Under Construction"
            } else {
                "Active"
            };

            let level_str = format!("L{}", building.level);
            let combined = format!(
                "{} {} {} {}",
                zone_str, level_str, status, building.zone_type as u8
            )
            .to_lowercase();

            if combined.contains(query_lower)
                || zone_str.to_lowercase().contains(query_lower)
                || status.to_lowercase().contains(query_lower)
                || level_str.to_lowercase().contains(query_lower)
            {
                state.building_results.push(BuildingResult {
                    entity,
                    zone_label: zone_str.to_string(),
                    level: building.level,
                    status,
                    grid_x: building.grid_x,
                    grid_y: building.grid_y,
                });
                if state.building_results.len() >= MAX_RESULTS {
                    break;
                }
            }
        }
    }

    // --- Citizen search ---
    state.citizen_results.clear();
    if state.search_citizens && !query_lower.is_empty() {
        for (entity, details, pos, home, _work) in &citizens {
            let name = citizen_name(entity, details.gender);
            let edu = education_label(details.education);
            let age_str = format!("{}", details.age);

            let combined = format!("{} {} {}", name, age_str, edu).to_lowercase();

            if combined.contains(query_lower) {
                // Use position if available, otherwise home location
                let (gx, gy) = if let Some(p) = pos {
                    (p.x, p.y)
                } else if let Some(h) = home {
                    (
                        h.grid_x as f32 * CELL_SIZE + CELL_SIZE * 0.5,
                        h.grid_y as f32 * CELL_SIZE + CELL_SIZE * 0.5,
                    )
                } else {
                    (0.0, 0.0)
                };

                state.citizen_results.push(CitizenResult {
                    entity,
                    name,
                    age: details.age,
                    education: edu,
                    grid_x: gx,
                    grid_y: gy,
                });
                if state.citizen_results.len() >= MAX_RESULTS {
                    break;
                }
            }
        }
    }
}

/// Render the search panel UI.
pub fn search_panel_ui(
    mut contexts: EguiContexts,
    mut state: ResMut<SearchState>,
    mut orbit: ResMut<OrbitCamera>,
) {
    if !state.visible {
        return;
    }

    let mut jump_target: Option<(f32, f32)> = None;

    egui::Window::new("\u{1f50d} Search")
        .default_width(350.0)
        .default_height(400.0)
        .anchor(egui::Align2::RIGHT_TOP, egui::vec2(-8.0, 40.0))
        .resizable(true)
        .collapsible(true)
        .show(contexts.ctx_mut(), |ui| {
            // Search input
            ui.horizontal(|ui| {
                ui.label("Search:");
                let text_edit = ui.text_edit_singleline(&mut state.query);
                if state.request_focus {
                    text_edit.request_focus();
                    state.request_focus = false;
                }
            });

            ui.horizontal(|ui| {
                if ui
                    .checkbox(&mut state.search_buildings, "Buildings")
                    .changed()
                {
                    state.dirty = true;
                }
                if ui
                    .checkbox(&mut state.search_citizens, "Citizens")
                    .changed()
                {
                    state.dirty = true;
                }
            });

            if state.query.trim().is_empty() {
                ui.separator();
                ui.label("Type to search buildings and citizens.");
                ui.label("Examples: \"residential\", \"abandoned\", \"L3\", \"James\"");
                return;
            }

            ui.separator();

            let total = state.building_results.len() + state.citizen_results.len();
            ui.label(format!(
                "{} result{} ({} buildings, {} citizens)",
                total,
                if total == 1 { "" } else { "s" },
                state.building_results.len(),
                state.citizen_results.len(),
            ));

            egui::ScrollArea::vertical()
                .max_height(500.0)
                .show(ui, |ui| {
                    // --- Building results ---
                    if !state.building_results.is_empty() {
                        ui.add_space(4.0);
                        ui.strong("Buildings");
                        ui.separator();

                        for result in &state.building_results {
                            let label = format!(
                                "{} L{} - {} ({},{})",
                                result.zone_label,
                                result.level,
                                result.status,
                                result.grid_x,
                                result.grid_y,
                            );
                            if ui
                                .selectable_label(false, &label)
                                .on_hover_text("Click to jump to location")
                                .clicked()
                            {
                                let wx = result.grid_x as f32 * CELL_SIZE + CELL_SIZE * 0.5;
                                let wy = result.grid_y as f32 * CELL_SIZE + CELL_SIZE * 0.5;
                                jump_target = Some((wx, wy));
                            }
                        }
                    }

                    // --- Citizen results ---
                    if !state.citizen_results.is_empty() {
                        ui.add_space(4.0);
                        ui.strong("Citizens");
                        ui.separator();

                        for result in &state.citizen_results {
                            let label = format!(
                                "{} (Age {}, {})",
                                result.name, result.age, result.education,
                            );
                            if ui
                                .selectable_label(false, &label)
                                .on_hover_text("Click to jump to location")
                                .clicked()
                            {
                                jump_target = Some((result.grid_x, result.grid_y));
                            }
                        }
                    }

                    if state.building_results.is_empty() && state.citizen_results.is_empty() {
                        ui.add_space(8.0);
                        ui.label("No results found.");
                    }
                });
        });

    // Apply camera jump outside the egui closure (to avoid borrow issues)
    if let Some((wx, wy)) = jump_target {
        orbit.focus.x = wx;
        orbit.focus.z = wy;
        // Zoom in closer to show the target
        orbit.distance = orbit.distance.min(400.0);
    }
}

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

pub struct SearchPlugin;

impl Plugin for SearchPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SearchState>().add_systems(
            Update,
            (search_keybind, update_search_results, search_panel_ui).chain(),
        );
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zone_label_all_variants() {
        assert_eq!(zone_label(ZoneType::None), "None");
        assert_eq!(zone_label(ZoneType::ResidentialLow), "Residential (Low)");
        assert_eq!(zone_label(ZoneType::ResidentialMedium), "Residential (Med)");
        assert_eq!(zone_label(ZoneType::ResidentialHigh), "Residential (High)");
        assert_eq!(zone_label(ZoneType::CommercialLow), "Commercial (Low)");
        assert_eq!(zone_label(ZoneType::CommercialHigh), "Commercial (High)");
        assert_eq!(zone_label(ZoneType::Industrial), "Industrial");
        assert_eq!(zone_label(ZoneType::Office), "Office");
        assert_eq!(zone_label(ZoneType::MixedUse), "Mixed Use");
    }

    #[test]
    fn test_education_label() {
        assert_eq!(education_label(0), "None");
        assert_eq!(education_label(1), "Elementary");
        assert_eq!(education_label(2), "High School");
        assert_eq!(education_label(3), "University");
        assert_eq!(education_label(4), "Advanced");
    }

    #[test]
    fn test_citizen_name_deterministic() {
        let entity = Entity::from_raw(42);
        let name1 = citizen_name(entity, Gender::Male);
        let name2 = citizen_name(entity, Gender::Male);
        assert_eq!(name1, name2, "Names should be deterministic");
    }

    #[test]
    fn test_citizen_name_gender_difference() {
        let entity = Entity::from_raw(0);
        let male_name = citizen_name(entity, Gender::Male);
        let female_name = citizen_name(entity, Gender::Female);
        assert_ne!(male_name, female_name, "Male/female names should differ");
    }

    #[test]
    fn test_search_state_default() {
        let state = SearchState::default();
        assert!(!state.visible);
        assert!(state.query.is_empty());
        assert!(state.search_buildings);
        assert!(state.search_citizens);
        assert!(state.building_results.is_empty());
        assert!(state.citizen_results.is_empty());
    }
}
