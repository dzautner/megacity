//! Info panel sections: City Stats, Buildings, RCIO Demand, Employment,
//! and City Attractiveness.

use bevy_egui::egui;

use simulation::education_jobs::JobType;
use simulation::immigration::{CityAttractiveness, ImmigrationStats};
use simulation::new_game_config::NewGameConfig;
use simulation::stats::CityStats;
use simulation::zones::ZoneDemand;

use super::types::{coverage_bar, demand_bar, format_pop, InfoPanelExtras};

/// Render city stats, buildings, RCIO demand, and employment sections.
pub fn draw_city_stats(
    ui: &mut egui::Ui,
    stats: &CityStats,
    demand: &ZoneDemand,
    extras: &InfoPanelExtras,
    config: &NewGameConfig,
) {
    let homeless_stats = &extras.homeless_stats;
    let wind = &extras.wind;

    // Display player-chosen city name as the heading
    ui.heading(&config.city_name);
    ui.separator();

    ui.label(format!("Population: {}", format_pop(stats.population)));
    ui.label(format!("Happiness: {:.0}%", stats.average_happiness));
    if homeless_stats.total_homeless > 0 {
        let homeless_color = egui::Color32::from_rgb(220, 120, 50);
        ui.colored_label(
            homeless_color,
            format!(
                "Homeless: {} ({} sheltered)",
                homeless_stats.total_homeless, homeless_stats.sheltered
            ),
        );
    }
    // Welfare stats (show if any welfare infrastructure exists)
    {
        let ws = &extras.welfare_stats;
        if ws.shelter_count > 0 || ws.welfare_office_count > 0 {
            let welfare_color = egui::Color32::from_rgb(90, 160, 140);
            if ws.shelter_count > 0 {
                ui.colored_label(
                    welfare_color,
                    format!(
                        "Shelters: {} ({}/{} beds)",
                        ws.shelter_count, ws.shelter_occupancy, ws.shelter_capacity
                    ),
                );
            }
            if ws.welfare_office_count > 0 {
                ui.colored_label(
                    welfare_color,
                    format!(
                        "Welfare: {} offices, {} recipients",
                        ws.welfare_office_count, ws.total_welfare_recipients
                    ),
                );
            }
        }
    }
    if extras.death_care_stats.unprocessed > 0 {
        ui.colored_label(
            egui::Color32::from_rgb(220, 50, 50),
            format!("Unprocessed: {}", extras.death_care_stats.unprocessed),
        );
    }
    if extras.forest_fire_stats.active_fires > 0 {
        ui.colored_label(
            egui::Color32::from_rgb(255, 60, 20),
            format!(
                "FOREST FIRE: {} cells burning!",
                extras.forest_fire_stats.active_fires
            ),
        );
    }
    ui.label(format!("Roads: {} cells", stats.road_cells));
    {
        let active_accidents = extras.accident_tracker.active_accidents.len();
        if active_accidents > 0 {
            ui.colored_label(
                egui::Color32::from_rgb(220, 200, 50),
                format!("Accidents: {} active", active_accidents),
            );
        }
    }
    ui.label(format!(
        "Wind: {} {}",
        wind.compass_direction(),
        wind.speed_label(),
    ));

    ui.separator();
    ui.heading("Buildings");
    ui.label(format!("Residential: {}", stats.residential_buildings));
    ui.label(format!("Commercial: {}", stats.commercial_buildings));
    ui.label(format!("Industrial: {}", stats.industrial_buildings));
    ui.label(format!("Office: {}", stats.office_buildings));

    ui.separator();
    ui.heading("RCIO Demand");

    let r_color = egui::Color32::from_rgb(50, 180, 50);
    let c_color = egui::Color32::from_rgb(50, 80, 200);
    let i_color = egui::Color32::from_rgb(200, 180, 30);
    let o_color = egui::Color32::from_rgb(160, 130, 220);

    demand_bar(ui, "R", demand.residential, r_color);
    demand_bar(ui, "C", demand.commercial, c_color);
    demand_bar(ui, "I", demand.industrial, i_color);
    demand_bar(ui, "O", demand.office, o_color);

    draw_employment(ui, extras);
}

