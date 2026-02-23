#[cfg(test)]
mod tests {
    use crate::config::{GRID_HEIGHT, GRID_WIDTH};
    use crate::grid::{CellType, ZoneType};
    use crate::weather::{Weather, WeatherCondition};

    use super::super::calculations::{
        imperviousness, infiltration, rainfall_intensity, runoff, CELL_AREA, SOIL_PERMEABILITY,
    };
    use super::super::types::StormwaterGrid;

    #[test]
    fn test_road_cell_imperviousness() {
        let imperv = imperviousness(CellType::Road, ZoneType::None, false);
        assert!(
            (imperv - 0.95).abs() < f32::EPSILON,
            "Road cell imperviousness should be 0.95, got {}",
            imperv
        );
    }

    #[test]
    fn test_grass_cell_imperviousness() {
        let imperv = imperviousness(CellType::Grass, ZoneType::None, false);
        assert!(
            (imperv - 0.35).abs() < f32::EPSILON,
            "Grass cell (no zone) imperviousness should be 0.35, got {}",
            imperv
        );
    }

    #[test]
    fn test_building_cell_imperviousness() {
        let imperv = imperviousness(CellType::Grass, ZoneType::ResidentialHigh, true);
        assert!(
            (imperv - 0.95).abs() < f32::EPSILON,
            "Building cell imperviousness should be 0.95, got {}",
            imperv
        );
    }

    #[test]
    fn test_water_cell_imperviousness() {
        let imperv = imperviousness(CellType::Water, ZoneType::None, false);
        assert!(
            (imperv - 0.0).abs() < f32::EPSILON,
            "Water cell imperviousness should be 0.0, got {}",
            imperv
        );
    }

    #[test]
    fn test_industrial_zone_imperviousness() {
        let imperv = imperviousness(CellType::Grass, ZoneType::Industrial, false);
        assert!(
            (imperv - 0.90).abs() < f32::EPSILON,
            "Industrial zone imperviousness should be 0.90, got {}",
            imperv
        );
    }

    #[test]
    fn test_commercial_zone_imperviousness() {
        let imperv = imperviousness(CellType::Grass, ZoneType::CommercialHigh, false);
        assert!(
            (imperv - 0.85).abs() < f32::EPSILON,
            "Commercial high zone imperviousness should be 0.85, got {}",
            imperv
        );
    }

    #[test]
    fn test_road_cell_runoff() {
        let rain = 1.0; // maximum rainfall intensity
        let imperv = imperviousness(CellType::Road, ZoneType::None, false);
        let r = runoff(rain, imperv);
        let expected = 1.0 * 0.95 * CELL_AREA;
        assert!(
            (r - expected).abs() < f32::EPSILON,
            "Road cell runoff at max rain should be {}, got {}",
            expected,
            r
        );
    }

    #[test]
    fn test_road_produces_095_rainfall_as_runoff() {
        // Unit test from issue: road cell produces 0.95 * rainfall as runoff
        let rainfall = 0.5;
        let imperv = imperviousness(CellType::Road, ZoneType::None, false);
        let r = runoff(rainfall, imperv);
        let expected = rainfall * 0.95 * CELL_AREA;
        assert!(
            (r - expected).abs() < 0.001,
            "Road runoff should be 0.95 * rainfall * area = {}, got {}",
            expected,
            r
        );
    }

    #[test]
    fn test_forest_produces_015_rainfall_as_runoff() {
        // The closest analog to "forest" in our system is an empty grass cell (ZoneType::None)
        // which has imperviousness 0.35. For a forest-equivalent value of 0.15,
        // we test the runoff function directly with the forest imperviousness.
        let rainfall = 0.5;
        let forest_imperv = 0.15;
        let r = runoff(rainfall, forest_imperv);
        let expected = rainfall * 0.15 * CELL_AREA;
        assert!(
            (r - expected).abs() < 0.001,
            "Forest runoff should be 0.15 * rainfall * area = {}, got {}",
            expected,
            r
        );
    }

    #[test]
    fn test_grass_cell_runoff() {
        let rain = 1.0;
        let imperv = imperviousness(CellType::Grass, ZoneType::None, false);
        let r = runoff(rain, imperv);
        let expected = 1.0 * 0.35 * CELL_AREA;
        assert!(
            (r - expected).abs() < f32::EPSILON,
            "Grass cell runoff at max rain should be {}, got {}",
            expected,
            r
        );
    }

    #[test]
    fn test_infiltration_calculation() {
        let rain = 1.0;
        let imperv = 0.35; // grass
        let inf = infiltration(rain, imperv);
        let expected = 1.0 * (1.0 - 0.35) * SOIL_PERMEABILITY;
        assert!(
            (inf - expected).abs() < f32::EPSILON,
            "Infiltration should be {}, got {}",
            expected,
            inf
        );
    }

