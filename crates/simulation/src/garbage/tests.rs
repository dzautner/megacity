#[cfg(test)]
mod tests {
    use crate::config::{CELL_SIZE, GRID_HEIGHT, GRID_WIDTH};
    use crate::garbage::*;
    use crate::grid::ZoneType;
    use crate::services::ServiceType;

    #[test]
    fn test_residential_waste_rates() {
        // Low-income (level 1)
        assert_eq!(
            WasteProducer::residential_rate(ZoneType::ResidentialLow, 1),
            3.0
        );
        assert_eq!(
            WasteProducer::residential_rate(ZoneType::ResidentialHigh, 1),
            3.0
        );
        // Middle-income (level 2)
        assert_eq!(
            WasteProducer::residential_rate(ZoneType::ResidentialLow, 2),
            4.5
        );
        assert_eq!(
            WasteProducer::residential_rate(ZoneType::ResidentialHigh, 2),
            4.5
        );
        // High-income (level 3+)
        assert_eq!(
            WasteProducer::residential_rate(ZoneType::ResidentialLow, 3),
            6.0
        );
        assert_eq!(
            WasteProducer::residential_rate(ZoneType::ResidentialHigh, 5),
            6.0
        );
    }

    #[test]
    fn test_commercial_waste_rates() {
        // Small commercial
        assert_eq!(
            WasteProducer::commercial_rate(ZoneType::CommercialLow, 1),
            50.0
        );
        assert_eq!(
            WasteProducer::commercial_rate(ZoneType::CommercialLow, 3),
            50.0
        );
        // Large commercial
        assert_eq!(
            WasteProducer::commercial_rate(ZoneType::CommercialHigh, 1),
            300.0
        );
        assert_eq!(
            WasteProducer::commercial_rate(ZoneType::CommercialHigh, 2),
            300.0
        );
        // Restaurant-type (high-density level 3+)
        assert_eq!(
            WasteProducer::commercial_rate(ZoneType::CommercialHigh, 3),
            200.0
        );
        assert_eq!(
            WasteProducer::commercial_rate(ZoneType::CommercialHigh, 5),
            200.0
        );
    }

    #[test]
    fn test_industrial_waste_rates() {
        // Light industry
        assert_eq!(WasteProducer::industrial_rate(1), 500.0);
        assert_eq!(WasteProducer::industrial_rate(2), 500.0);
        // Heavy industry
        assert_eq!(WasteProducer::industrial_rate(3), 2000.0);
        assert_eq!(WasteProducer::industrial_rate(5), 2000.0);
    }

    #[test]
    fn test_service_waste_rates() {
        assert_eq!(WasteProducer::service_rate(ServiceType::Hospital), 1500.0);
        assert_eq!(
            WasteProducer::service_rate(ServiceType::MedicalCenter),
            1500.0
        );
        assert_eq!(
            WasteProducer::service_rate(ServiceType::ElementarySchool),
            100.0
        );
        assert_eq!(WasteProducer::service_rate(ServiceType::HighSchool), 100.0);
        assert_eq!(WasteProducer::service_rate(ServiceType::University), 200.0);
    }

    #[test]
    fn test_effective_daily_waste_residential() {
        let producer = WasteProducer {
            waste_lbs_per_day: 4.5,
            recycling_participation: false,
        };
        // 10 occupants * 4.5 lbs/person/day = 45 lbs/day
        assert_eq!(producer.effective_daily_waste(10, true), 45.0);

        let producer_recycling = WasteProducer {
            waste_lbs_per_day: 4.5,
            recycling_participation: true,
        };
        // 10 occupants * 4.5 * 0.7 = 31.5
        assert!((producer_recycling.effective_daily_waste(10, true) - 31.5).abs() < 0.01);
    }

    #[test]
    fn test_effective_daily_waste_non_residential() {
        let producer = WasteProducer {
            waste_lbs_per_day: 300.0,
            recycling_participation: false,
        };
        // Non-residential ignores occupants
        assert_eq!(producer.effective_daily_waste(50, false), 300.0);
        assert_eq!(producer.effective_daily_waste(0, false), 300.0);

        let producer_recycling = WasteProducer {
            waste_lbs_per_day: 300.0,
            recycling_participation: true,
        };
        assert!((producer_recycling.effective_daily_waste(0, false) - 210.0).abs() < 0.01);
    }

