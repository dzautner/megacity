#[cfg(test)]
mod tests {
    use crate::config::{GRID_HEIGHT, GRID_WIDTH};
    use crate::game_params::ZoneDemandParams;
    use crate::grid::{WorldGrid, ZoneType};
    use crate::zones::demand::ZoneDemand;
    use crate::zones::market::{compute_market_demand, vacancy_demand_signal, vacancy_rate};
    use crate::zones::stats::ZoneStats;
    use crate::zones::systems::is_adjacent_to_road;

    // Helper to create a ZoneStats for testing the pure demand function.
    fn make_stats(
        has_roads: bool,
        r_cap: u32,
        r_occ: u32,
        c_cap: u32,
        c_occ: u32,
        i_cap: u32,
        i_occ: u32,
        o_cap: u32,
        o_occ: u32,
    ) -> ZoneStats {
        ZoneStats {
            population: r_occ,
            residential_capacity: r_cap,
            residential_occupants: r_occ,
            commercial_capacity: c_cap,
            commercial_occupants: c_occ,
            industrial_capacity: i_cap,
            industrial_occupants: i_occ,
            office_capacity: o_cap,
            office_occupants: o_occ,
            total_job_capacity: c_cap + i_cap + o_cap,
            total_job_occupants: c_occ + i_occ + o_occ,
            has_roads,
        }
    }

    #[test]
    fn test_zoning_requires_road_adjacency() {
        let grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        // No roads placed, no cell is adjacent to a road
        assert!(!is_adjacent_to_road(&grid, 10, 10));
    }

    #[test]
    fn test_demand_increases_with_roads() {
        // No roads: demand should be zero.
        let zs_no_roads = make_stats(false, 0, 0, 0, 0, 0, 0, 0, 0);
        let (r0, _, _, _) = compute_market_demand(&zs_no_roads);
        assert_eq!(r0, 0.0);

        // Roads but no buildings: bootstrap demand should be positive.
        let zs_roads = make_stats(true, 0, 0, 0, 0, 0, 0, 0, 0);
        let (r1, _, _, _) = compute_market_demand(&zs_roads);
        assert!(r1 > 0.0, "Residential demand should be positive with roads");
    }

    #[test]
    fn test_demand_formula_bounds() {
        let demand = ZoneDemand {
            residential: 0.8,
            commercial: 0.5,
            industrial: 0.3,
            office: 0.2,
            ..Default::default()
        };
        assert!(demand.residential >= 0.0 && demand.residential <= 1.0);
        assert!(demand.commercial >= 0.0 && demand.commercial <= 1.0);
        assert!(demand.industrial >= 0.0 && demand.industrial <= 1.0);
        assert!(demand.office >= 0.0 && demand.office <= 1.0);
    }

    #[test]
    fn test_demand_for_zones() {
        let demand = ZoneDemand {
            residential: 0.8,
            commercial: 0.5,
            industrial: 0.3,
            office: 0.2,
            ..Default::default()
        };
        assert_eq!(demand.demand_for(ZoneType::ResidentialLow), 0.8);
        assert_eq!(demand.demand_for(ZoneType::ResidentialMedium), 0.8);
        assert_eq!(demand.demand_for(ZoneType::ResidentialHigh), 0.8);
        assert_eq!(demand.demand_for(ZoneType::CommercialLow), 0.5);
        assert_eq!(demand.demand_for(ZoneType::CommercialHigh), 0.5);
        assert_eq!(demand.demand_for(ZoneType::Industrial), 0.3);
        assert_eq!(demand.demand_for(ZoneType::Office), 0.2);
        assert_eq!(demand.demand_for(ZoneType::None), 0.0);
    }

    #[test]
    fn test_mixed_use_demand_uses_max() {
        // MixedUse should respond to the higher of residential and commercial demand
        let demand = ZoneDemand {
            residential: 0.8,
            commercial: 0.5,
            industrial: 0.3,
            office: 0.2,
            ..Default::default()
        };
        assert_eq!(demand.demand_for(ZoneType::MixedUse), 0.8);

        let demand2 = ZoneDemand {
            residential: 0.3,
            commercial: 0.9,
            industrial: 0.3,
            office: 0.2,
            ..Default::default()
        };
        assert_eq!(demand2.demand_for(ZoneType::MixedUse), 0.9);
    }

