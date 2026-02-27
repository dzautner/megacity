//! Economy tab renderer for the Building Inspector.
//!
//! Shows land value, property value, zone-specific tax rate, salary/rent
//! estimates, income distribution (residential) or workforce education
//! (commercial).

use bevy::prelude::*;
use bevy_egui::egui;

use simulation::budget::ExtendedBudget;
use simulation::buildings::Building;
use simulation::citizen::{
    Citizen, CitizenDetails, CitizenStateComp, Family, HomeLocation, Needs, Personality,
    WorkLocation,
};
use simulation::grid::ZoneType;
use simulation::wealth::WealthTier;

// =============================================================================
// Tab content: Economy
// =============================================================================

#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub(crate) fn render_economy_tab(
    ui: &mut egui::Ui,
    building_entity: Entity,
    building: &Building,
    land_val: u8,
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
    ext_budget: &ExtendedBudget,
) {
    // Determine the zone-specific tax rate for this building
    let zone_tax_rate = zone_rate_for_building(building.zone_type, ext_budget);

    egui::Grid::new("tab_econ_overview")
        .num_columns(2)
        .spacing([16.0, 4.0])
        .show(ui, |ui| {
            ui.label("Land Value:");
            let lv_color = if land_val >= 180 {
                egui::Color32::from_rgb(50, 200, 50)
            } else if land_val >= 80 {
                egui::Color32::from_rgb(220, 180, 50)
            } else {
                egui::Color32::from_rgb(180, 100, 60)
            };
            ui.colored_label(lv_color, format!("{}/255", land_val));
            ui.end_row();

            // Estimated property value (land value * capacity factor)
            let property_value = land_val as f32 * building.capacity as f32 * 10.0;
            ui.label("Property Value:");
            ui.label(format!("${:.0}", property_value));
            ui.end_row();

            // Zone-specific tax rate
            ui.label("Tax Rate:");
            ui.label(format!("{:.1}%", zone_tax_rate * 100.0));
            ui.end_row();
        });

    ui.separator();

    if building.zone_type.is_residential() {
        let residents: Vec<&CitizenDetails> = citizens
            .iter()
            .filter(|(_, _, home, _, _, _, _, _)| home.building == building_entity)
            .map(|(_, details, _, _, _, _, _, _)| details)
            .collect();

        if !residents.is_empty() {
            let count = residents.len() as f32;
            let avg_salary: f32 = residents.iter().map(|r| r.salary).sum::<f32>() / count;
            let tax_revenue: f32 =
                residents.iter().map(|r| r.salary * zone_tax_rate).sum();
            let avg_rent = avg_salary * 0.3; // estimate 30% of income

            egui::Grid::new("tab_econ_res")
                .num_columns(2)
                .spacing([16.0, 4.0])
                .show(ui, |ui| {
                    ui.label("Avg Salary:");
                    ui.label(format!("${:.0}/mo", avg_salary));
                    ui.end_row();
                    ui.label("Est. Avg Rent:");
                    ui.label(format!("${:.0}/mo", avg_rent));
                    ui.end_row();
                    ui.label("Tax Revenue:");
                    ui.label(format!("${:.0}/mo", tax_revenue));
                    ui.end_row();
                });

            // Income distribution
            ui.separator();
            ui.label("Income Distribution:");
            let mut wealth_counts = [0u32; 3];
            for r in &residents {
                match WealthTier::from_education(r.education) {
                    WealthTier::LowIncome => wealth_counts[0] += 1,
                    WealthTier::MiddleIncome => wealth_counts[1] += 1,
                    WealthTier::HighIncome => wealth_counts[2] += 1,
                }
            }
            egui::Grid::new("tab_wealth")
                .num_columns(2)
                .spacing([16.0, 2.0])
                .show(ui, |ui| {
                    ui.label("Low income");
                    ui.label(format!("{}", wealth_counts[0]));
                    ui.end_row();
                    ui.label("Middle income");
                    ui.label(format!("{}", wealth_counts[1]));
                    ui.end_row();
                    ui.label("High income");
                    ui.label(format!("{}", wealth_counts[2]));
                    ui.end_row();
                });
        } else {
            ui.label("No residents yet");
        }
    } else {
        // Commercial/Industrial/Office
        let workers: Vec<&CitizenDetails> = citizens
            .iter()
            .filter(|(_, _, _, work, _, _, _, _)| {
                work.map(|w| w.building == building_entity).unwrap_or(false)
            })
            .map(|(_, details, _, _, _, _, _, _)| details)
            .collect();

        if !workers.is_empty() {
            let count = workers.len() as f32;
            let avg_salary: f32 = workers.iter().map(|w| w.salary).sum::<f32>() / count;

            egui::Grid::new("tab_econ_work")
                .num_columns(2)
                .spacing([16.0, 4.0])
                .show(ui, |ui| {
                    ui.label("Avg Salary:");
                    ui.label(format!("${:.0}/mo", avg_salary));
                    ui.end_row();
                    ui.label("Payroll:");
                    ui.label(format!("${:.0}/mo", avg_salary * count));
                    ui.end_row();
                });

            // Workforce education
            ui.separator();
            ui.label("Workforce Education:");
            let mut edu_counts = [0u32; 4];
            for w in &workers {
                let idx = (w.education as usize).min(3);
                edu_counts[idx] += 1;
            }
            egui::Grid::new("tab_worker_edu")
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
        } else {
            ui.label("No workers yet");
        }
    }
}

/// Return the zone-specific tax rate for a building's zone type.
fn zone_rate_for_building(zone_type: ZoneType, ext_budget: &ExtendedBudget) -> f32 {
    let zt = &ext_budget.zone_taxes;
    if zone_type.is_residential() || zone_type.is_mixed_use() {
        zt.residential
    } else if zone_type.is_commercial() {
        zt.commercial
    } else if zone_type == ZoneType::Industrial {
        zt.industrial
    } else if zone_type == ZoneType::Office {
        zt.office
    } else {
        // Fallback: average of all zone rates
        (zt.residential + zt.commercial + zt.industrial + zt.office) / 4.0
    }
}
