//! Citizen Info Panel (UX-063).
//!
//! When a citizen entity is clicked (in Inspect mode), displays:
//! - Name, age, gender
//! - Job type and workplace location
//! - Happiness with factor breakdown (needs)
//! - Current state (at home, commuting, at work, shopping, etc.)
//! - Home and work locations
//! - "Follow" button to enter camera follow mode

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use rendering::camera::OrbitCamera;
use rendering::enhanced_select::SelectionKind;
use rendering::input::ActiveTool;
use simulation::citizen::{
    Citizen, CitizenDetails, CitizenState, CitizenStateComp, Family, Gender, HomeLocation, Needs,
    Personality, Position, WorkLocation,
};
use simulation::config::CELL_SIZE;

// ---------------------------------------------------------------------------
// Resources
// ---------------------------------------------------------------------------

/// Resource tracking the currently selected citizen entity.
#[derive(Resource, Default)]
pub struct SelectedCitizen(pub Option<Entity>);

/// Resource indicating whether the camera should follow a citizen.
#[derive(Resource, Default)]
pub struct FollowCitizen(pub Option<Entity>);

// ---------------------------------------------------------------------------
// Name generation (deterministic from Entity index + Gender)
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
    "Emily",
    "Donna",
    "Michelle",
    "Carol",
    "Amanda",
    "Dorothy",
    "Melissa",
    "Deborah",
    "Stephanie",
    "Rebecca",
    "Sharon",
    "Laura",
    "Cynthia",
    "Kathleen",
    "Amy",
    "Angela",
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

fn state_label(state: CitizenState) -> &'static str {
    match state {
        CitizenState::AtHome => "At Home",
        CitizenState::CommutingToWork => "Commuting to Work",
        CitizenState::Working => "Working",
        CitizenState::CommutingHome => "Commuting Home",
        CitizenState::CommutingToShop => "Going Shopping",
        CitizenState::Shopping => "Shopping",
        CitizenState::CommutingToLeisure => "Going to Leisure",
        CitizenState::AtLeisure => "At Leisure",
        CitizenState::CommutingToSchool => "Going to School",
        CitizenState::AtSchool => "At School",
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

fn gender_label(gender: Gender) -> &'static str {
    match gender {
        Gender::Male => "Male",
        Gender::Female => "Female",
    }
}

fn happiness_color(happiness: f32) -> egui::Color32 {
    if happiness >= 70.0 {
        egui::Color32::from_rgb(50, 200, 50)
    } else if happiness >= 40.0 {
        egui::Color32::from_rgb(220, 180, 50)
    } else {
        egui::Color32::from_rgb(220, 50, 50)
    }
}

fn need_color(value: f32) -> egui::Color32 {
    let pct = value / 100.0;
    if pct > 0.6 {
        egui::Color32::from_rgb(50, 200, 50)
    } else if pct > 0.3 {
        egui::Color32::from_rgb(220, 180, 50)
    } else {
        egui::Color32::from_rgb(220, 50, 50)
    }
}

fn needs_bar(ui: &mut egui::Ui, label: &str, value: f32) {
    ui.horizontal(|ui| {
        ui.label(format!("{:>7}", label));
        let (rect, _) = ui.allocate_exact_size(egui::vec2(80.0, 10.0), egui::Sense::hover());
        let painter = ui.painter_at(rect);
        painter.rect_filled(rect, 2.0, egui::Color32::from_gray(40));
        let pct = (value / 100.0).clamp(0.0, 1.0);
        let color = need_color(value);
        let fill_rect =
            egui::Rect::from_min_size(rect.min, egui::vec2(rect.width() * pct, rect.height()));
        painter.rect_filled(fill_rect, 2.0, color);
        ui.label(format!("{:.0}%", value));
    });
}

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

