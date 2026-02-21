use crate::grid::{CellType, RoadType};
use crate::test_harness::TestCity;

// ====================================================================
// TEST-057: Road Maintenance and Degradation
// ====================================================================

#[test]
fn test_road_maintenance_condition_grid_exists() {
    use crate::road_maintenance::RoadConditionGrid;
    let city = TestCity::new();
    city.assert_resource_exists::<RoadConditionGrid>();
}

#[test]
fn test_road_maintenance_budget_resource_exists() {
    use crate::road_maintenance::RoadMaintenanceBudget;
    let city = TestCity::new();
    city.assert_resource_exists::<RoadMaintenanceBudget>();
}

#[test]
fn test_road_maintenance_stats_resource_exists() {
    use crate::road_maintenance::RoadMaintenanceStats;
    let city = TestCity::new();
    city.assert_resource_exists::<RoadMaintenanceStats>();
}

#[test]
fn test_road_maintenance_initial_condition_synced() {
    use crate::road_maintenance::RoadConditionGrid;
    let mut city = TestCity::new().with_road(10, 10, 30, 10, RoadType::Local);
    city.tick(50);
    let cond = city.resource::<RoadConditionGrid>();
    let grid = city.grid();
    let mut found = false;
    for x in 10..=30 {
        if grid.get(x, 10).cell_type == CellType::Road {
            found = true;
            assert!(
                cond.get(x, 10) >= 190,
                "Road condition should be high after sync"
            );
            break;
        }
    }
    assert!(found, "Should find at least one road cell");
}

#[test]
fn test_road_condition_degrades_over_time() {
    use crate::road_maintenance::{RoadConditionGrid, RoadMaintenanceBudget};
    let mut city = TestCity::new().with_road(10, 10, 30, 10, RoadType::Local);
    {
        city.world_mut()
            .resource_mut::<RoadMaintenanceBudget>()
            .budget_level = 0.0;
    }
    city.tick(50);
    let rx = find_road_x(&city, 10, 30, 10);
    let before = city.resource::<RoadConditionGrid>().get(rx, 10);
    city.tick(50);
    let after = city.resource::<RoadConditionGrid>().get(rx, 10);
    assert!(after < before, "Road should degrade: {before} -> {after}");
}

#[test]
fn test_road_degradation_base_rate() {
    use crate::road_maintenance::{RoadConditionGrid, RoadMaintenanceBudget};
    let mut city = TestCity::new().with_road(10, 10, 20, 10, RoadType::Local);
    {
        city.world_mut()
            .resource_mut::<RoadMaintenanceBudget>()
            .budget_level = 0.0;
    }
    city.tick(50);
    let rx = find_road_x(&city, 10, 20, 10);
    let before = city.resource::<RoadConditionGrid>().get(rx, 10);
    city.tick(50);
    let after = city.resource::<RoadConditionGrid>().get(rx, 10);
    assert_eq!(
        before.saturating_sub(after),
        1,
        "Base degradation should be 1 per cycle"
    );
}

#[test]
fn test_road_no_repair_with_zero_budget() {
    use crate::road_maintenance::{RoadConditionGrid, RoadMaintenanceBudget};
    let mut city = TestCity::new().with_road(10, 10, 30, 10, RoadType::Local);
    city.tick(50);
    {
        let w = city.world_mut();
        w.resource_mut::<RoadMaintenanceBudget>().budget_level = 0.0;
        w.resource_mut::<RoadConditionGrid>().set(15, 10, 150);
    }
    let before = city.resource::<RoadConditionGrid>().get(15, 10);
    city.tick(50);
    let after = city.resource::<RoadConditionGrid>().get(15, 10);
    assert!(
        after <= before,
        "Zero budget: condition should not increase"
    );
}

#[test]
fn test_road_higher_budget_repairs_faster() {
    use crate::road_maintenance::{RoadConditionGrid, RoadMaintenanceBudget};
    let mut cn = TestCity::new().with_road(10, 10, 30, 10, RoadType::Local);
    cn.tick(50);
    {
        let w = cn.world_mut();
        w.resource_mut::<RoadConditionGrid>().set(15, 10, 100);
        w.resource_mut::<RoadMaintenanceBudget>().budget_level = 1.0;
    }
    cn.tick(50);
    let cond_n = cn.resource::<RoadConditionGrid>().get(15, 10);
    let mut cd = TestCity::new().with_road(10, 10, 30, 10, RoadType::Local);
    cd.tick(50);
    {
        let w = cd.world_mut();
        w.resource_mut::<RoadConditionGrid>().set(15, 10, 100);
        w.resource_mut::<RoadMaintenanceBudget>().budget_level = 2.0;
    }
    cd.tick(50);
    let cond_d = cd.resource::<RoadConditionGrid>().get(15, 10);
    assert!(
        cond_d >= cond_n,
        "Double budget ({cond_d}) >= normal ({cond_n})"
    );
}