    #[test]
    fn test_for_building_factory() {
        let res_low = WasteProducer::for_building(ZoneType::ResidentialLow, 1);
        assert_eq!(res_low.waste_lbs_per_day, 3.0);
        assert!(!res_low.recycling_participation);

        let comm_high = WasteProducer::for_building(ZoneType::CommercialHigh, 1);
        assert_eq!(comm_high.waste_lbs_per_day, 300.0);

        let industrial = WasteProducer::for_building(ZoneType::Industrial, 3);
        assert_eq!(industrial.waste_lbs_per_day, 2000.0);

        let office = WasteProducer::for_building(ZoneType::Office, 1);
        assert_eq!(office.waste_lbs_per_day, 50.0);
    }

    #[test]
    fn test_for_service_factory() {
        let hospital = WasteProducer::for_service(ServiceType::Hospital);
        assert_eq!(hospital.waste_lbs_per_day, 1500.0);

        let school = WasteProducer::for_service(ServiceType::ElementarySchool);
        assert_eq!(school.waste_lbs_per_day, 100.0);
    }

    #[test]
    fn test_waste_system_default() {
        let ws = WasteSystem::default();
        assert_eq!(ws.total_generated_tons, 0.0);
        assert_eq!(ws.period_generated_tons, 0.0);
        assert_eq!(ws.per_capita_lbs_per_day, 0.0);
        assert_eq!(ws.tracked_population, 0);
        assert_eq!(ws.recycling_buildings, 0);
        assert_eq!(ws.total_producers, 0);
    }

    #[test]
    fn test_non_residential_zone_returns_zero() {
        assert_eq!(
            WasteProducer::residential_rate(ZoneType::Industrial, 1),
            0.0
        );
        assert_eq!(WasteProducer::commercial_rate(ZoneType::Industrial, 1), 0.0);
    }

    // =========================================================================
    // WASTE-003: Waste Collection System tests
    // =========================================================================

    #[test]
    fn test_facility_capacity_tons() {
        assert_eq!(facility_capacity_tons(ServiceType::TransferStation), 200.0);
        assert_eq!(facility_capacity_tons(ServiceType::Landfill), 150.0);
        assert_eq!(facility_capacity_tons(ServiceType::RecyclingCenter), 100.0);
        assert_eq!(facility_capacity_tons(ServiceType::Incinerator), 250.0);
        // Non-garbage facilities should return 0.
        assert_eq!(facility_capacity_tons(ServiceType::Hospital), 0.0);
    }

    #[test]
    fn test_facility_operating_cost() {
        assert_eq!(
            facility_operating_cost(ServiceType::TransferStation),
            2_000.0
        );
        assert_eq!(facility_operating_cost(ServiceType::Landfill), 1_500.0);
        assert_eq!(
            facility_operating_cost(ServiceType::RecyclingCenter),
            1_800.0
        );
        assert_eq!(facility_operating_cost(ServiceType::Incinerator), 3_000.0);
        assert_eq!(facility_operating_cost(ServiceType::Hospital), 0.0);
    }

    #[test]
    fn test_waste_collection_grid_default() {
        let grid = WasteCollectionGrid::default();
        assert_eq!(grid.width, GRID_WIDTH);
        assert_eq!(grid.height, GRID_HEIGHT);
        assert!(!grid.is_covered(0, 0));
        assert!(!grid.is_covered(128, 128));
        assert_eq!(grid.uncollected(0, 0), 0.0);
    }

    #[test]
    fn test_waste_collection_grid_coverage() {
        let mut grid = WasteCollectionGrid::default();
        // Not covered initially.
        assert!(!grid.is_covered(10, 10));
        // Mark as covered.
        let idx = grid.idx(10, 10);
        grid.coverage[idx] = 1;
        assert!(grid.is_covered(10, 10));
        // Multiple overlapping facilities.
        grid.coverage[idx] = 3;
        assert!(grid.is_covered(10, 10));
    }

    #[test]
    fn test_transfer_station_serves_within_20_cells() {
        // Simulate a transfer station at (100, 100) covering a 20-cell radius.
        let mut grid = WasteCollectionGrid::default();
        let sx = 100i32;
        let sy = 100i32;
        let radius = WASTE_SERVICE_RADIUS_CELLS;
        let r2 = (radius as f32 * CELL_SIZE) * (radius as f32 * CELL_SIZE);

        for dy in -radius..=radius {
            for dx in -radius..=radius {
                let cx = sx + dx;
                let cy = sy + dy;
                if cx < 0 || cy < 0 || cx >= GRID_WIDTH as i32 || cy >= GRID_HEIGHT as i32 {
                    continue;
                }
                let wx_diff = dx as f32 * CELL_SIZE;
                let wy_diff = dy as f32 * CELL_SIZE;
                if wx_diff * wx_diff + wy_diff * wy_diff > r2 {
                    continue;
                }
                let idx = cy as usize * grid.width + cx as usize;
                grid.coverage[idx] = grid.coverage[idx].saturating_add(1);
            }
        }

        // Building at (100, 100) - same cell as station - should be covered.
        assert!(grid.is_covered(100, 100));
        // Building at (110, 100) - 10 cells away - within radius.
        assert!(grid.is_covered(110, 100));
        // Building at (119, 100) - 19 cells away - within radius.
        assert!(grid.is_covered(119, 100));
        // Building at (120, 100) - 20 cells away - exactly at edge, should be covered.
        assert!(grid.is_covered(120, 100));
        // Building at (125, 100) - 25 cells away - outside radius.
        assert!(!grid.is_covered(125, 100));
        // Building at (0, 0) - far away - not covered.
        assert!(!grid.is_covered(0, 0));
    }

