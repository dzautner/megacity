//! Info panel sections: Service Coverage, Heating, Groundwater, Districts,
//! Outside Connections, and Aviation.

use bevy_egui::egui;

use simulation::districts::DistrictMap;

use super::types::{coverage_bar, format_pop, CoverageCache, InfoPanelExtras};

/// Render service coverage bars from cached values.
pub fn draw_service_coverage(
    ui: &mut egui::Ui,
    coverage: &CoverageCache,
    extras: &InfoPanelExtras,
) {
    ui.separator();
    ui.heading("Service Coverage");
    coverage_bar(
        ui,
        "Power",
        coverage.power,
        egui::Color32::from_rgb(220, 200, 50),
    );
    coverage_bar(
        ui,
        "Water",
        coverage.water,
        egui::Color32::from_rgb(50, 130, 220),
    );
    coverage_bar(
        ui,
        "Education",
        coverage.education,
        egui::Color32::from_rgb(100, 180, 220),
    );
    coverage_bar(
        ui,
        "Fire",
        coverage.fire,
        egui::Color32::from_rgb(220, 80, 50),
    );
    coverage_bar(
        ui,
        "Police",
        coverage.police,
        egui::Color32::from_rgb(50, 80, 200),
    );
    coverage_bar(
        ui,
        "Health",
        coverage.health,
        egui::Color32::from_rgb(220, 50, 120),
    );
    coverage_bar(
        ui,
        "Telecom",
        coverage.telecom,
        egui::Color32::from_rgb(150, 200, 80),
    );
    coverage_bar(
        ui,
        "Postal",
        extras.postal_stats.coverage_percentage / 100.0,
        egui::Color32::from_rgb(180, 130, 80),
    );

    // Heating coverage (only show when there's cold weather demand)
    {
        let heat_demand = simulation::heating::heating_demand(&extras.weather);
        if heat_demand > 0.0 {
            let heat_color = egui::Color32::from_rgb(220, 120, 30);
            coverage_bar(ui, "Heating", extras.heating_stats.coverage_pct, heat_color);
            ui.small(format!(
                "Demand: {:.0}% | Cost: ${:.0}/mo | Eff: {:.0}%",
                heat_demand * 100.0,
                extras.heating_stats.monthly_cost,
                extras.heating_stats.efficiency * 100.0,
            ));
        }
    }
}

/// Render the Groundwater collapsing section.
pub fn draw_groundwater(ui: &mut egui::Ui, extras: &InfoPanelExtras) {
    ui.separator();
    ui.collapsing("Groundwater", |ui| {
        let gw = &extras.groundwater_stats;
        let level_pct = gw.avg_level / 255.0;
        let quality_pct = gw.avg_quality / 255.0;
        coverage_bar(
            ui,
            "Water Table",
            level_pct,
            egui::Color32::from_rgb(50, 120, 200),
        );
        let quality_color = if quality_pct >= 0.7 {
            egui::Color32::from_rgb(50, 180, 80)
        } else if quality_pct >= 0.4 {
            egui::Color32::from_rgb(200, 180, 50)
        } else {
            egui::Color32::from_rgb(200, 60, 50)
        };
        coverage_bar(ui, "Water Quality", quality_pct, quality_color);
        if gw.contaminated_cells > 0 {
            ui.colored_label(
                egui::Color32::from_rgb(200, 80, 50),
                format!("{} contaminated cells", gw.contaminated_cells),
            );
        }
        if gw.treatment_capacity > 0 {
            ui.small(format!(
                "{} treatment plant(s) active",
                gw.treatment_capacity
            ));
        }
    });
}

/// Render the Districts section.
pub fn draw_districts(ui: &mut egui::Ui, district_map: &DistrictMap) {
    ui.separator();
    ui.heading("Districts");
    let has_any_district = district_map.districts.iter().any(|d| !d.cells.is_empty());
    if has_any_district {
        egui::Grid::new("district_stats_grid")
            .num_columns(3)
            .striped(true)
            .show(ui, |ui| {
                ui.strong("District");
                ui.strong("Pop");
                ui.strong("Happy");
                ui.end_row();

                for d in &district_map.districts {
                    if d.cells.is_empty() {
                        continue;
                    }
                    let color = egui::Color32::from_rgba_unmultiplied(
                        (d.color[0] * 255.0) as u8,
                        (d.color[1] * 255.0) as u8,
                        (d.color[2] * 255.0) as u8,
                        255,
                    );
                    ui.colored_label(color, &d.name);
                    ui.label(format_pop(d.stats.population));
                    if d.stats.population > 0 {
                        let h = d.stats.avg_happiness;
                        let h_color = if h >= 70.0 {
                            egui::Color32::from_rgb(50, 200, 50)
                        } else if h >= 40.0 {
                            egui::Color32::from_rgb(220, 180, 50)
                        } else {
                            egui::Color32::from_rgb(220, 50, 50)
                        };
                        ui.colored_label(h_color, format!("{:.0}%", h));
                    } else {
                        ui.label("-");
                    }
                    ui.end_row();
                }
            });
    } else {
        ui.small("No districts painted yet.");
        ui.small("Use the Districts toolbar to paint.");
    }
}