#[test]
fn test_road_repair_cap_at_200() {
    use crate::road_maintenance::{RoadConditionGrid, RoadMaintenanceBudget};
    let mut city = TestCity::new().with_road(10, 10, 20, 10, RoadType::Local);
    city.tick(50);
    {
        let w = city.world_mut();
        w.resource_mut::<RoadConditionGrid>().set(15, 10, 198);
        w.resource_mut::<RoadMaintenanceBudget>().budget_level = 2.0;
    }
    if city.grid().get(15, 10).cell_type == CellType::Road {
        city.tick(50);
        assert!(city.resource::<RoadConditionGrid>().get(15, 10) > 0);
    }
}

#[test]
fn test_road_maintenance_cost_proportional_to_road_count() {
    use crate::road_maintenance::RoadMaintenanceBudget;
    let mut cs = TestCity::new().with_road(10, 10, 15, 10, RoadType::Local);
    cs.tick(50);
    let cost_s = cs.resource::<RoadMaintenanceBudget>().monthly_cost;
    let mut cl = TestCity::new()
        .with_road(10, 10, 40, 10, RoadType::Local)
        .with_road(10, 10, 10, 40, RoadType::Local);
    cl.tick(50);
    let cost_l = cl.resource::<RoadMaintenanceBudget>().monthly_cost;
    assert!(
        cost_l > cost_s,
        "More roads = higher cost: {cost_s} vs {cost_l}"
    );
}

#[test]
fn test_road_maintenance_cost_scales_with_road_type() {
    use crate::road_maintenance::RoadMaintenanceBudget;
    let mut cl = TestCity::new().with_road(10, 10, 30, 10, RoadType::Local);
    cl.tick(50);
    let cost_l = cl.resource::<RoadMaintenanceBudget>().monthly_cost;
    let mut ch = TestCity::new().with_road(10, 10, 30, 10, RoadType::Highway);
    ch.tick(50);
    let cost_h = ch.resource::<RoadMaintenanceBudget>().monthly_cost;
    assert!(cost_h > cost_l, "Highway > local: {cost_h} vs {cost_l}");
}

#[test]
fn test_road_maintenance_cost_scales_with_budget_level() {
    use crate::road_maintenance::RoadMaintenanceBudget;
    let mut city = TestCity::new().with_road(10, 10, 30, 10, RoadType::Local);
    {
        city.world_mut()
            .resource_mut::<RoadMaintenanceBudget>()
            .budget_level = 1.0;
    }
    city.tick(50);
    let c1 = city.resource::<RoadMaintenanceBudget>().monthly_cost;
    {
        city.world_mut()
            .resource_mut::<RoadMaintenanceBudget>()
            .budget_level = 2.0;
    }
    city.tick(50);
    let c2 = city.resource::<RoadMaintenanceBudget>().monthly_cost;
    assert!(
        (c2 - c1 * 2.0).abs() < 0.01,
        "Double budget = double cost: {c1} vs {c2}"
    );
}

#[test]
fn test_road_poor_condition_reduces_speed() {
    use crate::road_maintenance::RoadConditionGrid;
    let mut city = TestCity::new().with_road(10, 10, 20, 10, RoadType::Local);
    city.tick(50);
    {
        city.world_mut()
            .resource_mut::<RoadConditionGrid>()
            .set(15, 10, 80);
    }
    assert_eq!(
        city.resource::<RoadConditionGrid>()
            .road_condition_speed_factor(15, 10),
        0.7
    );
}

#[test]
fn test_road_critical_condition_blocks_travel() {
    use crate::road_maintenance::RoadConditionGrid;
    let mut city = TestCity::new().with_road(10, 10, 20, 10, RoadType::Local);
    city.tick(50);
    {
        city.world_mut()
            .resource_mut::<RoadConditionGrid>()
            .set(15, 10, 10);
    }
    assert_eq!(
        city.resource::<RoadConditionGrid>()
            .road_condition_speed_factor(15, 10),
        0.0
    );
}

#[test]
fn test_road_good_condition_no_speed_penalty() {
    use crate::road_maintenance::RoadConditionGrid;
    let mut city = TestCity::new().with_road(10, 10, 20, 10, RoadType::Local);
    city.tick(50);
    {
        city.world_mut()
            .resource_mut::<RoadConditionGrid>()
            .set(15, 10, 150);
    }
    assert_eq!(
        city.resource::<RoadConditionGrid>()
            .road_condition_speed_factor(15, 10),
        1.0
    );
}

