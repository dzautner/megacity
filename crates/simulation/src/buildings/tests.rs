#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use crate::buildings::types::{max_level_for_far, Building, MixedUseBuilding};
    use crate::config::{GRID_HEIGHT, GRID_WIDTH};
    use crate::grid::{CellType, WorldGrid, ZoneType};
    use crate::zones::is_adjacent_to_road;

    #[test]
    fn test_building_capacity() {
        assert_eq!(
            Building::capacity_for_level(ZoneType::ResidentialLow, 1),
            10
        );
        assert_eq!(
            Building::capacity_for_level(ZoneType::ResidentialLow, 2),
            30
        );
        assert_eq!(
            Building::capacity_for_level(ZoneType::ResidentialHigh, 3),
            500
        );
        assert_eq!(
            Building::capacity_for_level(ZoneType::ResidentialHigh, 5),
            2000
        );
        // Medium-density residential
        assert_eq!(
            Building::capacity_for_level(ZoneType::ResidentialMedium, 1),
            15
        );
        assert_eq!(
            Building::capacity_for_level(ZoneType::ResidentialMedium, 2),
            50
        );
        assert_eq!(
            Building::capacity_for_level(ZoneType::ResidentialMedium, 3),
            120
        );
        assert_eq!(
            Building::capacity_for_level(ZoneType::ResidentialMedium, 4),
            250
        );
        assert_eq!(Building::capacity_for_level(ZoneType::CommercialLow, 1), 8);
        assert_eq!(Building::capacity_for_level(ZoneType::Industrial, 1), 20);
        assert_eq!(Building::capacity_for_level(ZoneType::Office, 1), 30);
    }

    #[test]
    fn test_mixed_use_capacity_per_level() {
        // L1=(5 comm, 8 res)
        assert_eq!(MixedUseBuilding::capacities_for_level(1), (5, 8));
        assert_eq!(MixedUseBuilding::total_capacity_for_level(1), 13);
        // L2=(15, 30)
        assert_eq!(MixedUseBuilding::capacities_for_level(2), (15, 30));
        assert_eq!(MixedUseBuilding::total_capacity_for_level(2), 45);
        // L3=(40, 80) — 20 commercial + 20 office = 40 commercial
        assert_eq!(MixedUseBuilding::capacities_for_level(3), (40, 80));
        assert_eq!(MixedUseBuilding::total_capacity_for_level(3), 120);
        // L4=(120, 200) — 40 commercial + 80 office = 120 commercial
        assert_eq!(MixedUseBuilding::capacities_for_level(4), (120, 200));
        assert_eq!(MixedUseBuilding::total_capacity_for_level(4), 320);
        // L5=(280, 400) — 80 commercial + 200 office = 280 commercial
        assert_eq!(MixedUseBuilding::capacities_for_level(5), (280, 400));
        assert_eq!(MixedUseBuilding::total_capacity_for_level(5), 680);
    }

    #[test]
    fn test_mixed_use_building_capacity_matches_total() {
        for level in 1..=5 {
            let total = Building::capacity_for_level(ZoneType::MixedUse, level);
            let (c, r) = MixedUseBuilding::capacities_for_level(level);
            assert_eq!(total, c + r, "Level {} total mismatch", level);
        }
    }

    #[test]
    fn test_building_only_in_zoned_cells() {
        let grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        for cell in &grid.cells {
            assert!(cell.building_id.is_none());
            assert_eq!(cell.zone, ZoneType::None);
        }
    }

    #[test]
    fn test_eligible_cells_finds_zoned_cells() {
        let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        // Place a road at (10, 10)
        grid.get_mut(10, 10).cell_type = CellType::Road;
        // Zone cells adjacent to road
        for x in 8..=9 {
            let cell = grid.get_mut(x, 10);
            cell.zone = ZoneType::ResidentialLow;
            cell.has_power = true;
            cell.has_water = true;
        }

        let mut res_cells = Vec::new();
        for y in 0..GRID_HEIGHT {
            for x in 0..GRID_WIDTH {
                let cell = grid.get(x, y);
                if cell.zone == ZoneType::ResidentialLow
                    && cell.building_id.is_none()
                    && cell.cell_type == CellType::Grass
                    && cell.has_power
                    && cell.has_water
                    && is_adjacent_to_road(&grid, x, y)
                {
                    res_cells.push((x, y));
                }
            }
        }

        assert_eq!(res_cells.len(), 2);
        assert!(res_cells.contains(&(8, 10)));
        assert!(res_cells.contains(&(9, 10)));
    }

    #[test]
    fn test_eligible_cells_excludes_occupied() {
        let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        grid.get_mut(10, 10).cell_type = CellType::Road;
        let cell = grid.get_mut(9, 10);
        cell.zone = ZoneType::Industrial;
        cell.has_power = true;
        cell.has_water = true;
        // Mark as having a building
        cell.building_id = Some(Entity::from_raw(1));

        let mut eligible_count = 0;
        for y in 0..GRID_HEIGHT {
            for x in 0..GRID_WIDTH {
                let cell = grid.get(x, y);
                if cell.zone == ZoneType::Industrial
                    && cell.building_id.is_none()
                    && cell.cell_type == CellType::Grass
                    && cell.has_power
                    && cell.has_water
                    && is_adjacent_to_road(&grid, x, y)
                {
                    eligible_count += 1;
                }
            }
        }

        assert_eq!(eligible_count, 0);
    }

    #[test]
    fn test_far_residential_low_limits_level() {
        // ResidentialLow FAR=0.5 should constrain building to low levels.
        // L1: capacity=10, implied_far = 10*20/256 = 0.78 > 0.5
        // So max_level_for_far should return 1 (the minimum).
        let max = max_level_for_far(ZoneType::ResidentialLow);
        assert!(max >= 1, "max_level_for_far must return at least 1");
        assert!(
            max <= 3,
            "ResidentialLow FAR=0.5 should limit to low levels, got {}",
            max
        );
    }

    #[test]
    fn test_far_residential_high_allows_higher_levels() {
        // ResidentialHigh FAR=3.0 should allow higher levels than ResidentialLow.
        let high = max_level_for_far(ZoneType::ResidentialHigh);
        let low = max_level_for_far(ZoneType::ResidentialLow);
        assert!(
            high >= low,
            "ResidentialHigh should allow at least as many levels as ResidentialLow"
        );
    }

    #[test]
    fn test_far_returns_at_least_one() {
        // All zone types (except None) should return at least 1.
        let zones = [
            ZoneType::ResidentialLow,
            ZoneType::ResidentialMedium,
            ZoneType::ResidentialHigh,
            ZoneType::CommercialLow,
            ZoneType::CommercialHigh,
            ZoneType::Industrial,
            ZoneType::Office,
            ZoneType::MixedUse,
        ];
        for zone in zones {
            let max = max_level_for_far(zone);
            assert!(
                max >= 1,
                "max_level_for_far({:?}) must be >= 1, got {}",
                zone,
                max
            );
        }
    }

    #[test]
    fn test_far_respects_zone_max_level() {
        // max_level_for_far should not exceed the zone's max_level.
        let zones = [
            ZoneType::ResidentialLow,
            ZoneType::ResidentialMedium,
            ZoneType::ResidentialHigh,
            ZoneType::CommercialLow,
            ZoneType::CommercialHigh,
            ZoneType::Industrial,
            ZoneType::Office,
            ZoneType::MixedUse,
        ];
        for zone in zones {
            let far_max = max_level_for_far(zone);
            let zone_max = zone.max_level() as u32;
            assert!(
                far_max <= zone_max,
                "max_level_for_far({:?})={} should not exceed max_level={}",
                zone,
                far_max,
                zone_max
            );
        }
    }

    #[test]
    fn test_default_far_values() {
        assert_eq!(ZoneType::ResidentialLow.default_far(), 0.5);
        assert_eq!(ZoneType::ResidentialMedium.default_far(), 1.5);
        assert_eq!(ZoneType::ResidentialHigh.default_far(), 3.0);
        assert_eq!(ZoneType::CommercialLow.default_far(), 1.5);
        assert_eq!(ZoneType::CommercialHigh.default_far(), 3.0);
        assert_eq!(ZoneType::Industrial.default_far(), 0.8);
        assert_eq!(ZoneType::Office.default_far(), 1.5);
        assert_eq!(ZoneType::MixedUse.default_far(), 3.0);
        assert_eq!(ZoneType::None.default_far(), 0.0);
    }
}