    // -----------------------------------------------------------------------
    // Vacancy rate tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_vacancy_rate_zero_capacity() {
        assert_eq!(vacancy_rate(0, 0), 0.0);
    }

    #[test]
    fn test_vacancy_rate_full_occupancy() {
        assert!((vacancy_rate(100, 100)).abs() < 0.001);
    }

    #[test]
    fn test_vacancy_rate_half_empty() {
        let vr = vacancy_rate(200, 100);
        assert!((vr - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_vacancy_rate_empty_building() {
        let vr = vacancy_rate(100, 0);
        assert!((vr - 1.0).abs() < 0.001);
    }

    // -----------------------------------------------------------------------
    // Vacancy demand signal tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_vacancy_signal_at_midpoint_is_near_zero() {
        // At midpoint of natural vacancy range, signal should be ~0.
        let zdp = ZoneDemandParams::default();
        let mid = (zdp.natural_vacancy_residential.0 + zdp.natural_vacancy_residential.1) * 0.5;
        let sig = vacancy_demand_signal(mid, zdp.natural_vacancy_residential);
        assert!(
            sig.abs() < 0.05,
            "Signal at midpoint should be near zero, got {}",
            sig
        );
    }

    #[test]
    fn test_vacancy_signal_zero_vacancy_is_positive() {
        // 0% vacancy = extremely tight market = high positive demand signal.
        let zdp = ZoneDemandParams::default();
        let sig = vacancy_demand_signal(0.0, zdp.natural_vacancy_residential);
        assert!(sig > 0.0, "Zero vacancy should give positive signal");
    }

    #[test]
    fn test_vacancy_signal_high_vacancy_is_negative() {
        // 50% vacancy = hugely oversupplied = negative demand signal.
        let zdp = ZoneDemandParams::default();
        let sig = vacancy_demand_signal(0.50, zdp.natural_vacancy_residential);
        assert!(sig < 0.0, "High vacancy should give negative signal");
    }

    // -----------------------------------------------------------------------
    // Market demand integration tests (pure function)
    // -----------------------------------------------------------------------

    #[test]
    fn test_zero_vacancy_demand_high() {
        // 0% vacancy across all zones: everything is fully occupied.
        // Jobs exist (meaning employment is available IF vacancy is 0 the jobs
        // are full, so employment_availability is 0 -- but vacancy signal is strong).
        let zs = make_stats(
            true, 1000, 1000, // residential: 100% occupied
            500, 500, // commercial: 100% occupied
            300, 300, // industrial: 100% occupied
            200, 200, // office: 100% occupied
        );
        let (r, c, i, o) = compute_market_demand(&zs);
        // Residential should be high because vacancy signal is strongly positive.
        assert!(
            r > 0.3,
            "0% vacancy should produce high residential demand, got {}",
            r
        );
        // Commercial should also be elevated.
        assert!(
            c > 0.2,
            "0% vacancy should produce elevated commercial demand, got {}",
            c
        );
        // Industrial should be elevated.
        assert!(
            i > 0.2,
            "0% vacancy should produce elevated industrial demand, got {}",
            i
        );
        // Office should be elevated.
        assert!(
            o > 0.2,
            "0% vacancy should produce elevated office demand, got {}",
            o
        );
    }

    #[test]
    fn test_high_vacancy_demand_low() {
        // 80% vacancy (only 20% occupied): massive oversupply.
        let zs = make_stats(
            true, 1000, 200, // residential: 80% vacant
            500, 100, // commercial: 80% vacant
            300, 60, // industrial: 80% vacant
            200, 40, // office: 80% vacant
        );
        let (r, c, i, o) = compute_market_demand(&zs);
        // All demands should be very low with 80% vacancy.
        assert!(
            r < 0.2,
            "80% vacancy should produce low residential demand, got {}",
            r
        );
        assert!(
            c < 0.2,
            "80% vacancy should produce low commercial demand, got {}",
            c
        );
        assert!(
            i < 0.2,
            "80% vacancy should produce low industrial demand, got {}",
            i
        );
        assert!(
            o < 0.2,
            "80% vacancy should produce low office demand, got {}",
            o
        );
    }

    #[test]
    fn test_bootstrap_demand_no_buildings() {
        // Roads exist, no buildings: bootstrap demand should be moderate.
        let zs = make_stats(true, 0, 0, 0, 0, 0, 0, 0, 0);
        let (r, c, i, o) = compute_market_demand(&zs);
        assert!(
            r > 0.3,
            "Bootstrap residential demand should be moderate, got {}",
            r
        );
        assert!(c > 0.0, "Bootstrap commercial demand should be positive");
        assert!(i > 0.0, "Bootstrap industrial demand should be positive");
        assert!(o > 0.0, "Bootstrap office demand should be positive");
    }

    #[test]
    fn test_no_roads_no_demand() {
        let zs = make_stats(false, 0, 0, 0, 0, 0, 0, 0, 0);
        let (r, c, i, o) = compute_market_demand(&zs);
        assert_eq!(r, 0.0);
        assert_eq!(c, 0.0);
        assert_eq!(i, 0.0);
        assert_eq!(o, 0.0);
    }

    #[test]
    fn test_adding_jobs_raises_residential_demand() {
        // Scenario A: few jobs, residential nearly full (low vacancy so the
        // vacancy signal doesn't overwhelm the employment-availability term).
        let zs_few_jobs = make_stats(
            true, 500, 475, // residential: 5% vacancy (within natural range)
            50, 50, // commercial: full
            50, 50, // industrial: full
            50, 50, // office: full
        );
        let (r_few, _, _, _) = compute_market_demand(&zs_few_jobs);

        // Scenario B: many unfilled jobs, same residential occupancy.
        let zs_many_jobs = make_stats(
            true, 500, 475, // residential: 5% vacancy (within natural range)
            500, 50, // commercial: mostly empty (= lots of job openings)
            500, 50, // industrial: mostly empty
            500, 50, // office: mostly empty
        );
        let (r_many, _, _, _) = compute_market_demand(&zs_many_jobs);

        // More available jobs should increase residential demand (people want to move in).
        assert!(
            r_many > r_few,
            "More job availability should raise residential demand: {} vs {}",
            r_many,
            r_few
        );
    }

    #[test]
    fn test_excess_residential_lowers_demand() {
        // Lots of residential capacity, few occupants (high vacancy).
        let zs_excess = make_stats(
            true, 2000, 200, // residential: 90% vacant
            200, 180, // commercial: near full
            200, 180, // industrial: near full
            100, 90, // office: near full
        );
        let (r, _, _, _) = compute_market_demand(&zs_excess);
        assert!(
            r < 0.2,
            "Excess residential should lower demand below 0.2, got {}",
            r
        );
    }

    #[test]
    fn test_demand_values_always_in_bounds() {
        // Test with a variety of extreme parameters.
        let cases = [
            make_stats(true, 0, 0, 0, 0, 0, 0, 0, 0),
            make_stats(true, 100, 100, 100, 100, 100, 100, 100, 100),
            make_stats(true, 100, 0, 100, 0, 100, 0, 100, 0),
            make_stats(true, 1, 1, 1, 1, 1, 1, 1, 1),
            make_stats(true, 100000, 1, 100000, 1, 100000, 1, 100000, 1),
            make_stats(true, 1, 100000, 1, 100000, 1, 100000, 1, 100000),
            make_stats(false, 100, 50, 100, 50, 100, 50, 100, 50),
        ];
        for zs in &cases {
            let (r, c, i, o) = compute_market_demand(zs);
            assert!(r >= 0.0 && r <= 1.0, "Residential out of bounds: {}", r);
            assert!(c >= 0.0 && c <= 1.0, "Commercial out of bounds: {}", c);
            assert!(i >= 0.0 && i <= 1.0, "Industrial out of bounds: {}", i);
            assert!(o >= 0.0 && o <= 1.0, "Office out of bounds: {}", o);
        }
    }

    #[test]
    fn test_damping_smooths_demand_changes() {
        // Simulate starting from zero demand and computing target.
        let mut demand = ZoneDemand::default();
        let zs = make_stats(true, 0, 0, 0, 0, 0, 0, 0, 0);
        let (r_target, c_target, i_target, o_target) = compute_market_demand(&zs);

        // Apply damping once.
        let zdp = ZoneDemandParams::default();
        demand.residential += (r_target - demand.residential) * zdp.damping;
        demand.commercial += (c_target - demand.commercial) * zdp.damping;
        demand.industrial += (i_target - demand.industrial) * zdp.damping;
        demand.office += (o_target - demand.office) * zdp.damping;

        // After one step, demand should be between 0 and target (not at target yet).
        assert!(
            demand.residential < r_target,
            "Damped residential {} should be below target {}",
            demand.residential,
            r_target
        );
        assert!(
            demand.residential > 0.0,
            "Damped residential should be above 0.0"
        );
    }
}
