//! Unit tests for the progressive disclosure module.

use bevy::prelude::*;
use bevy_egui::egui;

use simulation::citizen::{CitizenState, Gender};
use simulation::grid::ZoneType;
use simulation::trees::TreeGrid;

use super::helpers::{
    citizen_state_label, count_nearby_trees, education_short, gen_citizen_name, green_space_label,
    happiness_color, noise_color, occupancy_color, pollution_color, zone_type_label,
};
use super::types::{BuildingTab, SelectedBuildingTab};

// =========================================================================
// BuildingTab labels
// =========================================================================

#[test]
fn test_building_tab_labels() {
    assert_eq!(BuildingTab::Overview.label(), "Overview");
    assert_eq!(BuildingTab::Services.label(), "Services");
    assert_eq!(BuildingTab::Economy.label(), "Economy");
    assert_eq!(BuildingTab::Residents.label(), "Residents");
    assert_eq!(BuildingTab::Environment.label(), "Environment");
}

#[test]
fn test_building_tab_all_count() {
    assert_eq!(BuildingTab::ALL.len(), 5);
}

#[test]
fn test_building_tab_default_is_overview() {
    let tab = BuildingTab::default();
    assert_eq!(tab, BuildingTab::Overview);
}

// =========================================================================
// SelectedBuildingTab defaults
// =========================================================================

#[test]
fn test_selected_building_tab_default() {
    let state = SelectedBuildingTab::default();
    assert_eq!(state.0, BuildingTab::Overview);
}

// =========================================================================
// Zone type label
// =========================================================================

#[test]
fn test_zone_type_label() {
    assert_eq!(zone_type_label(ZoneType::None), "Unzoned");
    assert_eq!(
        zone_type_label(ZoneType::ResidentialLow),
        "Low-Density Residential"
    );
    assert_eq!(
        zone_type_label(ZoneType::ResidentialMedium),
        "Medium-Density Residential"
    );
    assert_eq!(
        zone_type_label(ZoneType::ResidentialHigh),
        "High-Density Residential"
    );
    assert_eq!(
        zone_type_label(ZoneType::CommercialLow),
        "Low-Density Commercial"
    );
    assert_eq!(
        zone_type_label(ZoneType::CommercialHigh),
        "High-Density Commercial"
    );
    assert_eq!(zone_type_label(ZoneType::Industrial), "Industrial");
    assert_eq!(zone_type_label(ZoneType::Office), "Office");
    assert_eq!(zone_type_label(ZoneType::MixedUse), "Mixed-Use");
}

// =========================================================================
// Color helpers
// =========================================================================

#[test]
fn test_happiness_color_green_high() {
    let color = happiness_color(80.0);
    assert_eq!(color, egui::Color32::from_rgb(50, 200, 50));
}

#[test]
fn test_happiness_color_yellow_mid() {
    let color = happiness_color(55.0);
    assert_eq!(color, egui::Color32::from_rgb(220, 180, 50));
}

#[test]
fn test_happiness_color_red_low() {
    let color = happiness_color(20.0);
    assert_eq!(color, egui::Color32::from_rgb(220, 50, 50));
}

#[test]
fn test_occupancy_color_green() {
    let color = occupancy_color(50.0);
    assert_eq!(color, egui::Color32::from_rgb(50, 200, 50));
}

#[test]
fn test_occupancy_color_yellow() {
    let color = occupancy_color(75.0);
    assert_eq!(color, egui::Color32::from_rgb(220, 180, 50));
}

#[test]
fn test_occupancy_color_red() {
    let color = occupancy_color(95.0);
    assert_eq!(color, egui::Color32::from_rgb(220, 50, 50));
}

#[test]
fn test_pollution_color_green() {
    let color = pollution_color(10);
    assert_eq!(color, egui::Color32::from_rgb(50, 200, 50));
}

#[test]
fn test_pollution_color_yellow() {
    let color = pollution_color(30);
    assert_eq!(color, egui::Color32::from_rgb(200, 150, 50));
}

#[test]
fn test_pollution_color_red() {
    let color = pollution_color(60);
    assert_eq!(color, egui::Color32::from_rgb(200, 50, 50));
}

#[test]
fn test_noise_color_green() {
    let color = noise_color(10);
    assert_eq!(color, egui::Color32::from_rgb(50, 200, 50));
}

