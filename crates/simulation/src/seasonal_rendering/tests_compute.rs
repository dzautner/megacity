//! Tests for seasonal compute functions (leaf, flower, snow roof, heat shimmer,
//! shadow, brightness, and cell counting).

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use crate::config::{GRID_HEIGHT, GRID_WIDTH};
    use crate::grid::WorldGrid;
    use crate::seasonal_rendering::compute::*;
    use crate::seasonal_rendering::constants::*;
    use crate::trees::TreeGrid;
    use crate::weather::{Season, Weather, WeatherCondition};

    fn test_weather(season: Season, condition: WeatherCondition, temp: f32) -> Weather {
        Weather {
            season,
            current_event: condition,
            temperature: temp,
            precipitation_intensity: match condition {
                WeatherCondition::Rain => 0.3,
                WeatherCondition::HeavyRain => 1.5,
                WeatherCondition::Storm => 2.0,
                WeatherCondition::Snow => 0.5,
                _ => 0.0,
            },
            ..Default::default()
        }
    }

    // -------------------------------------------------------------------------
    // Leaf intensity tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_leaf_intensity_ramps_in_autumn() {
        let intensity = compute_leaf_intensity(0.0, Season::Autumn, true);
        assert!(
            intensity > 0.0,
            "leaf intensity should increase in autumn, got {}",
            intensity
        );
    }

    #[test]
    fn test_leaf_intensity_capped() {
        let intensity = compute_leaf_intensity(0.98, Season::Autumn, true);
        assert!(
            intensity <= MAX_LEAF_INTENSITY,
            "leaf intensity should not exceed {}, got {}",
            MAX_LEAF_INTENSITY,
            intensity
        );
    }

    #[test]
    fn test_leaf_intensity_decays_outside_autumn() {
        let intensity = compute_leaf_intensity(0.5, Season::Summer, true);
        assert!(
            intensity < 0.5,
            "leaf intensity should decay outside autumn, got {}",
            intensity
        );
    }

    #[test]
    fn test_leaf_intensity_zero_when_disabled() {
        let intensity = compute_leaf_intensity(0.5, Season::Autumn, false);
        assert_eq!(intensity, 0.0, "disabled leaves should have 0 intensity");
    }

    #[test]
    fn test_leaf_intensity_decay_floors_at_zero() {
        let intensity = compute_leaf_intensity(0.01, Season::Winter, true);
        assert!(
            intensity >= 0.0,
            "leaf intensity should not go below 0, got {}",
            intensity
        );
    }

    // -------------------------------------------------------------------------
    // Flower intensity tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_flower_intensity_ramps_in_spring() {
        let intensity = compute_flower_intensity(0.0, Season::Spring, true);
        assert!(
            intensity > 0.0,
            "flower intensity should increase in spring, got {}",
            intensity
        );
    }

    #[test]
    fn test_flower_intensity_capped() {
        let intensity = compute_flower_intensity(0.98, Season::Spring, true);
        assert!(
            intensity <= MAX_FLOWER_INTENSITY,
            "flower intensity should not exceed {}, got {}",
            MAX_FLOWER_INTENSITY,
            intensity
        );
    }

    #[test]
    fn test_flower_intensity_decays_outside_spring() {
        let intensity = compute_flower_intensity(0.5, Season::Winter, true);
        assert!(
            intensity < 0.5,
            "flower intensity should decay outside spring, got {}",
            intensity
        );
    }

    #[test]
    fn test_flower_intensity_zero_when_disabled() {
        let intensity = compute_flower_intensity(0.5, Season::Spring, false);
        assert_eq!(intensity, 0.0, "disabled flowers should have 0 intensity");
    }

    // -------------------------------------------------------------------------
    // Snow roof intensity tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_snow_roof_ramps_when_snowing() {
        let weather = test_weather(Season::Winter, WeatherCondition::Snow, -5.0);
        let intensity = compute_snow_roof_intensity(0.0, &weather, 3.0, true);
        assert!(
            intensity > 0.0,
            "snow roof should ramp up when snowing, got {}",
            intensity
        );
    }

    #[test]
    fn test_snow_roof_decays_above_freezing() {
        let weather = test_weather(Season::Spring, WeatherCondition::Sunny, 10.0);
        let intensity = compute_snow_roof_intensity(0.5, &weather, 0.0, true);
        assert!(
            intensity < 0.5,
            "snow roof should decay above freezing with no snow, got {}",
            intensity
        );
    }

    #[test]
    fn test_snow_roof_holds_below_freezing_no_snow() {
        let weather = test_weather(Season::Winter, WeatherCondition::Sunny, -2.0);
        let intensity = compute_snow_roof_intensity(0.3, &weather, 0.5, true);
        assert!(
            (intensity - 0.3).abs() < f32::EPSILON,
            "snow roof should hold below freezing with minimal snow, got {}",
            intensity
        );
    }

    #[test]
    fn test_snow_roof_zero_when_disabled() {
        let weather = test_weather(Season::Winter, WeatherCondition::Snow, -5.0);
        let intensity = compute_snow_roof_intensity(0.5, &weather, 6.0, false);
        assert_eq!(intensity, 0.0, "disabled snow roof should have 0 intensity");
    }

    #[test]
    fn test_snow_roof_capped() {
        let weather = test_weather(Season::Winter, WeatherCondition::Snow, -10.0);
        let intensity = compute_snow_roof_intensity(0.95, &weather, 12.0, true);
        assert!(
            intensity <= MAX_SNOW_ROOF_INTENSITY,
            "snow roof intensity should not exceed {}, got {}",
            MAX_SNOW_ROOF_INTENSITY,
            intensity
        );
    }

    // -------------------------------------------------------------------------
    // Heat shimmer tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_heat_shimmer_active_in_summer_heat() {
        let intensity = compute_heat_shimmer_intensity(35.0, Season::Summer, true);
        assert!(
            intensity > 0.0,
            "heat shimmer should be active at 35C in summer, got {}",
            intensity
        );
    }

    #[test]
    fn test_heat_shimmer_zero_below_threshold() {
        let intensity = compute_heat_shimmer_intensity(25.0, Season::Summer, true);
        assert_eq!(
            intensity, 0.0,
            "heat shimmer should be zero below threshold"
        );
    }

    #[test]
    fn test_heat_shimmer_zero_outside_summer() {
        let intensity = compute_heat_shimmer_intensity(35.0, Season::Spring, true);
        assert_eq!(intensity, 0.0, "heat shimmer should be zero outside summer");
    }

    #[test]
    fn test_heat_shimmer_zero_when_disabled() {
        let intensity = compute_heat_shimmer_intensity(40.0, Season::Summer, false);
        assert_eq!(intensity, 0.0, "disabled heat shimmer should be zero");
    }

    #[test]
    fn test_heat_shimmer_scales_with_temperature() {
        let low = compute_heat_shimmer_intensity(32.0, Season::Summer, true);
        let high = compute_heat_shimmer_intensity(38.0, Season::Summer, true);
        assert!(
            high > low,
            "higher temperature should produce more shimmer: {} vs {}",
            high,
            low
        );
    }

    #[test]
    fn test_heat_shimmer_capped() {
        let intensity = compute_heat_shimmer_intensity(50.0, Season::Summer, true);
        assert!(
            intensity <= MAX_HEAT_SHIMMER_INTENSITY,
            "heat shimmer should be capped at {}, got {}",
            MAX_HEAT_SHIMMER_INTENSITY,
            intensity
        );
    }

    // -------------------------------------------------------------------------
    // Shadow multiplier tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_shadow_multiplier_summer() {
        let mult = compute_shadow_multiplier(Season::Summer, true);
        assert!(
            (mult - SUMMER_SHADOW_MULTIPLIER).abs() < f32::EPSILON,
            "summer should have shadow multiplier {}, got {}",
            SUMMER_SHADOW_MULTIPLIER,
            mult
        );
    }

    #[test]
    fn test_shadow_multiplier_other_seasons() {
        for season in [Season::Spring, Season::Autumn, Season::Winter] {
            let mult = compute_shadow_multiplier(season, true);
            assert!(
                (mult - 1.0).abs() < f32::EPSILON,
                "{:?} should have shadow multiplier 1.0, got {}",
                season,
                mult
            );
        }
    }

    #[test]
    fn test_shadow_multiplier_disabled() {
        let mult = compute_shadow_multiplier(Season::Summer, false);
        assert!(
            (mult - 1.0).abs() < f32::EPSILON,
            "disabled should have shadow multiplier 1.0, got {}",
            mult
        );
    }

    // -------------------------------------------------------------------------
    // Spring brightness tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_spring_brightness_active() {
        let brightness = compute_spring_brightness(Season::Spring, true);
        assert!(
            (brightness - SPRING_BRIGHTNESS_BOOST).abs() < f32::EPSILON,
            "spring should have brightness boost {}, got {}",
            SPRING_BRIGHTNESS_BOOST,
            brightness
        );
    }

    #[test]
    fn test_spring_brightness_other_seasons() {
        for season in [Season::Summer, Season::Autumn, Season::Winter] {
            let brightness = compute_spring_brightness(season, true);
            assert!(
                brightness.abs() < f32::EPSILON,
                "{:?} should have 0 brightness boost, got {}",
                season,
                brightness
            );
        }
    }

    #[test]
    fn test_spring_brightness_disabled() {
        let brightness = compute_spring_brightness(Season::Spring, false);
        assert!(
            brightness.abs() < f32::EPSILON,
            "disabled should have 0 brightness boost, got {}",
            brightness
        );
    }

    // -------------------------------------------------------------------------
    // Cell counting tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_count_tree_cells_empty() {
        let grid = TreeGrid::default();
        assert_eq!(count_tree_cells(&grid), 0);
    }

    #[test]
    fn test_count_tree_cells_some() {
        let mut grid = TreeGrid::default();
        grid.set(5, 5, true);
        grid.set(10, 10, true);
        grid.set(15, 15, true);
        assert_eq!(count_tree_cells(&grid), 3);
    }

    #[test]
    fn test_count_building_cells_empty() {
        let grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        assert_eq!(count_building_cells(&grid), 0);
    }

    #[test]
    fn test_count_building_cells_some() {
        let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        grid.get_mut(5, 5).building_id = Some(Entity::from_raw(1));
        grid.get_mut(10, 10).building_id = Some(Entity::from_raw(2));
        assert_eq!(count_building_cells(&grid), 2);
    }

    // -------------------------------------------------------------------------
    // Flower cell counting tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_count_flower_cells_empty() {
        let world_grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        let tree_grid = TreeGrid::default();
        assert_eq!(count_flower_cells(&world_grid, &tree_grid), 0);
    }

    #[test]
    fn test_count_flower_cells_with_trees() {
        let world_grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        let mut tree_grid = TreeGrid::default();
        tree_grid.set(5, 5, true);
        tree_grid.set(10, 10, true);
        assert_eq!(count_flower_cells(&world_grid, &tree_grid), 2);
    }

    #[test]
    fn test_count_flower_cells_residential_zones() {
        let mut world_grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        let tree_grid = TreeGrid::default();
        world_grid.get_mut(3, 3).zone = crate::grid::ZoneType::ResidentialLow;
        world_grid.get_mut(4, 4).zone = crate::grid::ZoneType::ResidentialMedium;
        assert_eq!(count_flower_cells(&world_grid, &tree_grid), 2);
    }

    #[test]
    fn test_count_flower_cells_excludes_buildings() {
        let mut world_grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        let mut tree_grid = TreeGrid::default();
        tree_grid.set(5, 5, true);
        world_grid.get_mut(5, 5).building_id = Some(Entity::from_raw(1));
        assert_eq!(count_flower_cells(&world_grid, &tree_grid), 0);
    }
}
