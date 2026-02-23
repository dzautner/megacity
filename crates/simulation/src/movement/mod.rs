mod pathfinding;
mod state_machine;

pub use pathfinding::{
    collect_path_results, move_citizens, process_path_requests, update_pathfinding_snapshot,
    ComputingPath, PathfindingSnapshot,
};
pub use state_machine::{
    citizen_state_machine, find_nearest, invalidate_paths_on_road_removal,
    refresh_destination_cache, ActivityTimer, DestinationCache,
};

use bevy::prelude::*;

pub struct MovementPlugin;

impl Plugin for MovementPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DestinationCache>()
            .init_resource::<PathfindingSnapshot>()
            .add_systems(
                FixedUpdate,
                (
                    invalidate_paths_on_road_removal,
                    refresh_destination_cache,
                    citizen_state_machine,
                    bevy::ecs::schedule::apply_deferred,
                    update_pathfinding_snapshot,
                    process_path_requests,
                    bevy::ecs::schedule::apply_deferred,
                    collect_path_results,
                    bevy::ecs::schedule::apply_deferred,
                    move_citizens,
                )
                    .chain()
                    .after(crate::citizen_spawner::spawn_citizens)
                    .in_set(crate::SimulationSet::Simulation),
            );
    }
}

#[cfg(test)]
mod tests {
    use super::state_machine::find_nearest;

    // ------------------------------------------------------------------
    // find_nearest: basic nearest lookup
    // ------------------------------------------------------------------

    #[test]
    fn test_find_nearest_returns_closest_destination() {
        let spots = vec![(10, 10), (20, 20), (5, 5), (50, 50)];
        let result = find_nearest(&spots, 6, 6, 30);
        assert_eq!(
            result,
            Some((5, 5)),
            "should return (5,5) as closest to (6,6)"
        );
    }

    #[test]
    fn test_find_nearest_empty_returns_none() {
        let spots: Vec<(usize, usize)> = vec![];
        let result = find_nearest(&spots, 10, 10, 100);
        assert_eq!(result, None, "empty destination list should return None");
    }

    #[test]
    fn test_find_nearest_all_beyond_max_dist_returns_none() {
        let spots = vec![(100, 100), (200, 200)];
        let result = find_nearest(&spots, 0, 0, 10);
        assert_eq!(result, None, "all spots beyond max_dist should return None");
    }

    // ------------------------------------------------------------------
    // find_nearest: multiple destinations correct closest
    // ------------------------------------------------------------------

    #[test]
    fn test_find_nearest_multiple_destinations_various_query_points() {
        let spots = vec![(10, 10), (50, 50), (90, 90), (200, 200)];

        // From (12, 12): closest is (10, 10) with Manhattan dist 4
        assert_eq!(find_nearest(&spots, 12, 12, 100), Some((10, 10)));

        // From (48, 52): closest is (50, 50) with Manhattan dist 4
        assert_eq!(find_nearest(&spots, 48, 52, 100), Some((50, 50)));

        // From (88, 91): closest is (90, 90) with Manhattan dist 3
        assert_eq!(find_nearest(&spots, 88, 91, 100), Some((90, 90)));

        // From (199, 201): closest is (200, 200) with Manhattan dist 2
        assert_eq!(find_nearest(&spots, 199, 201, 250), Some((200, 200)));
    }

    // ------------------------------------------------------------------
    // find_nearest: exact position match
    // ------------------------------------------------------------------

    #[test]
    fn test_find_nearest_exact_match() {
        let spots = vec![(10, 10), (20, 20)];
        let result = find_nearest(&spots, 10, 10, 30);
        assert_eq!(
            result,
            Some((10, 10)),
            "querying from an exact destination position should return it"
        );
    }

    // ------------------------------------------------------------------
    // find_nearest: max_dist boundary
    // ------------------------------------------------------------------

    #[test]
    fn test_find_nearest_at_exact_max_dist_boundary() {
        let spots = vec![(15, 15)];
        // Manhattan distance from (10, 10) to (15, 15) = 10
        let result = find_nearest(&spots, 10, 10, 10);
        assert_eq!(
            result,
            Some((15, 15)),
            "spot at exactly max_dist should be included"
        );

        // max_dist = 9 -> should exclude
        let result2 = find_nearest(&spots, 10, 10, 9);
        assert_eq!(
            result2, None,
            "spot at dist 10 with max_dist 9 should be excluded"
        );
    }

    // ------------------------------------------------------------------
    // find_nearest: tiebreaker (min_by_key picks first minimum)
    // ------------------------------------------------------------------

    #[test]
    fn test_find_nearest_equidistant_returns_first() {
        // Two spots equidistant from query point
        let spots = vec![(10, 12), (12, 10)];
        // Manhattan dist from (11, 11): |10-11|+|12-11|=2 and |12-11|+|10-11|=2
        let result = find_nearest(&spots, 11, 11, 30);
        // min_by_key returns the first encountered minimum
        assert_eq!(
            result,
            Some((10, 12)),
            "should return first equidistant spot"
        );
    }

    // ------------------------------------------------------------------
    // find_nearest: single destination within range
    // ------------------------------------------------------------------

    #[test]
    fn test_find_nearest_single_spot_in_range() {
        let spots = vec![(100, 100)];
        let result = find_nearest(&spots, 95, 95, 20);
        assert_eq!(result, Some((100, 100)));
    }

    // ------------------------------------------------------------------
    // find_nearest: grid edge positions (boundary conditions)
    // ------------------------------------------------------------------

    #[test]
    fn test_find_nearest_at_grid_edges() {
        let spots = vec![(0, 0), (255, 255), (0, 255), (255, 0)];

        // From origin, closest is (0, 0)
        assert_eq!(find_nearest(&spots, 0, 0, 30), Some((0, 0)));

        // From max corner, closest is (255, 255)
        assert_eq!(find_nearest(&spots, 255, 255, 30), Some((255, 255)));

        // From (1, 254), closest is (0, 255) with dist 2
        assert_eq!(find_nearest(&spots, 1, 254, 30), Some((0, 255)));
    }

    // ------------------------------------------------------------------
    // find_nearest: large max_dist covers all
    // ------------------------------------------------------------------

    #[test]
    fn test_find_nearest_large_max_dist_returns_closest() {
        let spots = vec![(10, 10), (200, 200)];
        // Even with a huge max_dist, we should still get the closest
        let result = find_nearest(&spots, 12, 12, 1000);
        assert_eq!(
            result,
            Some((10, 10)),
            "should return closest even with large max_dist"
        );
    }
}