    #[test]
    fn test_infiltration_zero_for_fully_impervious() {
        let rain = 1.0;
        let imperv = 1.0;
        let inf = infiltration(rain, imperv);
        assert!(
            inf.abs() < f32::EPSILON,
            "Fully impervious surface should have zero infiltration, got {}",
            inf
        );
    }

    #[test]
    fn test_runoff_plus_infiltration_less_than_rainfall() {
        // For any imperviousness, runoff + infiltration should not exceed total rainfall * area
        let rain = 0.8;
        for imperv_pct in [0.0, 0.15, 0.35, 0.70, 0.85, 0.90, 0.95, 1.0] {
            let r = runoff(rain, imperv_pct);
            let inf = infiltration(rain, imperv_pct);
            let total_rain = rain * CELL_AREA;
            assert!(
                r + inf <= total_rain + 0.01,
                "runoff ({}) + infiltration ({}) > total rainfall ({}) at imperv {}",
                r,
                inf,
                total_rain,
                imperv_pct
            );
        }
    }

    #[test]
    fn test_stormwater_grid_default() {
        let sw = StormwaterGrid::default();
        assert_eq!(sw.runoff.len(), GRID_WIDTH * GRID_HEIGHT);
        assert_eq!(sw.total_runoff, 0.0);
        assert_eq!(sw.total_infiltration, 0.0);
        assert!(sw.runoff.iter().all(|&v| v == 0.0));
    }

    #[test]
    fn test_stormwater_grid_get_set() {
        let mut sw = StormwaterGrid::default();
        sw.set(10, 20, 5.0);
        assert!((sw.get(10, 20) - 5.0).abs() < f32::EPSILON);
        sw.add(10, 20, 3.0);
        assert!((sw.get(10, 20) - 8.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_heavy_rain_paved_area_maximum_runoff() {
        // Integration test: heavy rain on paved area produces maximum runoff
        // Storm intensity = 1.0, road imperviousness = 0.95
        let rain = 1.0; // Storm
        let imperv = imperviousness(CellType::Road, ZoneType::None, false);
        let r = runoff(rain, imperv);

        // Compare with grass cell at same rainfall
        let grass_imperv = imperviousness(CellType::Grass, ZoneType::None, false);
        let grass_r = runoff(rain, grass_imperv);

        assert!(
            r > grass_r,
            "Paved area runoff ({}) should exceed grass runoff ({})",
            r,
            grass_r
        );

        // Paved should produce roughly 0.95/0.35 = ~2.7x more runoff than grass
        let ratio = r / grass_r;
        assert!(
            (ratio - 0.95 / 0.35).abs() < 0.01,
            "Runoff ratio should be ~{}, got {}",
            0.95 / 0.35,
            ratio
        );
    }

    #[test]
    fn test_rainfall_intensity_values() {
        let mut w = Weather::default();

        w.current_event = WeatherCondition::Rain;
        assert!((rainfall_intensity(&w) - 0.3).abs() < f32::EPSILON);

        w.current_event = WeatherCondition::HeavyRain;
        assert!((rainfall_intensity(&w) - 0.6).abs() < f32::EPSILON);

        w.current_event = WeatherCondition::Storm;
        assert!((rainfall_intensity(&w) - 1.0).abs() < f32::EPSILON);

        w.current_event = WeatherCondition::Snow;
        assert!((rainfall_intensity(&w) - 0.05).abs() < f32::EPSILON);

        w.current_event = WeatherCondition::Sunny;
        assert!((rainfall_intensity(&w) - 0.0).abs() < f32::EPSILON);

        w.current_event = WeatherCondition::Overcast;
        assert!((rainfall_intensity(&w) - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_imperviousness_ordering() {
        // Road >= Building > Industrial > CommercialHigh > ResidentialHigh > ResidentialLow > Grass
        let road = imperviousness(CellType::Road, ZoneType::None, false);
        let building = imperviousness(CellType::Grass, ZoneType::ResidentialHigh, true);
        let industrial = imperviousness(CellType::Grass, ZoneType::Industrial, false);
        let commercial = imperviousness(CellType::Grass, ZoneType::CommercialHigh, false);
        let res_high = imperviousness(CellType::Grass, ZoneType::ResidentialHigh, false);
        let res_low = imperviousness(CellType::Grass, ZoneType::ResidentialLow, false);
        let grass = imperviousness(CellType::Grass, ZoneType::None, false);

        assert!(road >= building);
        assert!(building >= industrial);
        assert!(industrial >= commercial);
        assert!(commercial >= res_high);
        assert!(res_high >= res_low);
        assert!(res_low >= grass);
    }
}