/// Detect citizen clicks when in Inspect mode.
///
/// When the user left-clicks in Inspect mode, find the nearest citizen within
/// a small radius of the cursor position. Citizens are matched by their
/// world-space Position component against the cursor grid position.
pub fn detect_citizen_selection(
    selection_kind: Res<SelectionKind>,
    tool: Res<ActiveTool>,
    mut selected: ResMut<SelectedCitizen>,
    mut follow: ResMut<FollowCitizen>,
) {
    if *tool != ActiveTool::Inspect {
        selected.0 = None;
        follow.0 = None;
        return;
    }

    // Defer to the enhanced selection system for priority-based selection.
    // Only show the citizen panel when the enhanced selector determined a
    // citizen was the highest-priority entity at the click position.
    if let SelectionKind::Citizen(entity) = *selection_kind {
        selected.0 = Some(entity);
    } else if selection_kind.is_changed() {
        // Another entity type was selected; clear citizen selection.
        selected.0 = None;
        follow.0 = None;
    }
}

/// Camera follow system: if FollowCitizen is active, move the camera focus
/// to track the citizen's position each frame.
pub fn camera_follow_citizen(
    follow: Res<FollowCitizen>,
    citizens: Query<&Position, With<Citizen>>,
    mut orbit: ResMut<OrbitCamera>,
) {
    let Some(entity) = follow.0 else {
        return;
    };

    let Ok(pos) = citizens.get(entity) else {
        return;
    };

    // Update the camera focus to the citizen's world position.
    // The OrbitCamera uses (x, 0, z) where x = world_x, z = world_y.
    orbit.focus.x = pos.x;
    orbit.focus.z = pos.y;
}

