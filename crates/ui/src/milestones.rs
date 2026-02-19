use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use simulation::stats::CityStats;

#[derive(Resource, Default)]
pub struct Milestones {
    pub reached: Vec<MilestoneEntry>,
    pub last_check_pop: u32,
    pub unlocked_landmarks: Vec<&'static str>,
}

#[derive(Debug, Clone)]
pub struct MilestoneEntry {
    pub name: String,
    pub population: u32,
    pub achieved: bool,
}

const MILESTONE_POPS: &[(u32, &str)] = &[
    (100, "Village"),
    (500, "Hamlet"),
    (1_000, "Town"),
    (5_000, "Small City"),
    (10_000, "City"),
    (25_000, "Large City"),
    (50_000, "Metropolis"),
    (100_000, "Major Metropolis"),
    (250_000, "Megacity"),
    (500_000, "Megalopolis"),
    (1_000_000, "World Capital"),
];

const LANDMARK_UNLOCKS: &[(u32, &str)] = &[
    (1_000, "CityHall"),
    (10_000, "Museum"),
    (50_000, "Cathedral"),
    (100_000, "TVStation"),
];

pub fn check_milestones(stats: Res<CityStats>, mut milestones: ResMut<Milestones>) {
    if stats.population == milestones.last_check_pop {
        return;
    }
    milestones.last_check_pop = stats.population;

    for &(pop, name) in MILESTONE_POPS {
        if stats.population >= pop {
            let already = milestones.reached.iter().any(|m| m.population == pop);
            if !already {
                milestones.reached.push(MilestoneEntry {
                    name: name.to_string(),
                    population: pop,
                    achieved: true,
                });
            }
        }
    }

    // Update landmark unlocks
    milestones.unlocked_landmarks.clear();
    for &(pop, landmark) in LANDMARK_UNLOCKS {
        if stats.population >= pop {
            milestones.unlocked_landmarks.push(landmark);
        }
    }
}

pub fn milestones_ui(
    mut contexts: EguiContexts,
    milestones: Res<Milestones>,
    stats: Res<CityStats>,
) {
    egui::Window::new("Milestones")
        .default_open(false)
        .show(contexts.ctx_mut(), |ui| {
            ui.label(format!(
                "City Status: {}",
                current_city_name(stats.population)
            ));
            ui.separator();

            for &(pop, name) in MILESTONE_POPS {
                let reached = milestones.reached.iter().any(|m| m.population == pop);
                let text = if reached {
                    format!("[x] {} ({} pop)", name, pop)
                } else {
                    format!("[ ] {} ({} pop)", name, pop)
                };
                ui.label(text);
            }

            if !milestones.unlocked_landmarks.is_empty() {
                ui.separator();
                ui.label("Unlocked Landmarks:");
                for landmark in &milestones.unlocked_landmarks {
                    ui.label(format!("  - {}", landmark));
                }
            }
        });
}

fn current_city_name(pop: u32) -> &'static str {
    let mut name = "Settlement";
    for &(threshold, n) in MILESTONE_POPS {
        if pop >= threshold {
            name = n;
        }
    }
    name
}
