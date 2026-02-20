//! Service Coverage Detail Panel (UX-056).
//!
//! Displays a comprehensive overview of all service categories with:
//! - Coverage percentage per category computed from `ServiceCoverageGrid`
//! - Color coding: green (>80%), yellow (50-80%), red (<50%)
//! - Clickable rows to activate the corresponding overlay mode
//! - Total capacity (number of service buildings) and current demand
//!   (number of zoned/developed cells) for each category

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use rendering::overlay::{OverlayMode, OverlayState};
use simulation::config::{GRID_HEIGHT, GRID_WIDTH};
use simulation::grid::{WorldGrid, ZoneType};
use simulation::happiness::{
    ServiceCoverageGrid, COVERAGE_EDUCATION, COVERAGE_ENTERTAINMENT, COVERAGE_FIRE,
    COVERAGE_HEALTH, COVERAGE_PARK, COVERAGE_POLICE, COVERAGE_TELECOM, COVERAGE_TRANSPORT,
};
use simulation::services::{ServiceBuilding, ServiceType};

// =============================================================================
// Service categories
// =============================================================================

/// High-level service categories shown in the panel.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ServiceCategory {
    Health,
    Education,
    Police,
    Fire,
    Parks,
    Entertainment,
    Telecom,
    Transport,
}

impl ServiceCategory {
    /// All categories in display order.
    pub const ALL: [ServiceCategory; 8] = [
        ServiceCategory::Health,
        ServiceCategory::Education,
        ServiceCategory::Police,
        ServiceCategory::Fire,
        ServiceCategory::Parks,
        ServiceCategory::Entertainment,
        ServiceCategory::Telecom,
        ServiceCategory::Transport,
    ];

    /// Human-readable name.
    pub fn name(self) -> &'static str {
        match self {
            Self::Health => "Health",
            Self::Education => "Education",
            Self::Police => "Police",
            Self::Fire => "Fire",
            Self::Parks => "Parks",
            Self::Entertainment => "Entertainment",
            Self::Telecom => "Telecom",
            Self::Transport => "Transport",
        }
    }

    /// The coverage bitmask corresponding to this category.
    pub fn coverage_bit(self) -> u8 {
        match self {
            Self::Health => COVERAGE_HEALTH,
            Self::Education => COVERAGE_EDUCATION,
            Self::Police => COVERAGE_POLICE,
            Self::Fire => COVERAGE_FIRE,
            Self::Parks => COVERAGE_PARK,
            Self::Entertainment => COVERAGE_ENTERTAINMENT,
            Self::Telecom => COVERAGE_TELECOM,
            Self::Transport => COVERAGE_TRANSPORT,
        }
    }

    /// The overlay mode activated when clicking this category.
    pub fn overlay_mode(self) -> Option<OverlayMode> {
        match self {
            Self::Education => Some(OverlayMode::Education),
            Self::Police => Some(OverlayMode::Pollution), // closest available
            Self::Fire => Some(OverlayMode::Power),       // closest available
            Self::Transport => Some(OverlayMode::Traffic),
            Self::Parks => Some(OverlayMode::LandValue),
            _ => None,
        }
    }

    /// Returns true if the given `ServiceType` belongs to this category.
    pub fn matches_service(self, st: ServiceType) -> bool {
        match self {
            Self::Health => ServiceBuilding::is_health(st),
            Self::Education => ServiceBuilding::is_education(st),
            Self::Police => ServiceBuilding::is_police(st),
            Self::Fire => ServiceBuilding::is_fire(st),
            Self::Parks => ServiceBuilding::is_park(st),
            Self::Entertainment => matches!(
                st,
                ServiceType::Stadium
                    | ServiceType::Plaza
                    | ServiceType::SportsField
                    | ServiceType::Museum
                    | ServiceType::Cathedral
                    | ServiceType::TVStation
            ),
            Self::Telecom => ServiceBuilding::is_telecom(st),
            Self::Transport => ServiceBuilding::is_transport(st),
        }
    }
}

// =============================================================================
// Visibility resource
// =============================================================================