/// Render the Citizen Info Panel when a citizen is selected.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub fn citizen_info_panel_ui(
    mut contexts: EguiContexts,
    selected: Res<SelectedCitizen>,
    mut follow: ResMut<FollowCitizen>,
    citizens: Query<
        (
            Entity,
            &CitizenDetails,
            &CitizenStateComp,
            &HomeLocation,
            Option<&WorkLocation>,
            Option<&Needs>,
            Option<&Personality>,
            Option<&Family>,
        ),
        With<Citizen>,
    >,
    mut orbit: ResMut<OrbitCamera>,
) {
    let Some(entity) = selected.0 else {
        return;
    };

    let Ok((ent, details, state, home, work, needs, personality, family)) = citizens.get(entity)
    else {
        return;
    };

    let name = citizen_name(ent, details.gender);

    egui::Window::new("Citizen Info")
        .default_width(300.0)
        .anchor(egui::Align2::LEFT_TOP, egui::vec2(8.0, 40.0))
        .show(contexts.ctx_mut(), |ui| {
            // Name heading
            ui.heading(&name);
            ui.separator();

            // Basic info grid
            egui::Grid::new("citizen_basic_info")
                .num_columns(2)
                .show(ui, |ui| {
                    ui.label("Age:");
                    ui.label(format!("{}", details.age));
                    ui.end_row();

                    ui.label("Gender:");
                    ui.label(gender_label(details.gender));
                    ui.end_row();

                    ui.label("Education:");
                    ui.label(education_label(details.education));
                    ui.end_row();

                    ui.label("Salary:");
                    ui.label(format!("${:.0}/mo", details.salary));
                    ui.end_row();

                    ui.label("Savings:");
                    ui.label(format!("${:.0}", details.savings));
                    ui.end_row();

                    ui.label("Health:");
                    let health_color = happiness_color(details.health);
                    ui.colored_label(health_color, format!("{:.0}%", details.health));
                    ui.end_row();
                });

            // Happiness
            ui.separator();
            ui.heading("Happiness");
            let color = happiness_color(details.happiness);
            ui.colored_label(color, format!("{:.0}%", details.happiness));

            // Current state
            ui.separator();
            ui.heading("Status");
            egui::Grid::new("citizen_status")
                .num_columns(2)
                .show(ui, |ui| {
                    ui.label("Current:");
                    ui.label(state_label(state.0));
                    ui.end_row();

                    ui.label("Life stage:");
                    let stage = details.life_stage();
                    ui.label(format!("{:?}", stage));
                    ui.end_row();
                });

            // Needs breakdown
            if let Some(n) = needs {
                ui.separator();
                ui.heading("Needs");
                needs_bar(ui, "Hunger", n.hunger);
                needs_bar(ui, "Energy", n.energy);
                needs_bar(ui, "Social", n.social);
                needs_bar(ui, "Fun", n.fun);
                needs_bar(ui, "Comfort", n.comfort);

                let (critical_name, critical_val) = n.most_critical();
                if critical_val < 30.0 {
                    ui.colored_label(
                        egui::Color32::from_rgb(220, 50, 50),
                        format!("Critical: {} ({:.0}%)", critical_name, critical_val),
                    );
                }
            }

            // Personality traits
            if let Some(p) = personality {
                ui.separator();
                ui.heading("Personality");
                egui::Grid::new("citizen_personality")
                    .num_columns(2)
                    .show(ui, |ui| {
                        ui.label("Ambition:");
                        ui.label(format!("{:.0}%", p.ambition * 100.0));
                        ui.end_row();

                        ui.label("Sociability:");
                        ui.label(format!("{:.0}%", p.sociability * 100.0));
                        ui.end_row();

                        ui.label("Materialism:");
                        ui.label(format!("{:.0}%", p.materialism * 100.0));
                        ui.end_row();

                        ui.label("Resilience:");
                        ui.label(format!("{:.0}%", p.resilience * 100.0));
                        ui.end_row();
                    });
            }

            // Locations
            ui.separator();
            ui.heading("Locations");

            // Home location (clickable)
            ui.horizontal(|ui| {
                ui.label("Home:");
                let home_label = format!("({}, {})", home.grid_x, home.grid_y);
                if ui.small_button(&home_label).clicked() {
                    let wx = home.grid_x as f32 * CELL_SIZE + CELL_SIZE * 0.5;
                    let wy = home.grid_y as f32 * CELL_SIZE + CELL_SIZE * 0.5;
                    orbit.focus.x = wx;
                    orbit.focus.z = wy;
                }
            });

            // Work location (clickable)
            if let Some(w) = work {
                ui.horizontal(|ui| {
                    ui.label("Work:");
                    let work_label = format!("({}, {})", w.grid_x, w.grid_y);
                    if ui.small_button(&work_label).clicked() {
                        let wx = w.grid_x as f32 * CELL_SIZE + CELL_SIZE * 0.5;
                        let wy = w.grid_y as f32 * CELL_SIZE + CELL_SIZE * 0.5;
                        orbit.focus.x = wx;
                        orbit.focus.z = wy;
                    }
                });
            } else {
                ui.label("Work: Unemployed");
            }

            // Family info
            if let Some(fam) = family {
                let has_family =
                    fam.partner.is_some() || !fam.children.is_empty() || fam.parent.is_some();
                if has_family {
                    ui.separator();
                    ui.heading("Family");
                    egui::Grid::new("citizen_family")
                        .num_columns(2)
                        .show(ui, |ui| {
                            if fam.partner.is_some() {
                                ui.label("Partner:");
                                ui.label("Yes");
                                ui.end_row();
                            }
                            if !fam.children.is_empty() {
                                ui.label("Children:");
                                ui.label(format!("{}", fam.children.len()));
                                ui.end_row();
                            }
                            if fam.parent.is_some() {
                                ui.label("Has parent:");
                                ui.label("Yes");
                                ui.end_row();
                            }
                        });
                }
            }

            // Follow button
            ui.separator();
            let is_following = follow.0 == Some(entity);
            let btn_text = if is_following {
                "Stop Following"
            } else {
                "Follow"
            };
            if ui.button(btn_text).clicked() {
                if is_following {
                    follow.0 = None;
                } else {
                    follow.0 = Some(entity);
                }
            }
        });
}

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