fn draw_employment(ui: &mut egui::Ui, extras: &InfoPanelExtras) {
    let employment_stats = &extras.employment_stats;

    ui.separator();
    ui.heading("Employment");
    let unemp_color = if employment_stats.unemployment_rate > 0.10 {
        egui::Color32::from_rgb(220, 50, 50)
    } else if employment_stats.unemployment_rate > 0.05 {
        egui::Color32::from_rgb(220, 180, 50)
    } else {
        egui::Color32::from_rgb(50, 200, 50)
    };
    ui.colored_label(
        unemp_color,
        format!(
            "Unemployment: {:.1}%",
            employment_stats.unemployment_rate * 100.0
        ),
    );
    ui.label(format!(
        "Employed: {} | Unemployed: {}",
        format_pop(employment_stats.total_employed),
        format_pop(employment_stats.total_unemployed),
    ));
    for &jt in JobType::all() {
        let (filled, total) = employment_stats
            .jobs_by_type
            .get(&jt)
            .copied()
            .unwrap_or((0, 0));
        if total > 0 {
            let fill_pct = filled as f32 / total as f32;
            let bar_color = match jt {
                JobType::Unskilled => egui::Color32::from_rgb(180, 140, 80),
                JobType::Service => egui::Color32::from_rgb(80, 160, 180),
                JobType::Skilled => egui::Color32::from_rgb(80, 180, 80),
                JobType::Professional => egui::Color32::from_rgb(100, 100, 220),
                JobType::Executive => egui::Color32::from_rgb(180, 80, 180),
            };
            coverage_bar(ui, jt.name(), fill_pct, bar_color);
        }
    }
}

/// Render the City Attractiveness collapsing section.
pub fn draw_attractiveness(
    ui: &mut egui::Ui,
    attractiveness: &CityAttractiveness,
    imm_stats: &ImmigrationStats,
) {
    ui.separator();
    ui.collapsing("City Attractiveness", |ui| {
        let score = attractiveness.overall_score;
        let score_color = if score >= 70.0 {
            egui::Color32::from_rgb(50, 200, 50)
        } else if score >= 40.0 {
            egui::Color32::from_rgb(220, 180, 50)
        } else {
            egui::Color32::from_rgb(220, 50, 50)
        };

        ui.horizontal(|ui| {
            ui.label("Score:");
            ui.colored_label(score_color, format!("{:.0}/100", score));
        });

        // Score bar
        let (rect, _) = ui.allocate_exact_size(egui::vec2(160.0, 12.0), egui::Sense::hover());
        let painter = ui.painter_at(rect);
        painter.rect_filled(rect, 2.0, egui::Color32::from_gray(40));
        let fill_rect = egui::Rect::from_min_size(
            rect.min,
            egui::vec2(
                rect.width() * (score / 100.0).clamp(0.0, 1.0),
                rect.height(),
            ),
        );
        painter.rect_filled(fill_rect, 2.0, score_color);

        ui.add_space(4.0);

        let factor_color = |v: f32| -> egui::Color32 {
            if v >= 0.7 {
                egui::Color32::from_rgb(50, 200, 50)
            } else if v >= 0.4 {
                egui::Color32::from_rgb(220, 180, 50)
            } else {
                egui::Color32::from_rgb(220, 50, 50)
            }
        };

        egui::Grid::new("attractiveness_breakdown")
            .num_columns(2)
            .show(ui, |ui| {
                ui.label("Employment:");
                ui.colored_label(
                    factor_color(attractiveness.employment_factor),
                    format!("{:.0}%", attractiveness.employment_factor * 100.0),
                );
                ui.end_row();

                ui.label("Happiness:");
                ui.colored_label(
                    factor_color(attractiveness.happiness_factor),
                    format!("{:.0}%", attractiveness.happiness_factor * 100.0),
                );
                ui.end_row();

                ui.label("Services:");
                ui.colored_label(
                    factor_color(attractiveness.services_factor),
                    format!("{:.0}%", attractiveness.services_factor * 100.0),
                );
                ui.end_row();

                ui.label("Housing:");
                ui.colored_label(
                    factor_color(attractiveness.housing_factor),
                    format!("{:.0}%", attractiveness.housing_factor * 100.0),
                );
                ui.end_row();

                ui.label("Tax:");
                ui.colored_label(
                    factor_color(attractiveness.tax_factor),
                    format!("{:.0}%", attractiveness.tax_factor * 100.0),
                );
                ui.end_row();
            });

        ui.add_space(4.0);

        // Migration stats
        let net = imm_stats.net_migration;
        let net_color = if net > 0 {
            egui::Color32::from_rgb(50, 200, 50)
        } else if net < 0 {
            egui::Color32::from_rgb(220, 50, 50)
        } else {
            egui::Color32::from_rgb(180, 180, 180)
        };
        let sign = if net > 0 { "+" } else { "" };
        ui.horizontal(|ui| {
            ui.label("Net migration:");
            ui.colored_label(net_color, format!("{}{}", sign, net));
        });
        ui.label(format!(
            "In: {} | Out: {}",
            imm_stats.immigrants_this_month, imm_stats.emigrants_this_month
        ));
    });
}
