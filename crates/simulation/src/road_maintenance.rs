use bevy::prelude::*;

use crate::config::{GRID_HEIGHT, GRID_WIDTH};
use crate::grid::{CellType, WorldGrid};
use crate::traffic::TrafficGrid;

/// Per-cell road condition grid. Each byte represents condition 0-255.
/// 255 = perfect condition, 0 = destroyed.
/// Non-road cells stay at 0.
#[derive(Resource)]
pub struct RoadConditionGrid {
    pub conditions: Vec<u8>,
    pub width: usize,
    pub height: usize,
}

impl Default for RoadConditionGrid {
    fn default() -> Self {
        Self {
            conditions: vec![0; GRID_WIDTH * GRID_HEIGHT],
            width: GRID_WIDTH,
            height: GRID_HEIGHT,
        }
    }
}

impl RoadConditionGrid {
    #[inline]
    pub fn get(&self, x: usize, y: usize) -> u8 {
        self.conditions[y * self.width + x]
    }

    #[inline]
    pub fn set(&mut self, x: usize, y: usize, val: u8) {
        self.conditions[y * self.width + x] = val;
    }

    /// Initialize condition for all road cells to a given value.
    pub fn sync_with_grid(&mut self, grid: &WorldGrid) {
        for y in 0..self.height {
            for x in 0..self.width {
                if grid.get(x, y).cell_type == CellType::Road {
                    if self.get(x, y) == 0 {
                        self.set(x, y, 200); // Good condition for new roads
                    }
                } else {
                    self.set(x, y, 0);
                }
            }
        }
    }

    /// Returns a speed factor based on road condition at the given cell.
    /// - condition >= 100: factor = 1.0 (no penalty)
    /// - condition 25..100: factor = 0.7 (30% speed reduction)
    /// - condition < 25: factor = 0.0 (road effectively blocked)
    pub fn road_condition_speed_factor(&self, x: usize, y: usize) -> f32 {
        let condition = self.get(x, y);
        if condition >= 100 {
            1.0
        } else if condition >= 25 {
            0.7
        } else {
            0.0
        }
    }
}

/// Budget allocation for road maintenance.
#[derive(Resource)]
pub struct RoadMaintenanceBudget {
    /// Slider value: 0.0 = no maintenance, 1.0 = normal, 2.0 = double budget.
    pub budget_level: f32,
    /// Computed monthly cost: sum of per-cell maintenance costs * budget_level.
    pub monthly_cost: f64,
}

impl Default for RoadMaintenanceBudget {
    fn default() -> Self {
        Self {
            budget_level: 1.0,
            monthly_cost: 0.0,
        }
    }
}

/// Aggregated statistics about road conditions across the city.
#[derive(Resource, Default)]
pub struct RoadMaintenanceStats {
    /// Average condition across all road cells (0.0 - 255.0).
    pub avg_condition: f32,
    /// Number of road cells with condition < 100.
    pub poor_roads_count: u32,
    /// Number of road cells with condition < 25.
    pub critical_roads_count: u32,
}

/// Degrades road conditions based on traffic levels. Runs every 50 ticks.
pub fn degrade_roads(
    tick: Res<crate::TickCounter>,
    grid: Res<WorldGrid>,
    traffic: Res<TrafficGrid>,
    mut condition_grid: ResMut<RoadConditionGrid>,
) {
    if !tick.0.is_multiple_of(50) {
        return;
    }

    // On first run, sync condition grid with current road cells
    let needs_sync = grid
        .cells
        .iter()
        .enumerate()
        .any(|(i, cell)| cell.cell_type == CellType::Road && condition_grid.conditions[i] == 0);
    if needs_sync {
        condition_grid.sync_with_grid(&grid);
    }

    for y in 0..GRID_HEIGHT {
        for x in 0..GRID_WIDTH {
            if grid.get(x, y).cell_type != CellType::Road {
                continue;
            }

            let traffic_level = traffic.get(x, y);
            let degradation = (traffic_level * 2 + 1).min(255) as u8;
            let current = condition_grid.get(x, y);
            condition_grid.set(x, y, current.saturating_sub(degradation));
        }
    }
}

