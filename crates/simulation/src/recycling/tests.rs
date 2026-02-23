//! Unit tests for the recycling module.

#[cfg(test)]
mod tests {
    use crate::recycling::economics::RecyclingEconomics;
    use crate::recycling::state::RecyclingState;
    use crate::recycling::tiers::RecyclingTier;

    // =========================================================================
    // RecyclingTier tests
    // =========================================================================

    #[test]
    fn tier_none_has_lowest_diversion() {
        assert_eq!(RecyclingTier::None.diversion_rate(), 0.05);
    }

    #[test]
    fn tier_zero_waste_has_highest_diversion() {
        assert_eq!(RecyclingTier::ZeroWaste.diversion_rate(), 0.60);
    }

    #[test]
    fn diversion_rates_ordered() {
        // All tiers except SingleStream should have strictly increasing diversion
        // when traversing None -> VoluntaryDropoff -> CurbsideBasic -> CurbsideSort -> ZeroWaste.
        let ordered = [
            RecyclingTier::None,
            RecyclingTier::VoluntaryDropoff,
            RecyclingTier::CurbsideBasic,
            RecyclingTier::CurbsideSort,
            RecyclingTier::ZeroWaste,
        ];
        for pair in ordered.windows(2) {
            assert!(
                pair[0].diversion_rate() < pair[1].diversion_rate(),
                "{:?} diversion ({}) should be less than {:?} ({})",
                pair[0],
                pair[0].diversion_rate(),
                pair[1],
                pair[1].diversion_rate(),
            );
        }
    }

    #[test]
    fn all_tiers_count() {
        assert_eq!(RecyclingTier::all().len(), 7);
    }

    #[test]
    fn contamination_rates_in_valid_range() {
        for tier in RecyclingTier::all() {
            let rate = tier.contamination_rate();
            assert!(
                (0.15..=0.30).contains(&rate),
                "{:?} contamination rate {rate} outside 15%-30%",
                tier,
            );
        }
    }

    #[test]
    fn single_stream_has_higher_contamination_than_curbside_sort() {
        // Single stream mixes materials, so contamination is worse
        assert!(
            RecyclingTier::SingleStream.contamination_rate()
                > RecyclingTier::CurbsideSort.contamination_rate(),
        );
    }

    #[test]
    fn no_program_has_zero_cost() {
        assert_eq!(RecyclingTier::None.cost_per_household_year(), 0.0);
    }

    #[test]
    fn zero_waste_most_expensive() {
        for tier in RecyclingTier::all() {
            assert!(
                tier.cost_per_household_year()
                    <= RecyclingTier::ZeroWaste.cost_per_household_year(),
                "{:?} cost ({}) exceeds ZeroWaste ({})",
                tier,
                tier.cost_per_household_year(),
                RecyclingTier::ZeroWaste.cost_per_household_year(),
            );
        }
    }

    // =========================================================================
    // RecyclingEconomics tests
    // =========================================================================

    #[test]
    fn default_economics_neutral_multiplier() {
        let econ = RecyclingEconomics::default();
        // At position 0.0, sin(0) = 0, so multiplier = 0.9
        let mult = econ.price_multiplier();
        assert!(
            (mult - 0.9).abs() < 0.01,
            "expected ~0.9 at cycle start, got {mult}"
        );
    }

    #[test]
    fn market_cycle_boom() {
        let mut econ = RecyclingEconomics::default();
        // Position 0.25 => sin(TAU*0.25) = sin(PI/2) = 1.0 => mult = 0.9 + 0.6 = 1.5
        econ.market_cycle_position = 0.25;
        let mult = econ.price_multiplier();
        assert!(
            (mult - 1.5).abs() < 0.01,
            "expected ~1.5 at boom, got {mult}"
        );
    }

    #[test]
    fn market_cycle_bust() {
        let mut econ = RecyclingEconomics::default();
        // Position 0.75 => sin(TAU*0.75) = sin(3PI/2) = -1.0 => mult = 0.9 - 0.6 = 0.3
        econ.market_cycle_position = 0.75;
        let mult = econ.price_multiplier();
        assert!(
            (mult - 0.3).abs() < 0.01,
            "expected ~0.3 at bust, got {mult}"
        );
    }

    #[test]
    fn market_cycle_advance() {
        let mut econ = RecyclingEconomics::default();
        econ.last_update_day = 0;
        // Advance half a cycle (912 days)
        econ.update_market_cycle(912);
        assert!(
            (econ.market_cycle_position - 912.0 / 1825.0).abs() < 0.001,
            "cycle position should be ~0.5, got {}",
            econ.market_cycle_position,
        );
        assert_eq!(econ.last_update_day, 912);
    }

    #[test]
    fn market_cycle_wraps_around() {
        let mut econ = RecyclingEconomics::default();
        econ.last_update_day = 0;
        // Advance more than one full cycle
        econ.update_market_cycle(2000);
        assert!(
            econ.market_cycle_position < 1.0,
            "cycle position should wrap, got {}",
            econ.market_cycle_position,
        );
    }

    #[test]
    fn net_value_per_ton_can_be_negative() {
        let mut econ = RecyclingEconomics::default();
        // At bust (0.3x), revenue should be low enough that net is negative
        econ.market_cycle_position = 0.75;
        let net = econ.net_value_per_ton();
        assert!(
            net < 0.0,
            "net value per ton should be negative during bust, got {net}"
        );
    }

    #[test]
    fn net_value_per_ton_positive_at_boom() {
        let mut econ = RecyclingEconomics::default();
        econ.market_cycle_position = 0.25;
        let net = econ.net_value_per_ton();
        assert!(
            net > 0.0,
            "net value per ton should be positive during boom, got {net}"
        );
    }

    #[test]
    fn revenue_per_ton_scales_with_multiplier() {
        let mut econ = RecyclingEconomics::default();
        econ.market_cycle_position = 0.0;
        let rev_start = econ.revenue_per_ton();

        econ.market_cycle_position = 0.25;
        let rev_boom = econ.revenue_per_ton();

        assert!(
            rev_boom > rev_start,
            "boom revenue ({rev_boom}) should exceed start ({rev_start})"
        );
    }

    // =========================================================================
    // RecyclingState tests
    // =========================================================================

    #[test]
    fn default_state_is_no_program() {
        let state = RecyclingState::default();
        assert_eq!(state.tier, RecyclingTier::None);
        assert_eq!(state.daily_tons_diverted, 0.0);
        assert_eq!(state.daily_revenue, 0.0);
        assert_eq!(state.total_revenue, 0.0);
    }

    #[test]
    fn tier_names_are_unique() {
        let names: Vec<&str> = RecyclingTier::all().iter().map(|t| t.name()).collect();
        for (i, name) in names.iter().enumerate() {
            for (j, other) in names.iter().enumerate() {
                if i != j {
                    assert_ne!(name, other, "duplicate tier name at indices {i} and {j}");
                }
            }
        }
    }

    #[test]
    fn participation_rate_increases_with_better_programs() {
        // None should have lowest participation
        assert!(
            RecyclingTier::None.participation_rate()
                < RecyclingTier::ZeroWaste.participation_rate(),
        );
    }

    #[test]
    fn revenue_potential_none_lowest() {
        for tier in RecyclingTier::all() {
            assert!(
                tier.revenue_potential() >= RecyclingTier::None.revenue_potential(),
                "{:?} revenue potential should be >= None",
                tier,
            );
        }
    }
}
