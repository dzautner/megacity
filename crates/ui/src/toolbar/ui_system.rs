use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use simulation::bankruptcy_warning::{BankruptcyLevel, BankruptcyState};
use simulation::budget::ExtendedBudget;
use simulation::economy::CityBudget;
use simulation::stats::CityStats;
use simulation::time_of_day::GameClock;
use simulation::unlocks::UnlockState;
use simulation::weather::Weather;
use simulation::zones::ZoneDemand;

use rendering::input::{ActiveTool, GridSnap, StatusMessage};
use rendering::overlay::{DualOverlayMode, DualOverlayState, OverlayMode, OverlayState};
use save::NewGameEvent;

use crate::save_slot_ui::SaveSlotUiState;

use super::catalog::unlock_filter;
use super::catalog::{show_tool_tooltip, DashboardKind, OpenCategory, ToolCatalog};
use super::widgets::{format_pop, milestone_name, rci_demand_bars, speed_button};

use crate::energy_dashboard::EnergyDashboardVisible;
use crate::waste_dashboard::WasteDashboardVisible;
use crate::water_dashboard::WaterDashboardVisible;

/// Return the color for the treasury label based on the current bankruptcy level.
fn treasury_color(level: BankruptcyLevel) -> egui::Color32 {
    match level {
        BankruptcyLevel::Bankrupt | BankruptcyLevel::Critical => {
            egui::Color32::from_rgb(220, 60, 60)
        }
        BankruptcyLevel::Warning => egui::Color32::from_rgb(230, 200, 50),
        BankruptcyLevel::Normal => egui::Color32::from_rgb(200, 200, 200),
    }
}

/// Toggle a dashboard's visibility resource by kind.
fn toggle_dashboard(
    kind: DashboardKind,
    energy: &mut ResMut<EnergyDashboardVisible>,
    water: &mut ResMut<WaterDashboardVisible>,
    waste: &mut ResMut<WasteDashboardVisible>,
) {
    match kind {
        DashboardKind::Energy => energy.0 = !energy.0,
        DashboardKind::Water => water.0 = !water.0,
        DashboardKind::Waste => waste.0 = !waste.0,
    }
}

/// Check if a dashboard is currently visible.
fn is_dashboard_visible(
    kind: DashboardKind,
    energy: &EnergyDashboardVisible,
    water: &WaterDashboardVisible,
    waste: &WasteDashboardVisible,
) -> bool {
    match kind {
        DashboardKind::Energy => energy.0,
        DashboardKind::Water => water.0,
        DashboardKind::Waste => waste.0,
    }
}

// ---------------------------------------------------------------------------
// Main toolbar system
// ---------------------------------------------------------------------------