pub struct CitizenInfoPlugin;

impl Plugin for CitizenInfoPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SelectedCitizen>()
            .init_resource::<FollowCitizen>()
            .add_systems(
                Update,
                (
                    detect_citizen_selection,
                    citizen_info_panel_ui,
                    camera_follow_citizen,
                )
                    .chain(),
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
    fn test_state_label_all_states() {
        assert_eq!(state_label(CitizenState::AtHome), "At Home");
        assert_eq!(
            state_label(CitizenState::CommutingToWork),
            "Commuting to Work"
        );
        assert_eq!(state_label(CitizenState::Working), "Working");
        assert_eq!(state_label(CitizenState::CommutingHome), "Commuting Home");
        assert_eq!(state_label(CitizenState::CommutingToShop), "Going Shopping");
        assert_eq!(state_label(CitizenState::Shopping), "Shopping");
        assert_eq!(
            state_label(CitizenState::CommutingToLeisure),
            "Going to Leisure"
        );
        assert_eq!(state_label(CitizenState::AtLeisure), "At Leisure");
        assert_eq!(
            state_label(CitizenState::CommutingToSchool),
            "Going to School"
        );
        assert_eq!(state_label(CitizenState::AtSchool), "At School");
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
    fn test_gender_label() {
        assert_eq!(gender_label(Gender::Male), "Male");
        assert_eq!(gender_label(Gender::Female), "Female");
    }

    #[test]
    fn test_happiness_color_green() {
        let color = happiness_color(80.0);
        assert_eq!(color, egui::Color32::from_rgb(50, 200, 50));
    }

    #[test]
    fn test_happiness_color_yellow() {
        let color = happiness_color(50.0);
        assert_eq!(color, egui::Color32::from_rgb(220, 180, 50));
    }

    #[test]
    fn test_happiness_color_red() {
        let color = happiness_color(20.0);
        assert_eq!(color, egui::Color32::from_rgb(220, 50, 50));
    }

    #[test]
    fn test_need_color_green() {
        let color = need_color(80.0);
        assert_eq!(color, egui::Color32::from_rgb(50, 200, 50));
    }

    #[test]
    fn test_need_color_yellow() {
        let color = need_color(45.0);
        assert_eq!(color, egui::Color32::from_rgb(220, 180, 50));
    }

    #[test]
    fn test_need_color_red() {
        let color = need_color(10.0);
        assert_eq!(color, egui::Color32::from_rgb(220, 50, 50));
    }

    #[test]
    fn test_citizen_name_male() {
        let entity = Entity::from_raw(0);
        let name = citizen_name(entity, Gender::Male);
        assert_eq!(name, "James Smith");
    }

    #[test]
    fn test_citizen_name_female() {
        let entity = Entity::from_raw(0);
        let name = citizen_name(entity, Gender::Female);
        assert_eq!(name, "Mary Smith");
    }

    #[test]
    fn test_citizen_name_different_indices() {
        let e1 = Entity::from_raw(1);
        let e2 = Entity::from_raw(2);
        let n1 = citizen_name(e1, Gender::Male);
        let n2 = citizen_name(e2, Gender::Male);
        assert_ne!(n1, n2);
    }

    #[test]
    fn test_citizen_name_wraps_around() {
        // With 32 first names and 32 last names, index 32 should wrap first name
        let entity = Entity::from_raw(32);
        let name = citizen_name(entity, Gender::Male);
        // Index 32 % 32 = 0 -> "James", (32/31) % 32 = 1 -> "Johnson"
        assert_eq!(name, "James Johnson");
    }

    #[test]
    fn test_selected_citizen_default() {
        let selected = SelectedCitizen::default();
        assert!(selected.0.is_none());
    }

    #[test]
    fn test_follow_citizen_default() {
        let follow = FollowCitizen::default();
        assert!(follow.0.is_none());
    }
}