/// Repairs roads based on maintenance budget. Runs every 50 ticks, after degrade_roads.
pub fn repair_roads(
    tick: Res<crate::TickCounter>,
    grid: Res<WorldGrid>,
    maint_budget: Res<RoadMaintenanceBudget>,
    mut condition_grid: ResMut<RoadConditionGrid>,
) {
    if !tick.0.is_multiple_of(50) {
        return;
    }

    if maint_budget.budget_level <= 0.0 {
        return;
    }

    let repair_amount = (maint_budget.budget_level * 5.0) as u8;

    for y in 0..GRID_HEIGHT {
        for x in 0..GRID_WIDTH {
            if grid.get(x, y).cell_type != CellType::Road {
                continue;
            }

            let current = condition_grid.get(x, y);
            if current < 200 {
                condition_grid.set(x, y, current.saturating_add(repair_amount));
            }
        }
    }
}

/// Updates maintenance cost and aggregated statistics. Runs every 50 ticks.
pub fn update_road_maintenance_stats(
    tick: Res<crate::TickCounter>,
    grid: Res<WorldGrid>,
    condition_grid: Res<RoadConditionGrid>,
    mut maint_budget: ResMut<RoadMaintenanceBudget>,
    mut stats: ResMut<RoadMaintenanceStats>,
) {
    if !tick.0.is_multiple_of(50) {
        return;
    }

    let mut total_condition: u64 = 0;
    let mut road_cell_count: u32 = 0;
    let mut poor_count: u32 = 0;
    let mut critical_count: u32 = 0;

    for y in 0..GRID_HEIGHT {
        for x in 0..GRID_WIDTH {
            if grid.get(x, y).cell_type != CellType::Road {
                continue;
            }
            road_cell_count += 1;
            let cond = condition_grid.get(x, y);
            total_condition += cond as u64;
            if cond < 100 {
                poor_count += 1;
            }
            if cond < 25 {
                critical_count += 1;
            }
        }
    }

    stats.avg_condition = if road_cell_count > 0 {
        total_condition as f32 / road_cell_count as f32
    } else {
        0.0
    };
    stats.poor_roads_count = poor_count;
    stats.critical_roads_count = critical_count;

    // Update monthly cost (sum per-cell maintenance costs scaled by budget level)
    let mut total_maintenance_cost: f64 = 0.0;
    for y in 0..GRID_HEIGHT {
        for x in 0..GRID_WIDTH {
            let cell = grid.get(x, y);
            if cell.cell_type == CellType::Road {
                total_maintenance_cost += cell.road_type.maintenance_cost();
            }
        }
    }
    maint_budget.monthly_cost = total_maintenance_cost * maint_budget.budget_level as f64;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{GRID_HEIGHT, GRID_WIDTH};
    use crate::grid::{CellType, WorldGrid};

    fn make_grid_with_road(x: usize, y: usize) -> WorldGrid {
        let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        grid.get_mut(x, y).cell_type = CellType::Road;
        grid
    }

    #[test]
    fn test_default_grid_all_zeros() {
        let cond = RoadConditionGrid::default();
        for &val in &cond.conditions {
            assert_eq!(val, 0, "Non-road cells should default to 0");
        }
    }

    #[test]
    fn test_sync_sets_road_condition() {
        let grid = make_grid_with_road(10, 10);
        let mut cond = RoadConditionGrid::default();
        cond.sync_with_grid(&grid);
        assert_eq!(
            cond.get(10, 10),
            200,
            "Road cell should be initialized to 200"
        );
        assert_eq!(cond.get(0, 0), 0, "Non-road cell should remain 0");
    }

    #[test]
    fn test_degradation_reduces_condition() {
        let mut cond = RoadConditionGrid::default();
        cond.set(10, 10, 200);

        // Simulate degradation: traffic_level=5, so degradation = 5*2+1 = 11
        let traffic_level: u16 = 5;
        let degradation = (traffic_level * 2 + 1).min(255) as u8;
        let current = cond.get(10, 10);
        cond.set(10, 10, current.saturating_sub(degradation));

        assert_eq!(cond.get(10, 10), 189, "Condition should decrease by 11");
    }

    #[test]
    fn test_degradation_clamps_to_zero() {
        let mut cond = RoadConditionGrid::default();
        cond.set(10, 10, 5);

        // Heavy traffic degradation larger than current condition
        let traffic_level: u16 = 20;
        let degradation = (traffic_level * 2 + 1).min(255) as u8;
        let current = cond.get(10, 10);
        cond.set(10, 10, current.saturating_sub(degradation));

        assert_eq!(cond.get(10, 10), 0, "Condition should clamp to 0");
    }

    #[test]
    fn test_repair_increases_condition() {
        let mut cond = RoadConditionGrid::default();
        cond.set(10, 10, 150);

        let budget_level: f32 = 1.0;
        let repair_amount = (budget_level * 5.0) as u8;
        let current = cond.get(10, 10);
        cond.set(10, 10, current.saturating_add(repair_amount));

        assert_eq!(cond.get(10, 10), 155, "Condition should increase by 5");
    }

    #[test]
    fn test_repair_clamps_to_255() {
        let mut cond = RoadConditionGrid::default();
        cond.set(10, 10, 253);

        let budget_level: f32 = 2.0;
        let repair_amount = (budget_level * 5.0) as u8;
        let current = cond.get(10, 10);
        cond.set(10, 10, current.saturating_add(repair_amount));

        assert_eq!(cond.get(10, 10), 255, "Condition should clamp to 255");
    }

    #[test]
    fn test_speed_factor_perfect_condition() {
        let mut cond = RoadConditionGrid::default();
        cond.set(10, 10, 200);
        assert_eq!(cond.road_condition_speed_factor(10, 10), 1.0);
    }

    #[test]
    fn test_speed_factor_at_boundary_100() {
        let mut cond = RoadConditionGrid::default();
        cond.set(10, 10, 100);
        assert_eq!(cond.road_condition_speed_factor(10, 10), 1.0);
    }

    #[test]
    fn test_speed_factor_poor_condition() {
        let mut cond = RoadConditionGrid::default();
        cond.set(10, 10, 80);
        assert_eq!(cond.road_condition_speed_factor(10, 10), 0.7);
    }

    #[test]
    fn test_speed_factor_at_boundary_25() {
        let mut cond = RoadConditionGrid::default();
        cond.set(10, 10, 25);
        assert_eq!(cond.road_condition_speed_factor(10, 10), 0.7);
    }

    #[test]
    fn test_speed_factor_critical_condition() {
        let mut cond = RoadConditionGrid::default();
        cond.set(10, 10, 24);
        assert_eq!(cond.road_condition_speed_factor(10, 10), 0.0);
    }

    #[test]
    fn test_speed_factor_destroyed() {
        let mut cond = RoadConditionGrid::default();
        cond.set(10, 10, 0);
        assert_eq!(cond.road_condition_speed_factor(10, 10), 0.0);
    }

    #[test]
    fn test_maintenance_stats_empty_grid() {
        let stats = RoadMaintenanceStats::default();
        assert_eq!(stats.avg_condition, 0.0);
        assert_eq!(stats.poor_roads_count, 0);
        assert_eq!(stats.critical_roads_count, 0);
    }

    #[test]
    fn test_maintenance_budget_default() {
        let budget = RoadMaintenanceBudget::default();
        assert_eq!(budget.budget_level, 1.0);
        assert_eq!(budget.monthly_cost, 0.0);
    }

    #[test]
    fn test_double_budget_repairs_faster() {
        let mut cond = RoadConditionGrid::default();
        cond.set(10, 10, 100);

        let budget_level_normal: f32 = 1.0;
        let budget_level_double: f32 = 2.0;

        let repair_normal = (budget_level_normal * 5.0) as u8;
        let repair_double = (budget_level_double * 5.0) as u8;

        assert!(
            repair_double > repair_normal,
            "Double budget should repair faster"
        );
    }
}

pub struct RoadMaintenancePlugin;

impl Plugin for RoadMaintenancePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<RoadConditionGrid>()
            .init_resource::<RoadMaintenanceBudget>()
            .init_resource::<RoadMaintenanceStats>()
            .add_systems(
                FixedUpdate,
                (degrade_roads, repair_roads, update_road_maintenance_stats)
                    .chain()
                    .after(crate::traffic::update_traffic_density)
                    .in_set(crate::SimulationSet::Simulation),
            );
    }
}
