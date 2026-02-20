//! Multi-Select UI Panel (UX-059).
//!
//! Handles Ctrl+Click input for adding entities to the multi-selection,
//! displays the selection count in a status bar, and provides batch
//! operation buttons (bulldoze all, upgrade all roads) with total cost.

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use rendering::input::{ActiveTool, CursorGridPos, StatusMessage};
use simulation::economy::CityBudget;
use simulation::grid::{CellType, RoadType, WorldGrid, ZoneType};
use simulation::multi_select::{
    BatchBulldozeEvent, BatchRoadUpgradeEvent, MultiSelectState, SelectableItem,
};
use simulation::roads::RoadNetwork;
use simulation::services::ServiceBuilding;

// =============================================================================
// Systems
// =============================================================================

/// Handle Ctrl+Click to toggle entities in/out of the multi-selection.
///
/// When Ctrl is held and the player left-clicks:
/// - On a building: toggle that building entity in the selection.
/// - On a road cell: toggle that road cell in the selection.
/// - On empty terrain: no-op.
///
/// When clicking without Ctrl, the multi-selection is cleared (normal click
/// behaviour takes over via the existing input system).
#[allow(clippy::too_many_arguments)]
pub fn handle_multi_select_input(
    buttons: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    cursor: Res<CursorGridPos>,
    tool: Res<ActiveTool>,
    grid: Res<WorldGrid>,
    mut multi_select: ResMut<MultiSelectState>,
    mut status: ResMut<StatusMessage>,
    left_drag: Res<rendering::camera::LeftClickDrag>,
) {
    // Only process on fresh left-click
    if !buttons.just_pressed(MouseButton::Left) || !cursor.valid {
        return;
    }

    // Don't interfere during camera drag
    if left_drag.is_dragging {
        return;
    }

    let ctrl_held = keys.pressed(KeyCode::ControlLeft) || keys.pressed(KeyCode::ControlRight);

    // If Ctrl is not held, clear multi-selection on any click
    // (the regular tool input system handles the normal action)
    if !ctrl_held {
        if !multi_select.is_empty() {
            multi_select.clear();
        }
        return;
    }

    // Only allow multi-select with Inspect or Bulldoze tool
    if !matches!(*tool, ActiveTool::Inspect | ActiveTool::Bulldoze) {
        return;
    }

    let gx = cursor.grid_x as usize;
    let gy = cursor.grid_y as usize;

    if !grid.in_bounds(gx, gy) {
        return;
    }

    let cell = grid.get(gx, gy);

    if let Some(entity) = cell.building_id {
        let item = SelectableItem::Building(entity);
        multi_select.toggle(item);
        let count = multi_select.count();
        status.set(format!("{} item(s) selected", count), false);
    } else if cell.cell_type == CellType::Road {
        let item = SelectableItem::RoadCell { x: gx, y: gy };
        multi_select.toggle(item);
        let count = multi_select.count();
        status.set(format!("{} item(s) selected", count), false);
    }
}

/// Escape key clears multi-selection.
pub fn multi_select_escape(
    keys: Res<ButtonInput<KeyCode>>,
    mut multi_select: ResMut<MultiSelectState>,
    mut contexts: EguiContexts,
) {
    if contexts.ctx_mut().wants_keyboard_input() {
        return;
    }
    if keys.just_pressed(KeyCode::Escape) && !multi_select.is_empty() {
        multi_select.clear();
    }
}

/// Calculate the total cost of upgrading all selected road cells.
fn upgrade_cost(multi_select: &MultiSelectState, grid: &WorldGrid) -> f64 {
    let mut total = 0.0;
    for (x, y) in multi_select.road_cells() {
        if grid.in_bounds(x, y) {
            let cell = grid.get(x, y);
            if cell.cell_type == CellType::Road {
                if let Some(cost) = cell.road_type.upgrade_cost() {
                    total += cost;
                }
            }
        }
    }
    total
}

/// Count how many selected roads are actually upgradable.
fn upgradable_road_count(multi_select: &MultiSelectState, grid: &WorldGrid) -> usize {
    multi_select
        .road_cells()
        .iter()
        .filter(|&&(x, y)| {
            grid.in_bounds(x, y) && {
                let cell = grid.get(x, y);
                cell.cell_type == CellType::Road && cell.road_type.upgrade_tier().is_some()
            }
        })
        .count()
}