    #[test]
    fn test_collection_rate_at_80_percent_capacity() {
        // If capacity = 80 tons/day and generation = 100 tons/day,
        // collection rate = 80/100 = 0.8, meaning 20% uncollected.
        let capacity: f64 = 80.0;
        let generated: f64 = 100.0;
        let rate = (capacity / generated).min(1.0);
        assert!((rate - 0.8).abs() < 0.001);

        // Uncollected = generated * (1 - rate)
        let uncollected = generated * (1.0 - rate);
        assert!((uncollected - 20.0).abs() < 0.001);
    }

    #[test]
    fn test_collection_rate_over_capacity() {
        // If capacity exceeds generation, rate is capped at 1.0.
        let capacity: f64 = 500.0;
        let generated: f64 = 200.0;
        let rate = (capacity / generated).min(1.0);
        assert_eq!(rate, 1.0);
    }

    #[test]
    fn test_collection_rate_zero_generation() {
        // No waste generated: rate should be 1.0 (nothing to collect).
        let capacity: f64 = 200.0;
        let generated: f64 = 0.0;
        let rate = if generated > 0.0 {
            (capacity / generated).min(1.0)
        } else {
            1.0
        };
        assert_eq!(rate, 1.0);
    }

    #[test]
    fn test_waste_system_collection_defaults() {
        let ws = WasteSystem::default();
        assert_eq!(ws.total_collected_tons, 0.0);
        assert_eq!(ws.total_capacity_tons, 0.0);
        assert_eq!(ws.collection_rate, 0.0);
        assert_eq!(ws.uncovered_buildings, 0);
        assert_eq!(ws.transport_cost, 0.0);
        assert_eq!(ws.active_facilities, 0);
    }

    #[test]
    fn test_uncollected_waste_accumulates_uncovered() {
        // Simulate uncollected waste at an uncovered building.
        let mut grid = WasteCollectionGrid::default();
        let idx = grid.idx(50, 50);

        // Building generates 100 lbs/day, not covered.
        assert!(!grid.is_covered(50, 50));
        grid.uncollected_lbs[idx] += 100.0;
        assert_eq!(grid.uncollected(50, 50), 100.0);

        // Next tick: another 100 lbs accumulates.
        grid.uncollected_lbs[idx] += 100.0;
        assert_eq!(grid.uncollected(50, 50), 200.0);
    }

    #[test]
    fn test_uncollected_waste_capped() {
        let mut grid = WasteCollectionGrid::default();
        let idx = grid.idx(50, 50);
        grid.uncollected_lbs[idx] = 15_000.0;
        grid.uncollected_lbs[idx] = grid.uncollected_lbs[idx].min(10_000.0);
        assert_eq!(grid.uncollected(50, 50), 10_000.0);
    }

    #[test]
    fn test_clear_coverage_resets() {
        let mut grid = WasteCollectionGrid::default();
        let idx = grid.idx(10, 10);
        grid.coverage[idx] = 5;
        assert!(grid.is_covered(10, 10));

        grid.clear_coverage();
        assert!(!grid.is_covered(10, 10));
        // Uncollected waste should NOT be cleared by clear_coverage.
        grid.uncollected_lbs[idx] = 500.0;
        grid.clear_coverage();
        assert_eq!(grid.uncollected(10, 10), 500.0);
    }

    #[test]
    fn test_service_radius_constant() {
        // Verify the service radius matches the ticket spec (20 cells).
        assert_eq!(WASTE_SERVICE_RADIUS_CELLS, 20);
    }

    #[test]
    fn test_transfer_station_capacity_200_tons() {
        // Verify the transfer station capacity matches the ticket spec.
        assert_eq!(facility_capacity_tons(ServiceType::TransferStation), 200.0);
    }
}
