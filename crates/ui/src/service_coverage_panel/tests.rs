//! Tests for service categories, color helpers, and coverage labels.

use bevy_egui::egui;

use rendering::overlay::OverlayMode;
use simulation::services::ServiceType;

use super::categories::{OtherServiceGroup, ServiceCategory};
use super::panel_ui::{ExpandedCategories, ServiceCoveragePanelVisible};
use super::stats::{coverage_color, coverage_label};

// =========================================================================
// Color coding tests
// =========================================================================

#[test]
fn test_coverage_color_green_above_80() {
    let color = coverage_color(0.85);
    assert_eq!(color, egui::Color32::from_rgb(80, 200, 80));
}

#[test]
fn test_coverage_color_green_at_81() {
    let color = coverage_color(0.81);
    assert_eq!(color, egui::Color32::from_rgb(80, 200, 80));
}

#[test]
fn test_coverage_color_yellow_at_80() {
    // Exactly 80% should be yellow (not >80%)
    let color = coverage_color(0.80);
    assert_eq!(color, egui::Color32::from_rgb(220, 200, 50));
}

#[test]
fn test_coverage_color_yellow_at_50() {
    let color = coverage_color(0.50);
    assert_eq!(color, egui::Color32::from_rgb(220, 200, 50));
}

#[test]
fn test_coverage_color_yellow_at_65() {
    let color = coverage_color(0.65);
    assert_eq!(color, egui::Color32::from_rgb(220, 200, 50));
}

#[test]
fn test_coverage_color_red_below_50() {
    let color = coverage_color(0.49);
    assert_eq!(color, egui::Color32::from_rgb(255, 60, 60));
}

#[test]
fn test_coverage_color_red_at_zero() {
    let color = coverage_color(0.0);
    assert_eq!(color, egui::Color32::from_rgb(255, 60, 60));
}

#[test]
fn test_coverage_color_green_at_100() {
    let color = coverage_color(1.0);
    assert_eq!(color, egui::Color32::from_rgb(80, 200, 80));
}

// =========================================================================
// Coverage label tests
// =========================================================================

#[test]
fn test_coverage_label_good() {
    assert_eq!(coverage_label(0.85), "Good");
    assert_eq!(coverage_label(1.0), "Good");
}

#[test]
fn test_coverage_label_moderate() {
    assert_eq!(coverage_label(0.50), "Moderate");
    assert_eq!(coverage_label(0.80), "Moderate");
}

#[test]
fn test_coverage_label_poor() {
    assert_eq!(coverage_label(0.0), "Poor");
    assert_eq!(coverage_label(0.49), "Poor");
}

// =========================================================================
// ServiceCategory tests
// =========================================================================

#[test]
fn test_all_categories_count() {
    assert_eq!(ServiceCategory::ALL.len(), 8);
}

#[test]
fn test_category_names_non_empty() {
    for cat in ServiceCategory::ALL {
        assert!(!cat.name().is_empty());
    }
}

#[test]
fn test_category_coverage_bits_unique() {
    let mut seen = std::collections::HashSet::new();
    for cat in ServiceCategory::ALL {
        let bit = cat.coverage_bit();
        assert!(seen.insert(bit), "Duplicate coverage bit for {:?}", cat);
    }
}

#[test]
fn test_category_coverage_bits_nonzero() {
    for cat in ServiceCategory::ALL {
        assert_ne!(cat.coverage_bit(), 0);
    }
}

#[test]
fn test_health_matches_hospital() {
    assert!(ServiceCategory::Health.matches_service(ServiceType::Hospital));
    assert!(ServiceCategory::Health.matches_service(ServiceType::MedicalClinic));
    assert!(ServiceCategory::Health.matches_service(ServiceType::MedicalCenter));
}

#[test]
fn test_health_does_not_match_school() {
    assert!(!ServiceCategory::Health.matches_service(ServiceType::ElementarySchool));
}

#[test]
fn test_education_matches_schools() {
    assert!(ServiceCategory::Education.matches_service(ServiceType::ElementarySchool));
    assert!(ServiceCategory::Education.matches_service(ServiceType::HighSchool));
    assert!(ServiceCategory::Education.matches_service(ServiceType::University));
    assert!(ServiceCategory::Education.matches_service(ServiceType::Library));
    assert!(ServiceCategory::Education.matches_service(ServiceType::Kindergarten));
}

#[test]
fn test_police_matches_stations() {
    assert!(ServiceCategory::Police.matches_service(ServiceType::PoliceStation));
    assert!(ServiceCategory::Police.matches_service(ServiceType::PoliceKiosk));
    assert!(ServiceCategory::Police.matches_service(ServiceType::PoliceHQ));
    assert!(ServiceCategory::Police.matches_service(ServiceType::Prison));
}

#[test]
fn test_fire_matches_fire_services() {
    assert!(ServiceCategory::Fire.matches_service(ServiceType::FireStation));
    assert!(ServiceCategory::Fire.matches_service(ServiceType::FireHouse));
    assert!(ServiceCategory::Fire.matches_service(ServiceType::FireHQ));
}

