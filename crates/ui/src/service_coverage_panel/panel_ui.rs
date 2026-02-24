//! Service coverage panel UI system, keybind, and plugin.

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use rendering::overlay::{OverlayMode, OverlayState};
use simulation::config::{GRID_HEIGHT, GRID_WIDTH};
use simulation::grid::{WorldGrid, ZoneType};
use simulation::happiness::ServiceCoverageGrid;
use simulation::services::ServiceBuilding;
use simulation::SaveLoadState;

use super::categories::{OtherServiceGroup, ServiceCategory};
use super::stats::{
    compute_category_stats, compute_service_type_stats, coverage_color, coverage_label,
};

// =============================================================================
// Visibility resource
// =============================================================================

/// Resource controlling whether the service coverage panel is visible.
/// Toggle with 'K' key.
#[derive(Resource, Default)]
pub struct ServiceCoveragePanelVisible(pub bool);

// =============================================================================
// Expanded rows state
// =============================================================================

/// Resource tracking which categories are expanded to show per-service-type detail.
#[derive(Resource, Default)]
pub struct ExpandedCategories {
    pub expanded: std::collections::HashSet<u8>,
    pub other_expanded: std::collections::HashSet<u8>,
}

// =============================================================================
// Keybind system
// =============================================================================

/// Toggles service coverage panel with K key.
pub fn service_coverage_keybind(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut visible: ResMut<ServiceCoveragePanelVisible>,
    mut contexts: EguiContexts,
) {
    if contexts.ctx_mut().wants_keyboard_input() {
        return;
    }
    if keyboard.just_pressed(KeyCode::KeyK) {
        visible.0 = !visible.0;
    }
}

// =============================================================================
// Panel UI system
// =============================================================================