/// Display the multi-select status bar and batch operation panel.
///
/// Shows:
/// - Selection count
/// - Batch Bulldoze button
/// - Batch Upgrade button (for road cells) with total cost
/// - Clear Selection button
#[allow(clippy::too_many_arguments)]
pub fn multi_select_panel_ui(
    mut contexts: EguiContexts,
    mut multi_select: ResMut<MultiSelectState>,
    grid: Res<WorldGrid>,
    budget: Res<CityBudget>,
    mut bulldoze_events: EventWriter<BatchBulldozeEvent>,
    mut upgrade_events: EventWriter<BatchRoadUpgradeEvent>,
) {
    if multi_select.is_empty() {
        return;
    }

    let count = multi_select.count();
    let building_count = multi_select.building_count();
    let road_count = multi_select.road_count();
    let total_upgrade_cost = upgrade_cost(&multi_select, &grid);
    let upgradable_count = upgradable_road_count(&multi_select, &grid);

    let mut should_clear = false;

    egui::Window::new("Multi-Select")
        .default_width(240.0)
        .anchor(egui::Align2::RIGHT_BOTTOM, egui::vec2(-8.0, -8.0))
        .collapsible(false)
        .resizable(false)
        .show(contexts.ctx_mut(), |ui| {
            // Selection summary
            ui.heading(format!("{} Selected", count));
            ui.separator();

            if building_count > 0 {
                ui.label(format!("Buildings: {}", building_count));
            }
            if road_count > 0 {
                ui.label(format!("Road cells: {}", road_count));
            }

            ui.separator();

            // Batch Bulldoze (bulldozing is free)
            let bulldoze_label = format!("Bulldoze All ({} items)", count);
            if ui.button(bulldoze_label).clicked() {
                bulldoze_events.send(BatchBulldozeEvent);
            }

            // Batch Road Upgrade (only shown when road cells are selected)
            if road_count > 0 {
                let can_afford = budget.treasury >= total_upgrade_cost;
                let has_upgradable = upgradable_count > 0;

                let upgrade_label = format!(
                    "Upgrade Roads ({} upgradable, ${:.0})",
                    upgradable_count, total_upgrade_cost
                );

                ui.add_enabled_ui(can_afford && has_upgradable, |ui| {
                    if ui.button(upgrade_label).clicked() {
                        upgrade_events.send(BatchRoadUpgradeEvent);
                    }
                });

                if !can_afford && has_upgradable {
                    ui.colored_label(
                        egui::Color32::from_rgb(220, 50, 50),
                        "Not enough money for upgrade",
                    );
                }
                if !has_upgradable && road_count > 0 {
                    ui.colored_label(
                        egui::Color32::from_rgb(160, 160, 160),
                        "No roads can be upgraded further",
                    );
                }
            }

            ui.separator();
            if ui.button("Clear Selection (Esc)").clicked() {
                should_clear = true;
            }
        });

    if should_clear {
        multi_select.clear();
    }
}

/// Execute batch bulldoze when the event is received.
#[allow(clippy::too_many_arguments)]
pub fn execute_batch_bulldoze(
    mut events: EventReader<BatchBulldozeEvent>,
    mut multi_select: ResMut<MultiSelectState>,
    mut grid: ResMut<WorldGrid>,
    mut roads: ResMut<RoadNetwork>,
    mut status: ResMut<StatusMessage>,
    mut commands: Commands,
    service_q: Query<&ServiceBuilding>,
) {
    for _event in events.read() {
        let items: Vec<SelectableItem> = multi_select.selected_items.clone();
        let mut demolished = 0usize;

        for item in &items {
            match item {
                SelectableItem::Building(entity) => {
                    // Check if it's a multi-cell service building
                    if let Ok(service) = service_q.get(*entity) {
                        let (fw, fh) = ServiceBuilding::footprint(service.service_type);
                        let sx = service.grid_x;
                        let sy = service.grid_y;
                        for fy in sy..sy + fh {
                            for fx in sx..sx + fw {
                                if grid.in_bounds(fx, fy) {
                                    grid.get_mut(fx, fy).building_id = None;
                                    grid.get_mut(fx, fy).zone = ZoneType::None;
                                }
                            }
                        }
                    } else {
                        // Regular building: scan grid for matching entity
                        for y in 0..grid.height {
                            for x in 0..grid.width {
                                if grid.get(x, y).building_id == Some(*entity) {
                                    grid.get_mut(x, y).building_id = None;
                                    grid.get_mut(x, y).zone = ZoneType::None;
                                }
                            }
                        }
                    }
                    commands.entity(*entity).despawn();
                    demolished += 1;
                }
                SelectableItem::RoadCell { x, y } => {
                    if grid.in_bounds(*x, *y) && grid.get(*x, *y).cell_type == CellType::Road {
                        roads.remove_road(&mut grid, *x, *y);
                        demolished += 1;
                    }
                }
            }
        }

        multi_select.clear();
        status.set(format!("Demolished {} items", demolished), false);
    }
}

