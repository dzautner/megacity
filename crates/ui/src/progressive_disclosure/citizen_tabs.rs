//! Residents and Workers tab renderers for the Building Inspector.
//!
//! Both tabs display clickable citizen lists that allow following
//! individual citizens. Residents also show needs satisfaction and
//! demographic breakdowns.

use bevy::prelude::*;
use bevy_egui::egui;

use simulation::citizen::{
    Citizen, CitizenDetails, CitizenStateComp, Family, Gender, HomeLocation, Needs, Personality,
    WorkLocation,
};

use crate::citizen_info::{FollowCitizen, SelectedCitizen};

use super::helpers::{
    citizen_state_label, education_short, gen_citizen_name, happiness_color, needs_bar,
};

// =============================================================================
// Tab content: Residents (clickable)
// =============================================================================

#[allow(clippy::type_complexity)]
pub(crate) fn render_residents_tab(
    ui: &mut egui::Ui,
    building_entity: Entity,
    citizens: &Query<
        (
            Entity,
            &CitizenDetails,
            &HomeLocation,
            Option<&WorkLocation>,
            &CitizenStateComp,
            Option<&Needs>,
            Option<&Personality>,
            Option<&Family>,
        ),
        With<Citizen>,
    >,
    selected_citizen: &mut SelectedCitizen,
    follow_citizen: &mut FollowCitizen,
) {
    let mut residents: Vec<(
        Entity,
        &CitizenDetails,
        &CitizenStateComp,
        Option<&Needs>,
        Option<&Personality>,
        Option<&Family>,
    )> = citizens
        .iter()
        .filter(|(_, _, home, _, _, _, _, _)| home.building == building_entity)
        .map(|(e, details, _, _, state, needs, pers, fam)| (e, details, state, needs, pers, fam))
        .collect();

    let count = residents.len();
    ui.label(format!("{} residents tracked", count));

    if residents.is_empty() {
        return;
    }

    let avg_happiness: f32 = residents.iter().map(|r| r.1.happiness).sum::<f32>() / count as f32;
    let avg_age: f32 = residents.iter().map(|r| r.1.age as f32).sum::<f32>() / count as f32;
    let males = residents
        .iter()
        .filter(|r| r.1.gender == Gender::Male)
        .count();
    let children = residents
        .iter()
        .filter(|r| r.1.life_stage().should_attend_school() || !r.1.life_stage().can_work())
        .count();

    egui::Grid::new("tab_res_summary")
        .num_columns(2)
        .spacing([16.0, 4.0])
        .show(ui, |ui| {
            ui.label("Avg Happiness:");
            ui.colored_label(
                happiness_color(avg_happiness),
                format!("{:.0}%", avg_happiness),
            );
            ui.end_row();
            ui.label("Avg Age:");
            ui.label(format!("{:.0}", avg_age));
            ui.end_row();
            ui.label("Gender:");
            ui.label(format!("{} M / {} F", males, count - males));
            ui.end_row();
            ui.label("Children:");
            ui.label(format!("{}", children));
            ui.end_row();
        });

    // Average needs satisfaction
    let needs_count = residents.iter().filter(|r| r.3.is_some()).count();
    if needs_count > 0 {
        ui.separator();
        ui.label("Average Needs:");
        let (avg_h, avg_e, avg_s, avg_f, avg_c) = residents.iter().filter_map(|r| r.3).fold(
            (0.0f32, 0.0f32, 0.0f32, 0.0f32, 0.0f32),
            |acc, n| {
                (
                    acc.0 + n.hunger,
                    acc.1 + n.energy,
                    acc.2 + n.social,
                    acc.3 + n.fun,
                    acc.4 + n.comfort,
                )
            },
        );
        let nc = needs_count as f32;
        needs_bar(ui, "Hunger", avg_h / nc);
        needs_bar(ui, "Energy", avg_e / nc);
        needs_bar(ui, "Social", avg_s / nc);
        needs_bar(ui, "Fun", avg_f / nc);
        needs_bar(ui, "Comfort", avg_c / nc);
    }

    // Education breakdown
    ui.separator();
    ui.label("Education Breakdown:");
    let mut edu_counts = [0u32; 4];
    for r in &residents {
        let idx = (r.1.education as usize).min(3);
        edu_counts[idx] += 1;
    }
    egui::Grid::new("tab_edu_breakdown")
        .num_columns(2)
        .spacing([16.0, 2.0])
        .show(ui, |ui| {
            for (i, name) in ["None", "Elementary", "High School", "University"]
                .iter()
                .enumerate()
            {
                ui.label(*name);
                ui.label(format!("{}", edu_counts[i]));
                ui.end_row();
            }
        });

    // Individual resident list (scrollable, clickable)
    ui.separator();
    ui.label("Individual Residents (click to follow):");
    residents.sort_by(|a, b| {
        b.1.happiness
            .partial_cmp(&a.1.happiness)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    egui::ScrollArea::vertical()
        .max_height(280.0)
        .show(ui, |ui| {
            egui::Grid::new("tab_residents_list")
                .num_columns(7)
                .striped(true)
                .show(ui, |ui| {
                    ui.strong("Name");
                    ui.strong("Age");
                    ui.strong("Edu");
                    ui.strong("Happy");
                    ui.strong("Salary");
                    ui.strong("Needs");
                    ui.strong("Status");
                    ui.end_row();

                    for (i, (ent, details, state, needs, _personality, _family)) in
                        residents.iter().enumerate()
                    {
                        if i >= 50 {
                            break;
                        }
                        let name = gen_citizen_name(*ent, details.gender);
                        // Clickable name to select/follow citizen
                        if ui.small_button(&name).clicked() {
                            selected_citizen.0 = Some(*ent);
                            follow_citizen.0 = Some(*ent);
                        }
                        ui.label(format!("{}", details.age));
                        ui.label(education_short(details.education));
                        ui.colored_label(
                            happiness_color(details.happiness),
                            format!("{:.0}%", details.happiness),
                        );
                        ui.label(format!("${:.0}", details.salary));
                        if let Some(n) = needs {
                            let sat = n.overall_satisfaction();
                            let color = if sat > 0.7 {
                                egui::Color32::from_rgb(50, 200, 50)
                            } else if sat > 0.4 {
                                egui::Color32::from_rgb(220, 180, 50)
                            } else {
                                egui::Color32::from_rgb(220, 50, 50)
                            };
                            ui.colored_label(color, format!("{:.0}%", sat * 100.0));
                        } else {
                            ui.label("-");
                        }
                        ui.label(citizen_state_label(state.0));
                        ui.end_row();
                    }
                });

            if count > 50 {
                ui.label(format!("... and {} more", count - 50));
            }
        });
}

// =============================================================================
// Tab content: Workers (clickable)
// =============================================================================

#[allow(clippy::type_complexity)]
pub(crate) fn render_workers_tab(
    ui: &mut egui::Ui,
    building_entity: Entity,
    citizens: &Query<
        (
            Entity,
            &CitizenDetails,
            &HomeLocation,
            Option<&WorkLocation>,
            &CitizenStateComp,
            Option<&Needs>,
            Option<&Personality>,
            Option<&Family>,
        ),
        With<Citizen>,
    >,
    selected_citizen: &mut SelectedCitizen,
    follow_citizen: &mut FollowCitizen,
) {
    let mut workers: Vec<(Entity, &CitizenDetails, &CitizenStateComp)> = citizens
        .iter()
        .filter(|(_, _, _, work, _, _, _, _)| {
            work.map(|w| w.building == building_entity).unwrap_or(false)
        })
        .map(|(e, details, _, _, state, _, _, _)| (e, details, state))
        .collect();

    let count = workers.len();
    ui.label(format!("{} workers tracked", count));

    if workers.is_empty() {
        return;
    }

    let avg_happiness: f32 = workers.iter().map(|w| w.1.happiness).sum::<f32>() / count as f32;
    let avg_salary: f32 = workers.iter().map(|w| w.1.salary).sum::<f32>() / count as f32;

    egui::Grid::new("tab_worker_summary")
        .num_columns(2)
        .spacing([16.0, 4.0])
        .show(ui, |ui| {
            ui.label("Avg Happiness:");
            ui.colored_label(
                happiness_color(avg_happiness),
                format!("{:.0}%", avg_happiness),
            );
            ui.end_row();
            ui.label("Avg Salary:");
            ui.label(format!("${:.0}/mo", avg_salary));
            ui.end_row();
            ui.label("Payroll:");
            ui.label(format!("${:.0}/mo", avg_salary * count as f32));
            ui.end_row();
        });

    // Education breakdown
    ui.separator();
    let mut edu_counts = [0u32; 4];
    for w in &workers {
        let idx = (w.1.education as usize).min(3);
        edu_counts[idx] += 1;
    }
    ui.label("Workforce Education:");
    egui::Grid::new("tab_wrk_edu")
        .num_columns(2)
        .spacing([16.0, 2.0])
        .show(ui, |ui| {
            for (i, name) in ["None", "Elementary", "High School", "University"]
                .iter()
                .enumerate()
            {
                ui.label(*name);
                ui.label(format!("{}", edu_counts[i]));
                ui.end_row();
            }
        });

    // Individual worker list (clickable)
    ui.separator();
    ui.label("Individual Workers (click to follow):");
    workers.sort_by(|a, b| {
        b.1.happiness
            .partial_cmp(&a.1.happiness)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    egui::ScrollArea::vertical()
        .max_height(200.0)
        .show(ui, |ui| {
            egui::Grid::new("tab_workers_list")
                .num_columns(5)
                .striped(true)
                .show(ui, |ui| {
                    ui.strong("Name");
                    ui.strong("Age");
                    ui.strong("Edu");
                    ui.strong("Happy");
                    ui.strong("Salary");
                    ui.end_row();

                    for (i, (ent, details, _state)) in workers.iter().enumerate() {
                        if i >= 50 {
                            break;
                        }
                        let name = gen_citizen_name(*ent, details.gender);
                        // Clickable name to select/follow citizen
                        if ui.small_button(&name).clicked() {
                            selected_citizen.0 = Some(*ent);
                            follow_citizen.0 = Some(*ent);
                        }
                        ui.label(format!("{}", details.age));
                        ui.label(education_short(details.education));
                        ui.colored_label(
                            happiness_color(details.happiness),
                            format!("{:.0}%", details.happiness),
                        );
                        ui.label(format!("${:.0}", details.salary));
                        ui.end_row();
                    }
                });

            if count > 50 {
                ui.label(format!("... and {} more", count - 50));
            }
        });
}