/// Resource controlling whether the service coverage panel is visible.
/// Toggle with 'J' key.
#[derive(Resource, Default)]
pub struct ServiceCoveragePanelVisible(pub bool);

// =============================================================================
// Computed coverage data (updated each frame the panel is visible)
// =============================================================================

/// Per-category coverage statistics.
#[derive(Debug, Clone, Default)]
pub struct CategoryStats {
    /// Percentage of developed cells covered (0.0..1.0).
    pub coverage_pct: f64,
    /// Number of service buildings in this category (capacity proxy).
    pub building_count: u32,
    /// Number of developed/zoned cells that want service (demand proxy).
    pub demand_cells: u32,
    /// Number of those demand cells that are covered.
    pub covered_cells: u32,
}

// =============================================================================
// Coverage computation
// =============================================================================

/// Computes the coverage percentage for a single category.
///
/// Coverage = (developed cells with the category's coverage bit set) / (total developed cells).
/// "Developed" means the cell has a non-None zone type.
pub fn compute_category_stats(
    category: ServiceCategory,
    grid: &WorldGrid,
    coverage: &ServiceCoverageGrid,
    services: &[&ServiceBuilding],
) -> CategoryStats {
    let bit = category.coverage_bit();
    let mut demand_cells: u32 = 0;
    let mut covered_cells: u32 = 0;

    for y in 0..GRID_HEIGHT {
        for x in 0..GRID_WIDTH {
            let cell = grid.get(x, y);
            if cell.zone == ZoneType::None {
                continue;
            }
            demand_cells += 1;
            let idx = ServiceCoverageGrid::idx(x, y);
            if coverage.flags[idx] & bit != 0 {
                covered_cells += 1;
            }
        }
    }

    let coverage_pct = if demand_cells > 0 {
        covered_cells as f64 / demand_cells as f64
    } else {
        0.0
    };

    let building_count = services
        .iter()
        .filter(|s| category.matches_service(s.service_type))
        .count() as u32;

    CategoryStats {
        coverage_pct,
        building_count,
        demand_cells,
        covered_cells,
    }
}

// =============================================================================
// Color helpers
// =============================================================================

/// Returns the egui color for a coverage percentage.
/// Green (>80%), yellow (50-80%), red (<50%).
pub fn coverage_color(pct: f64) -> egui::Color32 {
    if pct > 0.80 {
        egui::Color32::from_rgb(80, 200, 80) // green
    } else if pct >= 0.50 {
        egui::Color32::from_rgb(220, 200, 50) // yellow
    } else {
        egui::Color32::from_rgb(255, 60, 60) // red
    }
}

/// Returns a label describing the coverage level.
pub fn coverage_label(pct: f64) -> &'static str {
    if pct > 0.80 {
        "Good"
    } else if pct >= 0.50 {
        "Moderate"
    } else {
        "Poor"
    }
}

// =============================================================================
// Keybind system
// =============================================================================

// =============================================================================
// Panel UI system
// =============================================================================