#[test]
fn test_parks_matches_parks() {
    assert!(ServiceCategory::Parks.matches_service(ServiceType::SmallPark));
    assert!(ServiceCategory::Parks.matches_service(ServiceType::LargePark));
    assert!(ServiceCategory::Parks.matches_service(ServiceType::Playground));
}

#[test]
fn test_entertainment_matches_venues() {
    assert!(ServiceCategory::Entertainment.matches_service(ServiceType::Stadium));
    assert!(ServiceCategory::Entertainment.matches_service(ServiceType::Plaza));
    assert!(ServiceCategory::Entertainment.matches_service(ServiceType::SportsField));
    assert!(ServiceCategory::Entertainment.matches_service(ServiceType::Museum));
}

#[test]
fn test_telecom_matches_towers() {
    assert!(ServiceCategory::Telecom.matches_service(ServiceType::CellTower));
    assert!(ServiceCategory::Telecom.matches_service(ServiceType::DataCenter));
}

#[test]
fn test_transport_matches_stations() {
    assert!(ServiceCategory::Transport.matches_service(ServiceType::BusDepot));
    assert!(ServiceCategory::Transport.matches_service(ServiceType::TrainStation));
    assert!(ServiceCategory::Transport.matches_service(ServiceType::SubwayStation));
}

// =========================================================================
// Overlay mode mapping tests
// =========================================================================

#[test]
fn test_education_has_overlay() {
    assert_eq!(
        ServiceCategory::Education.overlay_mode(),
        Some(OverlayMode::Education)
    );
}

#[test]
fn test_transport_has_overlay() {
    assert_eq!(
        ServiceCategory::Transport.overlay_mode(),
        Some(OverlayMode::Traffic)
    );
}

// =========================================================================
// Coverage bit uniqueness test
// =========================================================================

#[test]
fn test_coverage_bits_are_single_bits() {
    for cat in ServiceCategory::ALL {
        let bit = cat.coverage_bit();
        // Each coverage bit should be a power of 2
        assert_eq!(
            bit.count_ones(),
            1,
            "Coverage bit for {:?} is not a single bit",
            cat
        );
    }
}

// =========================================================================
// Other service groups tests
// =========================================================================

#[test]
fn test_other_service_groups_count() {
    assert_eq!(OtherServiceGroup::ALL.len(), 7);
}

#[test]
fn test_other_group_names_non_empty() {
    for group in OtherServiceGroup::ALL {
        assert!(!group.name().is_empty());
    }
}

#[test]
fn test_garbage_group_matches() {
    let types = OtherServiceGroup::Garbage.service_types();
    assert!(types.contains(&ServiceType::Landfill));
    assert!(types.contains(&ServiceType::RecyclingCenter));
    assert!(types.contains(&ServiceType::Incinerator));
    assert!(types.contains(&ServiceType::TransferStation));
}

#[test]
fn test_death_care_group_matches() {
    let types = OtherServiceGroup::DeathCare.service_types();
    assert!(types.contains(&ServiceType::Cemetery));
    assert!(types.contains(&ServiceType::Crematorium));
}

#[test]
fn test_heating_group_matches() {
    let types = OtherServiceGroup::Heating.service_types();
    assert!(types.contains(&ServiceType::HeatingBoiler));
    assert!(types.contains(&ServiceType::DistrictHeatingPlant));
    assert!(types.contains(&ServiceType::GeothermalPlant));
}

#[test]
fn test_garbage_has_overlay() {
    assert_eq!(
        OtherServiceGroup::Garbage.overlay_mode(),
        Some(OverlayMode::Garbage)
    );
}

#[test]
fn test_water_service_has_overlay() {
    assert_eq!(
        OtherServiceGroup::WaterService.overlay_mode(),
        Some(OverlayMode::Water)
    );
}

#[test]
fn test_death_care_no_overlay() {
    assert_eq!(OtherServiceGroup::DeathCare.overlay_mode(), None);
}

// =========================================================================
// Category service_types completeness test
// =========================================================================

#[test]
fn test_category_service_types_match_matches_service() {
    for cat in ServiceCategory::ALL {
        for &st in cat.service_types() {
            assert!(
                cat.matches_service(st),
                "{:?}.matches_service({:?}) should be true",
                cat,
                st
            );
        }
    }
}

// =========================================================================
// Visibility and ExpandedCategories tests
// =========================================================================

#[test]
fn test_visibility_default_hidden() {
    let visible = ServiceCoveragePanelVisible::default();
    assert!(!visible.0);
}

#[test]
fn test_visibility_toggle() {
    let mut visible = ServiceCoveragePanelVisible::default();
    visible.0 = !visible.0;
    assert!(visible.0);
    visible.0 = !visible.0;
    assert!(!visible.0);
}

#[test]
fn test_expanded_categories_default_empty() {
    let expanded = ExpandedCategories::default();
    assert!(expanded.expanded.is_empty());
    assert!(expanded.other_expanded.is_empty());
}

#[test]
fn test_expanded_categories_toggle() {
    let mut expanded = ExpandedCategories::default();
    expanded.expanded.insert(0);
    assert!(expanded.expanded.contains(&0));
    expanded.expanded.remove(&0);
    assert!(!expanded.expanded.contains(&0));
}