/// Renders the service coverage detail panel.
#[allow(clippy::too_many_arguments)]
pub fn service_coverage_panel_ui(
    mut contexts: EguiContexts,
    visible: Res<ServiceCoveragePanelVisible>,
    grid: Res<WorldGrid>,
    coverage: Res<ServiceCoverageGrid>,
    services: Query<&ServiceBuilding>,
    mut overlay: ResMut<OverlayState>,
    mut expanded: ResMut<ExpandedCategories>,
) {
    if !visible.0 {
        return;
    }

    let service_list: Vec<&ServiceBuilding> = services.iter().collect();

    egui::Window::new("Service Coverage")
        .default_open(true)
        .default_width(400.0)
        .show(contexts.ctx_mut(), |ui| {
            ui.small("Per-service coverage and utilization (K to toggle)");
            ui.separator();

            // Compute demand cells count (same for all categories)
            let mut demand_count: u32 = 0;
            for y in 0..GRID_HEIGHT {
                for x in 0..GRID_WIDTH {
                    if grid.get(x, y).zone != ZoneType::None {
                        demand_count += 1;
                    }
                }
            }

            // Header
            ui.heading("Coverage by Service Type");
            ui.small(format!("{} zoned cells in city", demand_count));
            ui.separator();

            let mut total_maintenance = 0.0f64;
            let mut total_covered_all: u32 = 0;
            let mut total_demand: u32 = 0;

            // --- Coverage-tracked categories ---
            for (cat_idx, category) in ServiceCategory::ALL.iter().enumerate() {
                let stats = compute_category_stats(*category, &grid, &coverage, &service_list);

                total_demand += stats.demand_cells;
                total_covered_all += stats.covered_cells;
                total_maintenance += stats.monthly_maintenance;

                let color = coverage_color(stats.coverage_pct);
                let pct_str = format!("{:.1}%", stats.coverage_pct * 100.0);
                let label = coverage_label(stats.coverage_pct);
                let is_expanded = expanded.expanded.contains(&(cat_idx as u8));

                // Category header row
                ui.horizontal(|ui| {
                    let arrow = if is_expanded { "\u{25BC}" } else { "\u{25B6}" };
                    let header_text = format!(
                        "{} {}  {}  {}  ({} bldgs, ${:.0}/mo)",
                        arrow,
                        category.name(),
                        pct_str,
                        label,
                        stats.building_count,
                        stats.monthly_maintenance
                    );

                    let resp = ui.add(
                        egui::Label::new(egui::RichText::new(header_text).strong().color(color))
                            .sense(egui::Sense::click()),
                    );

                    if resp.clicked() {
                        if is_expanded {
                            expanded.expanded.remove(&(cat_idx as u8));
                        } else {
                            expanded.expanded.insert(cat_idx as u8);
                        }
                    }

                    // Overlay toggle on right-click
                    if resp.secondary_clicked() {
                        if let Some(mode) = category.overlay_mode() {
                            if overlay.mode == mode {
                                overlay.mode = OverlayMode::None;
                            } else {
                                overlay.mode = mode;
                            }
                        }
                    }

                    resp.on_hover_text(format!(
                        "Covered: {} / {} cells\nClick to expand, right-click to toggle overlay",
                        stats.covered_cells, stats.demand_cells
                    ));
                });

                // Coverage bar
                ui.horizontal(|ui| {
                    ui.add_space(16.0);
                    let bar_width = 200.0;
                    let bar_height = 6.0;
                    let desired = egui::vec2(bar_width, bar_height);
                    let (rect, _) = ui.allocate_exact_size(desired, egui::Sense::hover());
                    let painter = ui.painter();
                    painter.rect_filled(rect, 2.0, egui::Color32::from_gray(40));
                    let fraction = stats.coverage_pct as f32;
                    let mut fill_rect = rect;
                    fill_rect.set_right(rect.left() + rect.width() * fraction.clamp(0.0, 1.0));
                    painter.rect_filled(fill_rect, 2.0, color);
                });

                // Expanded per-service-type detail
                if is_expanded {
                    for &st in category.service_types() {
                        let type_stats = compute_service_type_stats(st, &service_list);
                        if type_stats.count > 0 {
                            ui.horizontal(|ui| {
                                ui.add_space(24.0);
                                ui.label(
                                    egui::RichText::new(format!(
                                        "{}: {} bldg(s), ${:.0}/mo",
                                        st.name(),
                                        type_stats.count,
                                        type_stats.monthly_maintenance
                                    ))
                                    .small(),
                                );
                            });
                        }
                    }

                    // Show types with 0 count dimmed
                    let has_zero = category
                        .service_types()
                        .iter()
                        .any(|st| compute_service_type_stats(*st, &service_list).count == 0);
                    if has_zero {
                        ui.horizontal(|ui| {
                            ui.add_space(24.0);
                            let missing: Vec<&str> = category
                                .service_types()
                                .iter()
                                .filter(|st| {
                                    compute_service_type_stats(**st, &service_list).count == 0
                                })
                                .map(|st| st.name())
                                .collect();
                            ui.label(
                                egui::RichText::new(format!("Not built: {}", missing.join(", ")))
                                    .small()
                                    .weak(),
                            );
                        });
                    }
                }

                ui.add_space(2.0);
            }

            ui.separator();

            // --- Other Services (no coverage tracking) ---
            ui.heading("Other Services");
            ui.small("Services without area coverage tracking");
            ui.add_space(4.0);

            let mut other_maintenance = 0.0f64;

            for (group_idx, group) in OtherServiceGroup::ALL.iter().enumerate() {
                let mut group_count = 0u32;
                let mut group_maintenance = 0.0f64;

                for &st in group.service_types() {
                    let type_stats = compute_service_type_stats(st, &service_list);
                    group_count += type_stats.count;
                    group_maintenance += type_stats.monthly_maintenance;
                }

                other_maintenance += group_maintenance;

                if group_count == 0 {
                    // Show dimmed row for empty groups
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new(format!("{}: none", group.name())).weak());
                    });
                    continue;
                }

                let is_expanded = expanded.other_expanded.contains(&(group_idx as u8));
                let arrow = if is_expanded { "\u{25BC}" } else { "\u{25B6}" };

                let resp = ui.add(
                    egui::Label::new(
                        egui::RichText::new(format!(
                            "{} {}  ({} bldgs, ${:.0}/mo)",
                            arrow,
                            group.name(),
                            group_count,
                            group_maintenance
                        ))
                        .strong(),
                    )
                    .sense(egui::Sense::click()),
                );

                if resp.clicked() {
                    if is_expanded {
                        expanded.other_expanded.remove(&(group_idx as u8));
                    } else {
                        expanded.other_expanded.insert(group_idx as u8);
                    }
                }

                if resp.secondary_clicked() {
                    if let Some(mode) = group.overlay_mode() {
                        if overlay.mode == mode {
                            overlay.mode = OverlayMode::None;
                        } else {
                            overlay.mode = mode;
                        }
                    }
                }

                if is_expanded {
                    for &st in group.service_types() {
                        let type_stats = compute_service_type_stats(st, &service_list);
                        ui.horizontal(|ui| {
                            ui.add_space(24.0);
                            if type_stats.count > 0 {
                                ui.label(
                                    egui::RichText::new(format!(
                                        "{}: {} bldg(s), ${:.0}/mo",
                                        st.name(),
                                        type_stats.count,
                                        type_stats.monthly_maintenance
                                    ))
                                    .small(),
                                );
                            } else {
                                ui.label(
                                    egui::RichText::new(format!("{}: not built", st.name()))
                                        .small()
                                        .weak(),
                                );
                            }
                        });
                    }
                }

                ui.add_space(2.0);
            }

            ui.separator();

            // --- Overall summary ---
            total_maintenance += other_maintenance;

            let overall_pct = if total_demand > 0 {
                total_covered_all as f64 / total_demand as f64
            } else {
                0.0
            };
            let overall_color = coverage_color(overall_pct);

            ui.horizontal(|ui| {
                ui.strong("Overall Coverage:");
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

            ui.horizontal(|ui| {
                ui.strong("Total Monthly Maintenance:");
                ui.colored_label(
                    egui::Color32::from_rgb(220, 180, 80),
                    format!("${:.0}", total_maintenance),
                );
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
            .init_resource::<ExpandedCategories>()
            .add_systems(
                Update,
                (service_coverage_keybind, service_coverage_panel_ui)
                    .run_if(in_state(SaveLoadState::Idle)),
            );
    }
}