/// Renders the service coverage detail panel.
pub fn service_coverage_panel_ui(
    mut contexts: EguiContexts,
    visible: Res<ServiceCoveragePanelVisible>,
    grid: Res<WorldGrid>,
    coverage: Res<ServiceCoverageGrid>,
    services: Query<&ServiceBuilding>,
    mut overlay: ResMut<OverlayState>,
) {
    if !visible.0 {
        return;
    }

    let service_list: Vec<&ServiceBuilding> = services.iter().collect();

    egui::Window::new("Service Coverage")
        .default_open(true)
        .default_width(380.0)
        .show(contexts.ctx_mut(), |ui| {
            ui.small("Service coverage panel");
            ui.separator();

            // Compute overall stats
            let mut total_demand: u32 = 0;
            let mut total_covered_all: u32 = 0;

            // We need demand cells count (computed once since it's same for all categories)
            let mut demand_count: u32 = 0;
            for y in 0..GRID_HEIGHT {
                for x in 0..GRID_WIDTH {
                    if grid.get(x, y).zone != ZoneType::None {
                        demand_count += 1;
                    }
                }
            }

            // Header row
            ui.heading("Coverage by Service Type");
            ui.small(format!("{} zoned cells in city", demand_count));
            ui.separator();

            // Table header
            egui::Grid::new("service_coverage_grid")
                .num_columns(5)
                .striped(true)
                .min_col_width(60.0)
                .show(ui, |ui| {
                    ui.strong("Service");
                    ui.strong("Coverage");
                    ui.strong("Status");
                    ui.strong("Buildings");
                    ui.strong("Covered/Demand");
                    ui.end_row();

                    for category in ServiceCategory::ALL {
                        let stats =
                            compute_category_stats(category, &grid, &coverage, &service_list);

                        total_demand += stats.demand_cells;
                        total_covered_all += stats.covered_cells;

                        let color = coverage_color(stats.coverage_pct);
                        let pct_str = format!("{:.1}%", stats.coverage_pct * 100.0);
                        let label = coverage_label(stats.coverage_pct);

                        // Clickable service name to activate overlay
                        let name_response = ui.add(
                            egui::Label::new(egui::RichText::new(category.name()).strong())
                                .sense(egui::Sense::click()),
                        );

                        if name_response.clicked() {
                            if let Some(mode) = category.overlay_mode() {
                                if overlay.mode == mode {
                                    overlay.mode = OverlayMode::None;
                                } else {
                                    overlay.mode = mode;
                                }
                            }
                        }
                        if name_response.hovered() {
                            name_response.on_hover_text(match category.overlay_mode() {
                                Some(_) => "Click to toggle overlay",
                                None => "No overlay available",
                            });
                        }

                        // Coverage percentage with color
                        ui.colored_label(color, &pct_str);

                        // Status label
                        ui.colored_label(color, label);

                        // Building count (capacity)
                        ui.label(format!("{}", stats.building_count));

                        // Covered / demand
                        ui.label(format!("{} / {}", stats.covered_cells, stats.demand_cells));

                        ui.end_row();
                    }
                });

            ui.separator();

            // Overall summary
            let overall_pct = if total_demand > 0 {
                total_covered_all as f64 / total_demand as f64
            } else {
                0.0
            };
            let overall_color = coverage_color(overall_pct);

            ui.horizontal(|ui| {
                ui.strong("Overall Average:");
                ui.colored_label(
                    overall_color,
                    format!(
                        "{:.1}% ({})",
                        overall_pct * 100.0,
                        coverage_label(overall_pct)
                    ),
                );
            });

            ui.horizontal(|ui| {
                ui.strong("Total Service Buildings:");
                ui.label(format!("{}", service_list.len()));
            });

            // Active overlay indicator
            if overlay.mode != OverlayMode::None {
                ui.separator();
                ui.colored_label(
                    egui::Color32::from_rgb(100, 180, 255),
                    format!("Active overlay: {}", overlay.mode.label()),
                );
            }
        });
}

// =============================================================================
// Plugin
// =============================================================================

pub struct ServiceCoveragePanelPlugin;