/// Render the Outside Connections collapsing section.
pub fn draw_outside_connections(ui: &mut egui::Ui, extras: &InfoPanelExtras) {
    ui.separator();
    ui.collapsing("Outside Connections", |ui| {
        let outside = &extras.outside_connections;
        let conn_stats = outside.stats();

        for stat in &conn_stats {
            ui.horizontal(|ui| {
                let (status_color, status_text) = if stat.active {
                    (egui::Color32::from_rgb(50, 200, 50), "Connected")
                } else {
                    (egui::Color32::from_rgb(120, 120, 120), "Not available")
                };

                let (dot_rect, _) =
                    ui.allocate_exact_size(egui::vec2(10.0, 10.0), egui::Sense::hover());
                let painter = ui.painter_at(dot_rect);
                painter.circle_filled(dot_rect.center(), 4.0, status_color);

                ui.label(stat.connection_type.name());
                ui.colored_label(status_color, status_text);
            });

            if stat.active {
                ui.horizontal(|ui| {
                    ui.add_space(14.0);
                    ui.label("Utilization:");
                    let (bar_rect, _) =
                        ui.allocate_exact_size(egui::vec2(80.0, 10.0), egui::Sense::hover());
                    let painter = ui.painter_at(bar_rect);
                    painter.rect_filled(bar_rect, 2.0, egui::Color32::from_gray(40));
                    let util_color = if stat.avg_utilization > 0.8 {
                        egui::Color32::from_rgb(220, 50, 50)
                    } else if stat.avg_utilization > 0.5 {
                        egui::Color32::from_rgb(220, 180, 50)
                    } else {
                        egui::Color32::from_rgb(50, 200, 50)
                    };
                    let fill_rect = egui::Rect::from_min_size(
                        bar_rect.min,
                        egui::vec2(
                            bar_rect.width() * stat.avg_utilization.clamp(0.0, 1.0),
                            bar_rect.height(),
                        ),
                    );
                    painter.rect_filled(fill_rect, 2.0, util_color);
                    ui.label(format!("{:.0}%", stat.avg_utilization * 100.0));
                });

                ui.horizontal(|ui| {
                    ui.add_space(14.0);
                    ui.small(stat.effect_description);
                });

                if stat.count > 1 {
                    ui.horizontal(|ui| {
                        ui.add_space(14.0);
                        ui.small(format!("{} connection points", stat.count));
                    });
                }
            }
            ui.add_space(2.0);
        }
    });
}

/// Render the Aviation collapsing section.
pub fn draw_aviation(ui: &mut egui::Ui, extras: &InfoPanelExtras) {
    let airport = &extras.airport_stats;
    if airport.total_airports > 0 {
        ui.separator();
        ui.collapsing("Aviation", |ui| {
            egui::Grid::new("aviation_stats_grid")
                .num_columns(2)
                .show(ui, |ui| {
                    ui.label("Total airports:");
                    ui.label(format!("{}", airport.total_airports));
                    ui.end_row();

                    if airport.airports_by_tier[0] > 0 {
                        ui.label("  Airstrips:");
                        ui.label(format!("{}", airport.airports_by_tier[0]));
                        ui.end_row();
                    }
                    if airport.airports_by_tier[1] > 0 {
                        ui.label("  Regional:");
                        ui.label(format!("{}", airport.airports_by_tier[1]));
                        ui.end_row();
                    }
                    if airport.airports_by_tier[2] > 0 {
                        ui.label("  International:");
                        ui.label(format!("{}", airport.airports_by_tier[2]));
                        ui.end_row();
                    }

                    ui.label("Passenger flights/mo:");
                    ui.label(format_pop(airport.passenger_flights_per_month));
                    ui.end_row();

                    ui.label("Cargo flights/mo:");
                    ui.label(format_pop(airport.cargo_flights_per_month));
                    ui.end_row();

                    ui.label("Tourism multiplier:");
                    let mult_color = if airport.tourism_multiplier > 1.5 {
                        egui::Color32::from_rgb(50, 200, 50)
                    } else if airport.tourism_multiplier > 1.0 {
                        egui::Color32::from_rgb(220, 200, 50)
                    } else {
                        egui::Color32::from_rgb(180, 180, 180)
                    };
                    ui.colored_label(mult_color, format!("{:.2}x", airport.tourism_multiplier));
                    ui.end_row();

                    ui.label("Revenue:");
                    let rev_color = if airport.revenue > airport.total_monthly_cost {
                        egui::Color32::from_rgb(50, 200, 50)
                    } else {
                        egui::Color32::from_rgb(220, 50, 50)
                    };
                    ui.colored_label(rev_color, format!("${:.0}/mo", airport.revenue));
                    ui.end_row();

                    ui.label("Operating cost:");
                    ui.label(format!("${:.0}/mo", airport.total_monthly_cost));
                    ui.end_row();
                });
        });
    }
}
