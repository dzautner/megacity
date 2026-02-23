#[cfg(test)]
mod tests {
    use super::super::constants::*;
    use super::super::helpers::*;
    use super::super::resources::*;
    use crate::config::{GRID_HEIGHT, GRID_WIDTH};

    #[test]
    fn test_forest_fire_grid_default() {
        let grid = ForestFireGrid::default();
        assert_eq!(grid.width, GRID_WIDTH);
        assert_eq!(grid.height, GRID_HEIGHT);
        assert_eq!(grid.intensities.len(), GRID_WIDTH * GRID_HEIGHT);
        assert!(grid.intensities.iter().all(|&v| v == 0));
    }

    #[test]
    fn test_forest_fire_grid_get_set() {
        let mut grid = ForestFireGrid::default();
        assert_eq!(grid.get(10, 20), 0);
        grid.set(10, 20, 150);
        assert_eq!(grid.get(10, 20), 150);
    }

    #[test]
    fn test_forest_fire_grid_boundary() {
        let mut grid = ForestFireGrid::default();
        grid.set(0, 0, 255);
        assert_eq!(grid.get(0, 0), 255);
        grid.set(GRID_WIDTH - 1, GRID_HEIGHT - 1, 100);
        assert_eq!(grid.get(GRID_WIDTH - 1, GRID_HEIGHT - 1), 100);
    }

    #[test]
    fn test_forest_fire_stats_default() {
        let stats = ForestFireStats::default();
        assert_eq!(stats.active_fires, 0);
        assert_eq!(stats.total_area_burned, 0);
        assert_eq!(stats.fires_this_month, 0);
    }

    #[test]
    fn test_fire_hash_deterministic() {
        let a = fire_hash(100, 5000, 0);
        let b = fire_hash(100, 5000, 0);
        assert_eq!(a, b);
    }

    #[test]
    fn test_fire_hash_varies_with_inputs() {
        let a = fire_hash(100, 5000, 0);
        let b = fire_hash(101, 5000, 0);
        let c = fire_hash(100, 5001, 0);
        let d = fire_hash(100, 5000, 1);
        // All should be different (extremely high probability)
        assert_ne!(a, b);
        assert_ne!(a, c);
        assert_ne!(a, d);
    }

    #[test]
    fn test_neighbors4_center() {
        let n = neighbors4(128, 128);
        assert_eq!(n.len(), 4);
        assert!(n.contains(&(127, 128)));
        assert!(n.contains(&(129, 128)));
        assert!(n.contains(&(128, 127)));
        assert!(n.contains(&(128, 129)));
    }

    #[test]
    fn test_neighbors4_corner() {
        let n = neighbors4(0, 0);
        assert_eq!(n.len(), 2);
        assert!(n.contains(&(1, 0)));
        assert!(n.contains(&(0, 1)));
    }

    #[test]
    fn test_neighbors8_center() {
        let n = neighbors8(128, 128);
        assert_eq!(n.len(), 8);
        // Check all 8 directions
        assert!(n.contains(&(127, 127)));
        assert!(n.contains(&(128, 127)));
        assert!(n.contains(&(129, 127)));
        assert!(n.contains(&(127, 128)));
        assert!(n.contains(&(129, 128)));
        assert!(n.contains(&(127, 129)));
        assert!(n.contains(&(128, 129)));
        assert!(n.contains(&(129, 129)));
    }

    #[test]
    fn test_neighbors8_corner() {
        let n = neighbors8(0, 0);
        assert_eq!(n.len(), 3);
        assert!(n.contains(&(1, 0)));
        assert!(n.contains(&(0, 1)));
        assert!(n.contains(&(1, 1)));
    }

    #[test]
    fn test_constants_valid() {
        assert!(FIRE_UPDATE_INTERVAL > 0);
        assert!(INITIAL_INTENSITY > 0);
        assert!(BURNOUT_RATE > 0);
        assert!(RAIN_REDUCTION > BURNOUT_RATE);
        assert!(STORM_REDUCTION > RAIN_REDUCTION);
        assert!(BUILDING_IGNITION_THRESHOLD > INITIAL_INTENSITY);
    }

    #[test]
    fn test_is_near_industrial() {
        use crate::grid::{WorldGrid, ZoneType};
        let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        // No industrial zones by default
        assert!(!is_near_industrial(&grid, 128, 128, 3));

        // Place an industrial zone
        grid.get_mut(130, 128).zone = ZoneType::Industrial;
        assert!(is_near_industrial(&grid, 128, 128, 3));
        assert!(!is_near_industrial(&grid, 128, 128, 1));
    }

    #[test]
    fn test_burnout_reduces_intensity() {
        // Simulate burnout: intensity should decrease by BURNOUT_RATE
        let intensity: u8 = 50;
        let after_burnout = intensity.saturating_sub(BURNOUT_RATE);
        assert_eq!(after_burnout, 50 - BURNOUT_RATE);
    }

    #[test]
    fn test_rain_extinguishes_small_fires() {
        // A small fire (intensity = 5) should be extinguished by rain
        let intensity: u8 = 5;
        let after = intensity
            .saturating_sub(BURNOUT_RATE)
            .saturating_sub(RAIN_REDUCTION);
        assert_eq!(after, 0);
    }

    #[test]
    fn test_storm_extinguishes_moderate_fires() {
        // A moderate fire (intensity = 15) should be extinguished by storm
        let intensity: u8 = 15;
        let after = intensity
            .saturating_sub(BURNOUT_RATE)
            .saturating_sub(STORM_REDUCTION);
        assert_eq!(after, 0);
    }
}