impl Plugin for ServiceCoveragePanelPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ServiceCoveragePanelVisible>()
            .add_systems(Update, service_coverage_panel_ui);
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

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
    // Coverage computation tests
    // =========================================================================

    #[test]
    fn test_coverage_zero_demand() {
        let grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        let coverage = ServiceCoverageGrid::default();
        let services: Vec<&ServiceBuilding> = vec![];

        let stats = compute_category_stats(ServiceCategory::Health, &grid, &coverage, &services);

        assert_eq!(stats.demand_cells, 0);
        assert_eq!(stats.covered_cells, 0);
        assert_eq!(stats.building_count, 0);
        assert!((stats.coverage_pct - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_coverage_with_demand_no_coverage() {
        let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        // Zone a few cells
        grid.get_mut(10, 10).zone = ZoneType::ResidentialLow;
        grid.get_mut(11, 10).zone = ZoneType::CommercialLow;
        grid.get_mut(12, 10).zone = ZoneType::Industrial;

        let coverage = ServiceCoverageGrid::default();
        let services: Vec<&ServiceBuilding> = vec![];

        let stats = compute_category_stats(ServiceCategory::Health, &grid, &coverage, &services);

        assert_eq!(stats.demand_cells, 3);
        assert_eq!(stats.covered_cells, 0);
        assert!((stats.coverage_pct - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_coverage_full_coverage() {
        let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        grid.get_mut(10, 10).zone = ZoneType::ResidentialLow;
        grid.get_mut(11, 10).zone = ZoneType::CommercialLow;

        let mut coverage = ServiceCoverageGrid::default();
        // Set health coverage on both cells
        let idx1 = ServiceCoverageGrid::idx(10, 10);
        let idx2 = ServiceCoverageGrid::idx(11, 10);
        coverage.flags[idx1] |= COVERAGE_HEALTH;
        coverage.flags[idx2] |= COVERAGE_HEALTH;

        let services: Vec<&ServiceBuilding> = vec![];

        let stats = compute_category_stats(ServiceCategory::Health, &grid, &coverage, &services);

        assert_eq!(stats.demand_cells, 2);
        assert_eq!(stats.covered_cells, 2);
        assert!((stats.coverage_pct - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_coverage_partial_coverage() {
        let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        grid.get_mut(10, 10).zone = ZoneType::ResidentialLow;
        grid.get_mut(11, 10).zone = ZoneType::CommercialLow;
        grid.get_mut(12, 10).zone = ZoneType::Industrial;
        grid.get_mut(13, 10).zone = ZoneType::Office;

        let mut coverage = ServiceCoverageGrid::default();
        // Cover 2 of 4 cells
        coverage.flags[ServiceCoverageGrid::idx(10, 10)] |= COVERAGE_POLICE;
        coverage.flags[ServiceCoverageGrid::idx(12, 10)] |= COVERAGE_POLICE;

        let services: Vec<&ServiceBuilding> = vec![];

        let stats = compute_category_stats(ServiceCategory::Police, &grid, &coverage, &services);

        assert_eq!(stats.demand_cells, 4);
        assert_eq!(stats.covered_cells, 2);
        assert!((stats.coverage_pct - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_coverage_counts_buildings() {
        let grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        let coverage = ServiceCoverageGrid::default();

        let hospital = ServiceBuilding {
            service_type: ServiceType::Hospital,
            grid_x: 10,
            grid_y: 10,
            radius: 400.0,
        };
        let clinic = ServiceBuilding {
            service_type: ServiceType::MedicalClinic,
            grid_x: 20,
            grid_y: 20,
            radius: 192.0,
        };
        let school = ServiceBuilding {
            service_type: ServiceType::ElementarySchool,
            grid_x: 30,
            grid_y: 30,
            radius: 240.0,
        };

        let services: Vec<&ServiceBuilding> = vec![&hospital, &clinic, &school];

        let health_stats =
            compute_category_stats(ServiceCategory::Health, &grid, &coverage, &services);
        assert_eq!(health_stats.building_count, 2); // hospital + clinic

        let edu_stats =
            compute_category_stats(ServiceCategory::Education, &grid, &coverage, &services);
        assert_eq!(edu_stats.building_count, 1); // school only
    }

    // =========================================================================
    // Visibility tests
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
    // Cross-category isolation test
    // =========================================================================

    #[test]
    fn test_coverage_only_counts_matching_bit() {
        let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        grid.get_mut(5, 5).zone = ZoneType::ResidentialLow;

        let mut coverage = ServiceCoverageGrid::default();
        // Set only FIRE coverage
        coverage.flags[ServiceCoverageGrid::idx(5, 5)] |= COVERAGE_FIRE;

        let services: Vec<&ServiceBuilding> = vec![];

        // Fire should show 100%
        let fire_stats = compute_category_stats(ServiceCategory::Fire, &grid, &coverage, &services);
        assert_eq!(fire_stats.covered_cells, 1);
        assert!((fire_stats.coverage_pct - 1.0).abs() < f64::EPSILON);

        // Health should show 0%
        let health_stats =
            compute_category_stats(ServiceCategory::Health, &grid, &coverage, &services);
        assert_eq!(health_stats.covered_cells, 0);
        assert!((health_stats.coverage_pct - 0.0).abs() < f64::EPSILON);
    }
}