#[test]
fn test_noise_color_yellow() {
    let color = noise_color(45);
    assert_eq!(color, egui::Color32::from_rgb(200, 150, 50));
}

#[test]
fn test_noise_color_red() {
    let color = noise_color(70);
    assert_eq!(color, egui::Color32::from_rgb(200, 50, 50));
}

// =========================================================================
// Education abbreviation
// =========================================================================

#[test]
fn test_education_short() {
    assert_eq!(education_short(0), "-");
    assert_eq!(education_short(1), "Elem");
    assert_eq!(education_short(2), "HS");
    assert_eq!(education_short(3), "Uni");
    assert_eq!(education_short(4), "Adv");
    assert_eq!(education_short(255), "Adv");
}

// =========================================================================
// Citizen state label
// =========================================================================

#[test]
fn test_citizen_state_labels() {
    assert_eq!(citizen_state_label(CitizenState::AtHome), "Home");
    assert_eq!(
        citizen_state_label(CitizenState::CommutingToWork),
        "To Work"
    );
    assert_eq!(citizen_state_label(CitizenState::Working), "Working");
    assert_eq!(
        citizen_state_label(CitizenState::CommutingHome),
        "Going Home"
    );
    assert_eq!(
        citizen_state_label(CitizenState::CommutingToShop),
        "To Shop"
    );
    assert_eq!(citizen_state_label(CitizenState::Shopping), "Shopping");
    assert_eq!(
        citizen_state_label(CitizenState::CommutingToLeisure),
        "To Leisure"
    );
    assert_eq!(citizen_state_label(CitizenState::AtLeisure), "Leisure");
    assert_eq!(
        citizen_state_label(CitizenState::CommutingToSchool),
        "To School"
    );
    assert_eq!(citizen_state_label(CitizenState::AtSchool), "At School");
}

// =========================================================================
// Citizen name generation (deterministic)
// =========================================================================

#[test]
fn test_citizen_name_deterministic() {
    let entity = Entity::from_raw(42);
    let name1 = gen_citizen_name(entity, Gender::Male);
    let name2 = gen_citizen_name(entity, Gender::Male);
    assert_eq!(name1, name2);
}

#[test]
fn test_citizen_name_gender_difference() {
    let entity = Entity::from_raw(7);
    let male_name = gen_citizen_name(entity, Gender::Male);
    let female_name = gen_citizen_name(entity, Gender::Female);
    // Same last name (based on entity index) but different first name
    assert_ne!(male_name, female_name);
}

// =========================================================================
// Green space helpers
// =========================================================================

#[test]
fn test_green_space_label_none() {
    let (label, _) = green_space_label(0);
    assert_eq!(label, "None");
}

#[test]
fn test_green_space_label_low() {
    let (label, _) = green_space_label(2);
    assert_eq!(label, "Low");
}

#[test]
fn test_green_space_label_moderate() {
    let (label, _) = green_space_label(5);
    assert_eq!(label, "Moderate");
}

#[test]
fn test_green_space_label_good() {
    let (label, _) = green_space_label(15);
    assert_eq!(label, "Good");
}

#[test]
fn test_green_space_label_excellent() {
    let (label, _) = green_space_label(25);
    assert_eq!(label, "Excellent");
}

#[test]
fn test_count_nearby_trees_empty_grid() {
    let grid = TreeGrid::default();
    assert_eq!(count_nearby_trees(&grid, 128, 128, 5), 0);
}

// =========================================================================
// Tab cycling (no mutation needed, just equality checks)
// =========================================================================

#[test]
fn test_building_tab_equality() {
    assert_eq!(BuildingTab::Overview, BuildingTab::Overview);
    assert_ne!(BuildingTab::Overview, BuildingTab::Services);
    assert_ne!(BuildingTab::Economy, BuildingTab::Environment);
}

#[test]
fn test_all_tabs_in_order() {
    assert_eq!(BuildingTab::ALL[0], BuildingTab::Overview);
    assert_eq!(BuildingTab::ALL[1], BuildingTab::Services);
    assert_eq!(BuildingTab::ALL[2], BuildingTab::Economy);
    assert_eq!(BuildingTab::ALL[3], BuildingTab::Residents);
    assert_eq!(BuildingTab::ALL[4], BuildingTab::Environment);
}