#[test]
fn test_road_maintenance_stats_count_poor_and_critical() {
    use crate::road_maintenance::{RoadConditionGrid, RoadMaintenanceStats};
    let mut city = TestCity::new().with_road(10, 10, 30, 10, RoadType::Local);
    city.tick(50);
    let grid = city.grid();
    let mut rc: Vec<(usize, usize)> = Vec::new();
    for x in 10..=30 {
        if grid.get(x, 10).cell_type == CellType::Road {
            rc.push((x, 10));
        }
    }
    assert!(rc.len() >= 3);
    {
        let w = city.world_mut();
        let mut c = w.resource_mut::<RoadConditionGrid>();
        c.set(rc[0].0, rc[0].1, 50);
        c.set(rc[1].0, rc[1].1, 10);
    }
    city.tick(50);
    let s = city.resource::<RoadMaintenanceStats>();
    assert!(s.poor_roads_count >= 1);
    assert!(s.critical_roads_count >= 1);
}

#[test]
fn test_road_maintenance_stats_avg_condition_healthy() {
    use crate::road_maintenance::RoadMaintenanceStats;
    let mut city = TestCity::new().with_road(10, 10, 30, 10, RoadType::Local);
    city.tick(50);
    assert!(city.resource::<RoadMaintenanceStats>().avg_condition > 150.0);
}

#[test]
fn test_road_sustained_degradation_without_repair() {
    use crate::road_maintenance::{RoadConditionGrid, RoadMaintenanceBudget};
    let mut city = TestCity::new().with_road(10, 10, 20, 10, RoadType::Local);
    {
        city.world_mut()
            .resource_mut::<RoadMaintenanceBudget>()
            .budget_level = 0.0;
    }
    city.tick(50);
    let rx = find_road_x(&city, 10, 20, 10);
    let start = city.resource::<RoadConditionGrid>().get(rx, 10);
    for _ in 0..110 {
        city.tick(50);
    }
    let end = city.resource::<RoadConditionGrid>().get(rx, 10);
    assert!(end < 100, "After 110 cycles: start={start}, end={end}");
}

#[test]
fn test_road_maintenance_cost_zero_without_roads() {
    use crate::road_maintenance::RoadMaintenanceBudget;
    let mut city = TestCity::new();
    city.tick(50);
    assert_eq!(city.resource::<RoadMaintenanceBudget>().monthly_cost, 0.0);
}

#[test]
fn test_road_degradation_only_runs_every_50_ticks() {
    use crate::road_maintenance::RoadConditionGrid;
    let mut city = TestCity::new().with_road(10, 10, 20, 10, RoadType::Local);
    city.tick(50);
    let before = city.resource::<RoadConditionGrid>().get(15, 10);
    city.tick(49);
    assert_eq!(before, city.resource::<RoadConditionGrid>().get(15, 10));
}

#[test]
fn test_road_non_road_cells_stay_at_zero_condition() {
    use crate::road_maintenance::RoadConditionGrid;
    let mut city = TestCity::new().with_road(10, 10, 20, 10, RoadType::Local);
    city.tick(50);
    let c = city.resource::<RoadConditionGrid>();
    assert_eq!(c.get(0, 0), 0);
    assert_eq!(c.get(5, 5), 0);
}

#[test]
fn test_road_speed_factor_boundary_at_100() {
    let c = road_condition_grid_with(15, 10, 100);
    assert_eq!(c.road_condition_speed_factor(15, 10), 1.0);
}

#[test]
fn test_road_speed_factor_boundary_at_25() {
    let c = road_condition_grid_with(15, 10, 25);
    assert_eq!(c.road_condition_speed_factor(15, 10), 0.7);
}

#[test]
fn test_road_speed_factor_boundary_at_99() {
    let c = road_condition_grid_with(15, 10, 99);
    assert_eq!(c.road_condition_speed_factor(15, 10), 0.7);
}

#[test]
fn test_road_speed_factor_boundary_at_24() {
    let c = road_condition_grid_with(15, 10, 24);
    assert_eq!(c.road_condition_speed_factor(15, 10), 0.0);
}

fn find_road_x(city: &TestCity, x_start: usize, x_end: usize, y: usize) -> usize {
    let grid = city.grid();
    for x in x_start..=x_end {
        if grid.get(x, y).cell_type == CellType::Road {
            return x;
        }
    }
    panic!("No road cell found in x=[{x_start},{x_end}] y={y}");
}

fn road_condition_grid_with(
    x: usize,
    y: usize,
    condition: u8,
) -> crate::road_maintenance::RoadConditionGrid {
    let mut c = crate::road_maintenance::RoadConditionGrid::default();
    c.set(x, y, condition);
    c
}
