//! Unit tests for NIMBY opinion calculation, density helpers, and effect functions.

#[cfg(test)]
mod tests {
    use crate::citizen::Personality;
    use crate::grid::ZoneType;
    use crate::nimby::opinion::{
        calculate_opinion, construction_slowdown, is_residential, nimby_happiness_penalty,
        zone_density_score,
    };
    use crate::nimby::types::{MAX_CONSTRUCTION_SLOWDOWN, MAX_NIMBY_HAPPINESS_PENALTY};

    // -------------------------------------------------------------------------
    // Zone density score tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_zone_density_scores_increase_with_density() {
        assert!(
            zone_density_score(ZoneType::ResidentialLow)
                < zone_density_score(ZoneType::ResidentialMedium)
        );
        assert!(
            zone_density_score(ZoneType::ResidentialMedium)
                < zone_density_score(ZoneType::ResidentialHigh)
        );
        assert!(
            zone_density_score(ZoneType::CommercialLow)
                < zone_density_score(ZoneType::CommercialHigh)
        );
        assert!(zone_density_score(ZoneType::None) < zone_density_score(ZoneType::ResidentialLow));
    }

    #[test]
    fn test_industrial_has_highest_density_score() {
        assert!(
            zone_density_score(ZoneType::Industrial)
                > zone_density_score(ZoneType::ResidentialHigh)
        );
    }

    // -------------------------------------------------------------------------
    // is_residential tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_is_residential() {
        assert!(is_residential(ZoneType::ResidentialLow));
        assert!(is_residential(ZoneType::ResidentialMedium));
        assert!(is_residential(ZoneType::ResidentialHigh));
        assert!(!is_residential(ZoneType::CommercialLow));
        assert!(!is_residential(ZoneType::Industrial));
        assert!(!is_residential(ZoneType::None));
        assert!(!is_residential(ZoneType::Office));
        assert!(!is_residential(ZoneType::MixedUse));
    }

    // -------------------------------------------------------------------------
    // Opinion calculation tests
    // -------------------------------------------------------------------------

    fn default_personality() -> Personality {
        Personality {
            ambition: 0.5,
            sociability: 0.5,
            materialism: 0.5,
            resilience: 0.5,
        }
    }

    #[test]
    fn test_density_increase_causes_opposition() {
        let opinion = calculate_opinion(
            ZoneType::ResidentialLow,
            ZoneType::ResidentialHigh,
            1.0,
            128,
            2,
            &default_personality(),
            false,
            false,
            0.10,
        );
        assert!(
            opinion > 0.0,
            "density increase should cause opposition, got {}",
            opinion
        );
    }

    #[test]
    fn test_industrial_rezoning_causes_strong_opposition() {
        let opinion = calculate_opinion(
            ZoneType::ResidentialLow,
            ZoneType::Industrial,
            1.0,
            128,
            2,
            &default_personality(),
            false,
            false,
            0.10,
        );
        assert!(
            opinion > 10.0,
            "industrial rezoning should cause strong opposition, got {}",
            opinion
        );
    }

    #[test]
    fn test_job_creation_reduces_opposition() {
        let opinion = calculate_opinion(
            ZoneType::None,
            ZoneType::CommercialLow,
            1.0,
            128,
            2,
            &default_personality(),
            false,
            false,
            0.10,
        );
        let opinion_no_jobs = calculate_opinion(
            ZoneType::None,
            ZoneType::Industrial,
            1.0,
            128,
            2,
            &default_personality(),
            false,
            false,
            0.10,
        );
        assert!(
            opinion < opinion_no_jobs,
            "job creation should produce less opposition than industrial"
        );
    }

    #[test]
    fn test_housing_need_creates_support() {
        let opinion_low_vacancy = calculate_opinion(
            ZoneType::None,
            ZoneType::ResidentialHigh,
            1.0,
            128,
            2,
            &default_personality(),
            false,
            false,
            0.02,
        );
        let opinion_high_vacancy = calculate_opinion(
            ZoneType::None,
            ZoneType::ResidentialHigh,
            1.0,
            128,
            2,
            &default_personality(),
            false,
            false,
            0.20,
        );
        assert!(
            opinion_low_vacancy < opinion_high_vacancy,
            "low vacancy should produce more support: low={}, high={}",
            opinion_low_vacancy,
            opinion_high_vacancy
        );
    }

    #[test]
    fn test_park_coverage_reduces_opposition() {
        let opinion_no_park = calculate_opinion(
            ZoneType::None,
            ZoneType::ResidentialHigh,
            1.0,
            128,
            2,
            &default_personality(),
            false,
            false,
            0.10,
        );
        let opinion_with_park = calculate_opinion(
            ZoneType::None,
            ZoneType::ResidentialHigh,
            1.0,
            128,
            2,
            &default_personality(),
            true,
            false,
            0.10,
        );
        assert!(
            opinion_with_park < opinion_no_park,
            "park coverage should reduce opposition: park={}, no_park={}",
            opinion_with_park,
            opinion_no_park
        );
    }

    #[test]
    fn test_transit_coverage_reduces_opposition() {
        let opinion_no_transit = calculate_opinion(
            ZoneType::None,
            ZoneType::ResidentialHigh,
            1.0,
            128,
            2,
            &default_personality(),
            false,
            false,
            0.10,
        );
        let opinion_with_transit = calculate_opinion(
            ZoneType::None,
            ZoneType::ResidentialHigh,
            1.0,
            128,
            2,
            &default_personality(),
            false,
            true,
            0.10,
        );
        assert!(
            opinion_with_transit < opinion_no_transit,
            "transit should reduce opposition: transit={}, no_transit={}",
            opinion_with_transit,
            opinion_no_transit
        );
    }

    #[test]
    fn test_distance_reduces_opposition() {
        let close_opinion = calculate_opinion(
            ZoneType::None,
            ZoneType::Industrial,
            1.0,
            128,
            2,
            &default_personality(),
            false,
            false,
            0.10,
        );
        let far_opinion = calculate_opinion(
            ZoneType::None,
            ZoneType::Industrial,
            6.0,
            128,
            2,
            &default_personality(),
            false,
            false,
            0.10,
        );
        assert!(
            far_opinion < close_opinion,
            "distance should reduce opposition: close={}, far={}",
            close_opinion,
            far_opinion
        );
    }

    #[test]
    fn test_high_land_value_increases_opposition() {
        let low_lv_opinion = calculate_opinion(
            ZoneType::None,
            ZoneType::Industrial,
            1.0,
            50,
            2,
            &default_personality(),
            false,
            false,
            0.10,
        );
        let high_lv_opinion = calculate_opinion(
            ZoneType::None,
            ZoneType::Industrial,
            1.0,
            250,
            2,
            &default_personality(),
            false,
            false,
            0.10,
        );
        assert!(
            high_lv_opinion > low_lv_opinion,
            "high land value should increase opposition: high={}, low={}",
            high_lv_opinion,
            low_lv_opinion
        );
    }

    // -------------------------------------------------------------------------
    // Construction slowdown tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_construction_slowdown_zero_opposition() {
        assert_eq!(construction_slowdown(0.0), 0);
        assert_eq!(construction_slowdown(-5.0), 0);
    }

    #[test]
    fn test_construction_slowdown_moderate_opposition() {
        let slow = construction_slowdown(20.0);
        assert!(
            slow > 0 && slow <= MAX_CONSTRUCTION_SLOWDOWN,
            "moderate opposition should cause some slowdown: {}",
            slow
        );
    }

    #[test]
    fn test_construction_slowdown_capped() {
        let slow = construction_slowdown(1000.0);
        assert_eq!(
            slow, MAX_CONSTRUCTION_SLOWDOWN,
            "slowdown should be capped at max: {}",
            slow
        );
    }

    // -------------------------------------------------------------------------
    // Happiness penalty tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_happiness_penalty_zero() {
        assert_eq!(nimby_happiness_penalty(0.0), 0.0);
        assert_eq!(nimby_happiness_penalty(-5.0), 0.0);
    }

    #[test]
    fn test_happiness_penalty_moderate() {
        let penalty = nimby_happiness_penalty(10.0);
        assert!(
            penalty > 0.0 && penalty <= MAX_NIMBY_HAPPINESS_PENALTY,
            "moderate opposition should cause some penalty: {}",
            penalty
        );
    }

    #[test]
    fn test_happiness_penalty_capped() {
        let penalty = nimby_happiness_penalty(1000.0);
        assert!(
            (penalty - MAX_NIMBY_HAPPINESS_PENALTY).abs() < f32::EPSILON,
            "penalty should be capped: {}",
            penalty
        );
    }

    // -------------------------------------------------------------------------
    // Mixed-use support tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_mixed_use_on_empty_land_is_welcomed() {
        let opinion = calculate_opinion(
            ZoneType::None,
            ZoneType::MixedUse,
            1.0,
            128,
            2,
            &default_personality(),
            false,
            false,
            0.10,
        );
        let industrial_opinion = calculate_opinion(
            ZoneType::None,
            ZoneType::Industrial,
            1.0,
            128,
            2,
            &default_personality(),
            false,
            false,
            0.10,
        );
        assert!(
            opinion < industrial_opinion,
            "mixed-use should be less opposed than industrial: mixed={}, ind={}",
            opinion,
            industrial_opinion
        );
    }
}