/// Execute batch road upgrade when the event is received.
pub fn execute_batch_road_upgrade(
    mut events: EventReader<BatchRoadUpgradeEvent>,
    mut multi_select: ResMut<MultiSelectState>,
    mut grid: ResMut<WorldGrid>,
    mut budget: ResMut<CityBudget>,
    mut status: ResMut<StatusMessage>,
) {
    for _event in events.read() {
        let road_cells = multi_select.road_cells();

        // Calculate total cost first
        let mut total_cost = 0.0;
        let mut upgradable: Vec<(usize, usize, RoadType)> = Vec::new();

        for (x, y) in &road_cells {
            if grid.in_bounds(*x, *y) {
                let cell = grid.get(*x, *y);
                if cell.cell_type == CellType::Road {
                    if let Some(next_tier) = cell.road_type.upgrade_tier() {
                        let cost = cell.road_type.upgrade_cost().unwrap_or(0.0);
                        total_cost += cost;
                        upgradable.push((*x, *y, next_tier));
                    }
                }
            }
        }

        if budget.treasury < total_cost {
            status.set("Not enough money for batch upgrade", true);
            continue;
        }

        if upgradable.is_empty() {
            status.set("No roads can be upgraded", true);
            continue;
        }

        // Apply all upgrades
        for (x, y, next_tier) in &upgradable {
            grid.get_mut(*x, *y).road_type = *next_tier;
        }

        budget.treasury -= total_cost;
        let count = upgradable.len();

        multi_select.clear();
        status.set(
            format!("Upgraded {} roads (${:.0})", count, total_cost),
            false,
        );
    }
}

// =============================================================================
// Plugin
// =============================================================================

pub struct MultiSelectUiPlugin;

impl Plugin for MultiSelectUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                handle_multi_select_input,
                multi_select_escape,
                multi_select_panel_ui,
                execute_batch_bulldoze,
                execute_batch_road_upgrade,
            ),
        );
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use simulation::grid::WorldGrid;

    #[test]
    fn test_upgrade_cost_calculation() {
        let mut grid = WorldGrid::new(4, 4);
        grid.get_mut(0, 0).cell_type = CellType::Road;
        grid.get_mut(0, 0).road_type = RoadType::Local;
        grid.get_mut(1, 0).cell_type = CellType::Road;
        grid.get_mut(1, 0).road_type = RoadType::Avenue;
        grid.get_mut(2, 0).cell_type = CellType::Road;
        grid.get_mut(2, 0).road_type = RoadType::Boulevard; // max tier

        let mut state = MultiSelectState::default();
        state.add(SelectableItem::RoadCell { x: 0, y: 0 });
        state.add(SelectableItem::RoadCell { x: 1, y: 0 });
        state.add(SelectableItem::RoadCell { x: 2, y: 0 });

        let cost = upgrade_cost(&state, &grid);
        // Local->Avenue = $10, Avenue->Boulevard = $10, Boulevard = $0
        assert!((cost - 20.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_upgradable_road_count() {
        let mut grid = WorldGrid::new(4, 4);
        grid.get_mut(0, 0).cell_type = CellType::Road;
        grid.get_mut(0, 0).road_type = RoadType::Local;
        grid.get_mut(1, 0).cell_type = CellType::Road;
        grid.get_mut(1, 0).road_type = RoadType::Boulevard;

        let mut state = MultiSelectState::default();
        state.add(SelectableItem::RoadCell { x: 0, y: 0 });
        state.add(SelectableItem::RoadCell { x: 1, y: 0 });

        assert_eq!(upgradable_road_count(&state, &grid), 1);
    }

    #[test]
    fn test_upgrade_cost_empty_selection() {
        let grid = WorldGrid::new(4, 4);
        let state = MultiSelectState::default();
        assert!((upgrade_cost(&state, &grid)).abs() < f64::EPSILON);
    }

    #[test]
    fn test_upgradable_count_empty_selection() {
        let grid = WorldGrid::new(4, 4);
        let state = MultiSelectState::default();
        assert_eq!(upgradable_road_count(&state, &grid), 0);
    }
}
