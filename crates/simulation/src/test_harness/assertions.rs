//! Assertion helpers for `TestCity` integration tests.

use bevy::prelude::*;

use crate::grid::{CellType, ZoneType};
use crate::roads::RoadNode;

use super::TestCity;

impl TestCity {
    // -----------------------------------------------------------------------
    // Assertions
    // -----------------------------------------------------------------------

    /// Assert citizen count is between min and max (inclusive).
    pub fn assert_citizen_count_between(&mut self, min: usize, max: usize) {
        let count = self.citizen_count();
        assert!(
            count >= min && count <= max,
            "Expected citizen count in [{min}, {max}], got {count}"
        );
    }

    /// Assert treasury is above a given amount.
    pub fn assert_budget_above(&self, amount: f64) {
        let treasury = self.budget().treasury;
        assert!(
            treasury > amount,
            "Expected treasury > {amount}, got {treasury}"
        );
    }

    /// Assert treasury is below a given amount.
    pub fn assert_budget_below(&self, amount: f64) {
        let treasury = self.budget().treasury;
        assert!(
            treasury < amount,
            "Expected treasury < {amount}, got {treasury}"
        );
    }

    /// Assert that a cell contains a road.
    pub fn assert_has_road(&self, x: usize, y: usize) {
        let cell = self.cell(x, y);
        assert_eq!(
            cell.cell_type,
            CellType::Road,
            "Expected road at ({x}, {y}), found {:?}",
            cell.cell_type
        );
    }

    /// Assert that a cell has a building.
    pub fn assert_has_building(&self, x: usize, y: usize) {
        let cell = self.cell(x, y);
        assert!(
            cell.building_id.is_some(),
            "Expected building at ({x}, {y}), found none"
        );
    }

    /// Assert that a cell has a specific zone type.
    pub fn assert_zone(&self, x: usize, y: usize, expected: ZoneType) {
        let cell = self.cell(x, y);
        assert_eq!(
            cell.zone, expected,
            "Expected zone {:?} at ({x}, {y}), found {:?}",
            expected, cell.zone
        );
    }

    /// Assert the road network contains a node at (x, y).
    pub fn assert_road_node_exists(&self, x: usize, y: usize) {
        let network = self.road_network();
        let node = RoadNode(x, y);
        assert!(
            network.edges.contains_key(&node),
            "Expected road node at ({x}, {y}) in RoadNetwork"
        );
    }

    /// Assert the slow tick timer has reached at least the given count.
    pub fn assert_ticks_at_least(&self, min: u32) {
        let counter = self.slow_tick_timer().counter;
        assert!(
            counter >= min,
            "Expected at least {min} ticks, got {counter}"
        );
    }

    /// Assert the game clock hour is approximately the expected value.
    pub fn assert_hour_approx(&self, expected: f32, tolerance: f32) {
        let hour = self.clock().hour;
        assert!(
            (hour - expected).abs() < tolerance,
            "Expected hour ~{expected} (±{tolerance}), got {hour}"
        );
    }

    /// Assert a resource has been initialized (exists in the world).
    pub fn assert_resource_exists<T: Resource>(&self) {
        assert!(
            self.app.world().get_resource::<T>().is_some(),
            "Expected resource {} to exist",
            std::any::type_name::<T>()
        );
    }
}