#[allow(clippy::too_many_arguments)]
pub fn toolbar_ui(
    mut contexts: EguiContexts,
    mut tool: ResMut<ActiveTool>,
    mut clock: ResMut<GameClock>,
    stats: Res<CityStats>,
    budget: Res<CityBudget>,
    demand: Res<ZoneDemand>,
    overlay_params: (ResMut<OverlayState>, Res<DualOverlayState>),
    status: Res<StatusMessage>,
    mut slot_ui: ResMut<SaveSlotUiState>,
    mut new_game_events: EventWriter<NewGameEvent>,
    mut open_cat: ResMut<OpenCategory>,
    weather_snap: (Res<Weather>, Res<GridSnap>),
    extended_budget: Res<ExtendedBudget>,
    catalog_unlocks_bankruptcy: (Res<ToolCatalog>, Res<UnlockState>, Res<BankruptcyState>),
    mut dashboard_vis: (
        ResMut<EnergyDashboardVisible>,
        ResMut<WaterDashboardVisible>,
        ResMut<WasteDashboardVisible>,
    ),
) {
    let (mut overlay, dual_overlay) = overlay_params;
    let (catalog, unlocks, bankruptcy) = catalog_unlocks_bankruptcy;
    let (weather, grid_snap) = weather_snap;
    let categories = &catalog.categories;
    let current_pop = stats.population;

    // Set tooltip delay to 300ms for tool tooltips
    contexts
        .ctx_mut()
        .style_mut(|style| style.interaction.tooltip_delay = 0.3);

    // ---- Top info bar ----
    egui::TopBottomPanel::top("top_info_bar")
        .exact_height(36.0)
        .show(contexts.ctx_mut(), |ui| {
            ui.horizontal_centered(|ui| {
                ui.spacing_mut().item_spacing.x = 12.0;

                // Milestone name
                let name = milestone_name(stats.population);
                ui.label(
                    egui::RichText::new(name)
                        .strong()
                        .color(egui::Color32::from_rgb(180, 200, 240)),
                );

                ui.separator();

                // Population
                ui.label(format!("Pop: {}", format_pop(stats.population)));

                ui.separator();

                // RCI Demand Bars
                rci_demand_bars(ui, &demand);

                ui.separator();

                // Money — colored by bankruptcy level
                let money_color = treasury_color(bankruptcy.level);
                ui.label(
                    egui::RichText::new(format!("${:.0}", budget.treasury)).color(money_color),
                );

                // Net income indicator
                {
                    let net = budget.monthly_income - budget.monthly_expenses;
                    let (sign, color) = if net >= 0.0 {
                        ("+", egui::Color32::from_rgb(80, 200, 80))
                    } else {
                        ("", egui::Color32::from_rgb(220, 60, 60))
                    };
                    let label_text =
                        egui::RichText::new(format!("{}${:.0}/mo", sign, net)).color(color);
                    let resp = ui.label(label_text);
                    let ib = &extended_budget.income_breakdown;
                    let eb = &extended_budget.expense_breakdown;
                    let total_income = budget.monthly_income;
                    let total_expenses = budget.monthly_expenses;
                    resp.on_hover_ui(|ui| {
                        ui.heading("Monthly Budget");
                        ui.separator();
                        ui.label(
                            egui::RichText::new(format!("Income: ${:.0}", total_income))
                                .color(egui::Color32::from_rgb(80, 200, 80)),
                        );
                        ui.indent("income_details", |ui| {
                            ui.label(format!("Residential Tax: ${:.0}", ib.residential_tax));
                            ui.label(format!("Commercial Tax: ${:.0}", ib.commercial_tax));
                            ui.label(format!("Industrial Tax: ${:.0}", ib.industrial_tax));
                            ui.label(format!("Office Tax: ${:.0}", ib.office_tax));
                            ui.label(format!("Tourism: ${:.0}", ib.trade_income));
                        });
                        ui.separator();
                        ui.label(
                            egui::RichText::new(format!("Expenses: ${:.0}", total_expenses))
                                .color(egui::Color32::from_rgb(220, 60, 60)),
                        );
                        ui.indent("expense_details", |ui| {
                            ui.label(format!("Road Maintenance: ${:.0}", eb.road_maintenance));
                            ui.label(format!("Service Costs: ${:.0}", eb.service_costs));
                            ui.label(format!("Policy Costs: ${:.0}", eb.policy_costs));
                            ui.label(format!("Loan Payments: ${:.0}", eb.loan_payments));
                            ui.label(format!("Power Fuel: ${:.0}", eb.fuel_costs));
                        });
                        ui.separator();
                        ui.label(
                            egui::RichText::new(format!("Net: {}${:.0}/mo", sign, net))
                                .strong()
                                .color(color),
                        );
                    });
                }

                ui.separator();

                // Day / time / season
                ui.label(format!("{} | {}", clock.formatted(), weather.season.name()));

                // Speed controls with color-coded indicators
                // Colors: red = paused, green = 1x, yellow = 2x, orange = 4x
                let pause_color = egui::Color32::from_rgb(220, 60, 60);
                let speed1_color = egui::Color32::from_rgb(60, 200, 60);
                let speed2_color = egui::Color32::from_rgb(230, 220, 50);
                let speed4_color = egui::Color32::from_rgb(240, 160, 40);

                let pause_active = clock.paused;
                let speed1_active = !clock.paused && clock.speed == 1.0;
                let speed2_active = !clock.paused && clock.speed == 2.0;
                let speed4_active = !clock.paused && clock.speed == 4.0;

                if speed_button(ui, "||", pause_active, pause_color).clicked() {
                    clock.paused = !clock.paused;
                }
                if speed_button(ui, "1x", speed1_active, speed1_color).clicked() {
                    clock.speed = 1.0;
                    clock.paused = false;
                }
                if speed_button(ui, "2x", speed2_active, speed2_color).clicked() {
                    clock.speed = 2.0;
                    clock.paused = false;
                }
                if speed_button(ui, "4x", speed4_active, speed4_color).clicked() {
                    clock.speed = 4.0;
                    clock.paused = false;
                }

                ui.separator();

                // Happiness
                ui.label(format!("Happy: {:.0}%", stats.average_happiness));

                ui.separator();

                // Save / Load / New Game — Save/Load open slot picker dialogs
                if ui.button("New").clicked() {
                    new_game_events.send(NewGameEvent);
                }
                if ui.button("Save").clicked() {
                    slot_ui.save_dialog_open = true;
                    slot_ui.save_name_input = String::new();
                    slot_ui.confirm_overwrite = None;
                }
                if ui.button("Load").clicked() {
                    slot_ui.load_dialog_open = true;
                    slot_ui.confirm_delete = None;
                }

                // Current overlay
                if overlay.mode != OverlayMode::None {
                    ui.separator();
                    if dual_overlay.secondary != OverlayMode::None {
                        let mode_label = match dual_overlay.mode {
                            DualOverlayMode::Blend => {
                                format!(
                                    "{} + {} (Blend {:.0}%)",
                                    overlay.mode.label(),
                                    dual_overlay.secondary.label(),
                                    dual_overlay.blend_factor * 100.0
                                )
                            }
                            DualOverlayMode::Split => {
                                format!(
                                    "{} | {} (Split)",
                                    overlay.mode.label(),
                                    dual_overlay.secondary.label()
                                )
                            }
                        };
                        ui.label(
                            egui::RichText::new(format!("Overlay: {}", mode_label))
                                .color(egui::Color32::from_rgb(140, 220, 255)),
                        );
                    } else {
                        ui.label(
                            egui::RichText::new(format!("Overlay: {}", overlay.mode.label()))
                                .color(egui::Color32::from_rgb(140, 220, 255)),
                        );
                    }
                }

                // Active tool + cost
                if let Some(cost) = tool.cost() {
                    ui.separator();
                    ui.label(format!("{}: ${:.0}", tool.label(), cost));
                } else {
                    ui.separator();
                    ui.label(tool.label());
                }

                // Grid snap indicator
                if grid_snap.enabled {
                    ui.separator();
                    ui.label(
                        egui::RichText::new("[GRID SNAP]")
                            .strong()
                            .color(egui::Color32::from_rgb(100, 255, 100)),
                    );
                }
            });
        });

    // ---- Floating toast for status messages ----
    if status.active() {
        let color = if status.is_error {
            egui::Color32::from_rgb(220, 60, 50)
        } else {
            egui::Color32::from_rgb(60, 200, 80)
        };
        egui::Area::new(egui::Id::new("status_toast"))
            .fixed_pos(egui::pos2(
                contexts.ctx_mut().screen_rect().center().x - 100.0,
                42.0,
            ))
            .show(contexts.ctx_mut(), |ui| {
                egui::Frame::popup(ui.style())
                    .fill(egui::Color32::from_rgba_premultiplied(30, 30, 30, 220))
                    .show(ui, |ui| {
                        ui.colored_label(color, &status.text);
                    });
            });
    }

    // ---- Bottom toolbar: category buttons with full names ----
    let bottom_resp = egui::TopBottomPanel::bottom("bottom_toolbar")
        .exact_height(36.0)
        .show(contexts.ctx_mut(), |ui| {
            ui.horizontal_centered(|ui| {
                ui.spacing_mut().item_spacing.x = 6.0;

                for (idx, cat) in categories.iter().enumerate() {
                    let is_open = open_cat.0 == Some(idx);
                    let btn = ui.selectable_label(is_open, egui::RichText::new(cat.name).strong());
                    if btn.clicked() {
                        if is_open {
                            open_cat.0 = None;
                        } else {
                            open_cat.0 = Some(idx);
                        }
                    }
                }
            });
        });

    // ---- Category popup: compact horizontal strip just above the bottom bar ----
    if let Some(cat_idx) = open_cat.0 {
        if cat_idx < categories.len() {
            let cat = &categories[cat_idx];
            let bottom_rect = bottom_resp.response.rect;
            let screen_width = contexts.ctx_mut().screen_rect().width();

            let mut should_close = false;

            egui::Area::new(egui::Id::new("category_popup"))
                .fixed_pos(egui::pos2(0.0, bottom_rect.top() - 2.0))
                .pivot(egui::Align2::LEFT_BOTTOM)
                .show(contexts.ctx_mut(), |ui| {
                    egui::Frame::popup(ui.style())
                        .inner_margin(egui::Margin::symmetric(8, 4))
                        .show(ui, |ui| {
                            ui.set_min_width(screen_width - 16.0);
                            ui.horizontal(|ui| {
                                ui.spacing_mut().item_spacing.x = 4.0;

                                // Category label
                                ui.label(egui::RichText::new(cat.name).strong().size(12.0));
                                ui.separator();

                                // All items in a single horizontal row
                                for item in cat.items.iter() {
                                    // Check unlock state for this item
                                    let progress = item.tool.as_ref().and_then(|t| {
                                        unlock_filter::unlock_progress(t, &unlocks, current_pop)
                                    });
                                    let is_locked = progress.is_some();

                                    let label_text = if let Some(cost) = item.cost {
                                        format!("{} ${:.0}", item.name, cost)
                                    } else {
                                        item.name.to_string()
                                    };

                                    let is_active = match item.tool {
                                        Some(ref t) => *tool == *t,
                                        None => match item.overlay {
                                            Some(ov) => overlay.mode == ov,
                                            None => match item.dashboard {
                                                Some(dk) => is_dashboard_visible(
                                                    dk,
                                                    &dashboard_vis.0,
                                                    &dashboard_vis.1,
                                                    &dashboard_vis.2,
                                                ),
                                                None => false,
                                            },
                                        },
                                    };

                                    if is_locked {
                                        // Locked item color: amber when
                                        // close (>80%), gray otherwise
                                        let nearly =
                                            progress.as_ref().is_some_and(|p| p.nearly_unlocked);
                                        let text_color = if nearly {
                                            egui::Color32::from_rgb(255, 180, 60)
                                        } else {
                                            egui::Color32::from_rgb(100, 100, 100)
                                        };
                                        let response = ui
                                            .add(
                                                egui::Label::new(
                                                    egui::RichText::new(&label_text)
                                                        .size(11.0)
                                                        .color(text_color),
                                                )
                                                .sense(egui::Sense::hover()),
                                            )
                                            .on_hover_ui(|tip| {
                                                show_tool_tooltip(tip, item, progress.as_ref());
                                            });

                                        // Swallow clicks on locked items
                                        let _ = response;
                                    } else {
                                        // Normal unlocked item
                                        let response = ui
                                            .selectable_label(
                                                is_active,
                                                egui::RichText::new(&label_text).size(11.0),
                                            )
                                            .on_hover_ui(|tip| {
                                                show_tool_tooltip(tip, item, None);
                                            });

                                        if response.clicked() {
                                            if let Some(ref t) = item.tool {
                                                *tool = *t;
                                                should_close = true;
                                            } else if let Some(ov) = item.overlay {
                                                overlay.mode = if overlay.mode == ov {
                                                    OverlayMode::None
                                                } else {
                                                    ov
                                                };
                                            } else if let Some(dk) = item.dashboard {
                                                toggle_dashboard(
                                                    dk,
                                                    &mut dashboard_vis.0,
                                                    &mut dashboard_vis.1,
                                                    &mut dashboard_vis.2,
                                                );
                                            }
                                        }
                                    }
                                }
                            });
                        });
                });

            if should_close {
                open_cat.0 = None;
            }
        }
    }
}
