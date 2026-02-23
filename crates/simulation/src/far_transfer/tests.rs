//! Unit tests for FAR bonus types, constants, and helper functions.

#[cfg(test)]
mod tests {
    use crate::config::{GRID_HEIGHT, GRID_WIDTH};
    use crate::districts::{DISTRICTS_X, DISTRICTS_Y};
    use crate::grid::ZoneType;
    use crate::services::ServiceType;

    use crate::far_transfer::types::*;

    // -------------------------------------------------------------------------
    // Constants validation tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_constants_are_reasonable() {
        assert!(AFFORDABLE_HOUSING_BONUS > 0.0);
        assert!(AFFORDABLE_HOUSING_BONUS < 1.0);
        assert!(PUBLIC_PLAZA_BONUS > 0.0);
        assert!(PUBLIC_PLAZA_BONUS < 1.0);
        assert!(TRANSIT_CONTRIBUTION_BONUS > 0.0);
        assert!(TRANSIT_CONTRIBUTION_BONUS < 1.0);
        assert!(MAX_BONUS_MULTIPLIER > 0.0);
        assert!(MAX_BONUS_MULTIPLIER <= 1.0);
        assert!(HISTORIC_UNUSED_FAR_PER_CELL > 0.0);
        assert!(PARK_UNUSED_FAR_PER_CELL > 0.0);
        assert!(TRANSFER_DISTRICT_RADIUS >= 1);
        assert!(MAX_TRANSFER_FAR_PER_CELL > 0.0);
    }

    #[test]
    fn test_bonus_values_match_spec() {
        assert!((AFFORDABLE_HOUSING_BONUS - 0.20).abs() < f32::EPSILON);
        assert!((PUBLIC_PLAZA_BONUS - 0.10).abs() < f32::EPSILON);
        assert!((TRANSIT_CONTRIBUTION_BONUS - 0.15).abs() < f32::EPSILON);
    }

    #[test]
    fn test_max_bonus_equals_sum_of_all() {
        let sum = AFFORDABLE_HOUSING_BONUS + PUBLIC_PLAZA_BONUS + TRANSIT_CONTRIBUTION_BONUS;
        assert!(
            (MAX_BONUS_MULTIPLIER - sum).abs() < f32::EPSILON,
            "MAX_BONUS_MULTIPLIER should equal sum of all bonuses: {} vs {}",
            MAX_BONUS_MULTIPLIER,
            sum
        );
    }

    // -------------------------------------------------------------------------
    // Bonus type bit conversion tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_bonus_type_bits_are_distinct() {
        let a = bonus_type_to_bit(FarBonusType::AffordableHousing);
        let b = bonus_type_to_bit(FarBonusType::PublicPlaza);
        let c = bonus_type_to_bit(FarBonusType::TransitContribution);
        assert_ne!(a, b);
        assert_ne!(a, c);
        assert_ne!(b, c);
        assert!(a.is_power_of_two());
        assert!(b.is_power_of_two());
        assert!(c.is_power_of_two());
    }

    #[test]
    fn test_bonus_type_multiplier() {
        assert!(
            (FarBonusType::AffordableHousing.multiplier() - AFFORDABLE_HOUSING_BONUS).abs()
                < f32::EPSILON
        );
        assert!((FarBonusType::PublicPlaza.multiplier() - PUBLIC_PLAZA_BONUS).abs() < f32::EPSILON);
        assert!(
            (FarBonusType::TransitContribution.multiplier() - TRANSIT_CONTRIBUTION_BONUS).abs()
                < f32::EPSILON
        );
    }

    // -------------------------------------------------------------------------
    // Bonus multiplier calculation tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_calculate_bonus_multiplier_no_flags() {
        assert!((calculate_bonus_multiplier(0)).abs() < f32::EPSILON);
    }

    #[test]
    fn test_calculate_bonus_multiplier_affordable_only() {
        let flags = bonus_type_to_bit(FarBonusType::AffordableHousing);
        let mult = calculate_bonus_multiplier(flags);
        assert!((mult - AFFORDABLE_HOUSING_BONUS).abs() < f32::EPSILON);
    }

    #[test]
    fn test_calculate_bonus_multiplier_plaza_only() {
        let flags = bonus_type_to_bit(FarBonusType::PublicPlaza);
        let mult = calculate_bonus_multiplier(flags);
        assert!((mult - PUBLIC_PLAZA_BONUS).abs() < f32::EPSILON);
    }

    #[test]
    fn test_calculate_bonus_multiplier_transit_only() {
        let flags = bonus_type_to_bit(FarBonusType::TransitContribution);
        let mult = calculate_bonus_multiplier(flags);
        assert!((mult - TRANSIT_CONTRIBUTION_BONUS).abs() < f32::EPSILON);
    }

    #[test]
    fn test_calculate_bonus_multiplier_all_flags() {
        let flags = bonus_type_to_bit(FarBonusType::AffordableHousing)
            | bonus_type_to_bit(FarBonusType::PublicPlaza)
            | bonus_type_to_bit(FarBonusType::TransitContribution);
        let mult = calculate_bonus_multiplier(flags);
        let expected = (AFFORDABLE_HOUSING_BONUS + PUBLIC_PLAZA_BONUS + TRANSIT_CONTRIBUTION_BONUS)
            .min(MAX_BONUS_MULTIPLIER);
        assert!((mult - expected).abs() < f32::EPSILON);
    }

    #[test]
    fn test_calculate_bonus_multiplier_capped() {
        let flags = 0xFF; // all bits set
        let mult = calculate_bonus_multiplier(flags);
        assert!(mult <= MAX_BONUS_MULTIPLIER + f32::EPSILON);
    }

    // -------------------------------------------------------------------------
    // FAR bonus calculation tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_calculate_far_bonus_residential_high_affordable() {
        let base_far = ZoneType::ResidentialHigh.default_far();
        let flags = bonus_type_to_bit(FarBonusType::AffordableHousing);
        let bonus = calculate_far_bonus(base_far, flags);
        let expected = 3.0 * 0.20;
        assert!(
            (bonus - expected).abs() < f32::EPSILON,
            "expected {}, got {}",
            expected,
            bonus
        );
    }

    #[test]
    fn test_calculate_far_bonus_zero_flags() {
        let base_far = ZoneType::CommercialHigh.default_far();
        let bonus = calculate_far_bonus(base_far, 0);
        assert!(bonus.abs() < f32::EPSILON);
    }

    #[test]
    fn test_affordable_housing_bonus_20_percent() {
        let base_far = 3.0;
        let flags = bonus_type_to_bit(FarBonusType::AffordableHousing);
        let bonus = calculate_far_bonus(base_far, flags);
        let effective = effective_far(base_far, bonus, 0.0);
        assert!(
            (effective - 3.6).abs() < f32::EPSILON,
            "affordable housing should give +20% FAR: base={}, effective={}",
            base_far,
            effective
        );
    }

    // -------------------------------------------------------------------------
    // Eligible bonuses tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_eligible_bonuses_low_level_no_bonuses() {
        let flags = eligible_bonuses(ZoneType::ResidentialLow, 1);
        assert_eq!(flags, 0, "level 1 should have no bonuses");
    }

    #[test]
    fn test_eligible_bonuses_level2_commercial_gets_plaza() {
        let flags = eligible_bonuses(ZoneType::CommercialHigh, 2);
        assert!(flags & bonus_type_to_bit(FarBonusType::PublicPlaza) != 0);
        assert!(flags & bonus_type_to_bit(FarBonusType::AffordableHousing) == 0);
    }

    #[test]
    fn test_eligible_bonuses_level3_residential_gets_affordable() {
        let flags = eligible_bonuses(ZoneType::ResidentialHigh, 3);
        assert!(flags & bonus_type_to_bit(FarBonusType::AffordableHousing) != 0);
    }

    #[test]
    fn test_eligible_bonuses_level4_gets_transit() {
        let flags = eligible_bonuses(ZoneType::ResidentialHigh, 4);
        assert!(flags & bonus_type_to_bit(FarBonusType::TransitContribution) != 0);
    }

    #[test]
    fn test_eligible_bonuses_level5_mixed_use_gets_all() {
        let flags = eligible_bonuses(ZoneType::MixedUse, 5);
        assert!(flags & bonus_type_to_bit(FarBonusType::AffordableHousing) != 0);
        assert!(flags & bonus_type_to_bit(FarBonusType::PublicPlaza) != 0);
        assert!(flags & bonus_type_to_bit(FarBonusType::TransitContribution) != 0);
    }

    #[test]
    fn test_eligible_bonuses_industrial_no_affordable() {
        let flags = eligible_bonuses(ZoneType::Industrial, 3);
        assert!(flags & bonus_type_to_bit(FarBonusType::AffordableHousing) == 0);
        assert!(flags & bonus_type_to_bit(FarBonusType::PublicPlaza) == 0);
    }

    #[test]
    fn test_eligible_bonuses_none_zone() {
        let flags = eligible_bonuses(ZoneType::None, 5);
        assert_eq!(flags, 0, "None zone type should have no bonuses");
    }

    #[test]
    fn test_eligible_bonuses_office_level2_gets_plaza() {
        let flags = eligible_bonuses(ZoneType::Office, 2);
        assert!(flags & bonus_type_to_bit(FarBonusType::PublicPlaza) != 0);
    }

    // -------------------------------------------------------------------------
    // District transfer radius tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_same_district_within_radius() {
        assert!(districts_within_transfer_radius(5, 5, 5, 5));
    }

    #[test]
    fn test_adjacent_district_within_radius() {
        assert!(districts_within_transfer_radius(5, 5, 6, 5));
        assert!(districts_within_transfer_radius(5, 5, 5, 6));
        assert!(districts_within_transfer_radius(5, 5, 6, 6));
        assert!(districts_within_transfer_radius(5, 5, 4, 4));
    }

    #[test]
    fn test_distant_district_outside_radius() {
        assert!(!districts_within_transfer_radius(0, 0, 3, 0));
        assert!(!districts_within_transfer_radius(0, 0, 0, 3));
        assert!(!districts_within_transfer_radius(0, 0, 2, 2));
    }

    // -------------------------------------------------------------------------
    // Effective FAR tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_effective_far_base_only() {
        assert!((effective_far(3.0, 0.0, 0.0) - 3.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_effective_far_with_bonus() {
        assert!((effective_far(3.0, 0.6, 0.0) - 3.6).abs() < f32::EPSILON);
    }

    #[test]
    fn test_effective_far_with_transfer() {
        assert!((effective_far(3.0, 0.0, 1.5) - 4.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_effective_far_with_both() {
        assert!((effective_far(3.0, 0.6, 1.5) - 5.1).abs() < f32::EPSILON);
    }

    // -------------------------------------------------------------------------
    // Park service detection tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_park_services_detected() {
        assert!(is_park_service(ServiceType::SmallPark));
        assert!(is_park_service(ServiceType::LargePark));
        assert!(is_park_service(ServiceType::Playground));
        assert!(is_park_service(ServiceType::Plaza));
        assert!(is_park_service(ServiceType::SportsField));
    }

    #[test]
    fn test_non_park_services_rejected() {
        assert!(!is_park_service(ServiceType::FireStation));
        assert!(!is_park_service(ServiceType::Hospital));
        assert!(!is_park_service(ServiceType::PoliceStation));
        assert!(!is_park_service(ServiceType::University));
    }

    // -------------------------------------------------------------------------
    // FarTransferState tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_default_state() {
        let state = FarTransferState::default();
        assert_eq!(state.bonus_far.len(), GRID_WIDTH * GRID_HEIGHT);
        assert_eq!(state.transferred_far.len(), GRID_WIDTH * GRID_HEIGHT);
        assert_eq!(state.bonus_flags.len(), GRID_WIDTH * GRID_HEIGHT);
        assert_eq!(
            state.district_available_far.len(),
            DISTRICTS_X * DISTRICTS_Y
        );
        assert_eq!(
            state.district_transferred_far.len(),
            DISTRICTS_X * DISTRICTS_Y
        );
        assert!(state.total_bonus_far.abs() < f32::EPSILON);
        assert!(state.total_transferred_far.abs() < f32::EPSILON);
    }

    #[test]
    fn test_bonus_at_default_zero() {
        let state = FarTransferState::default();
        assert!(state.bonus_at(0, 0).abs() < f32::EPSILON);
        assert!(state.bonus_at(128, 128).abs() < f32::EPSILON);
    }

    #[test]
    fn test_transferred_at_default_zero() {
        let state = FarTransferState::default();
        assert!(state.transferred_at(0, 0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_effective_far_adjustment() {
        let mut state = FarTransferState::default();
        let idx = 10 * GRID_WIDTH + 10;
        state.bonus_far[idx] = 0.5;
        state.transferred_far[idx] = 1.0;
        assert!((state.effective_far_adjustment(10, 10) - 1.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_has_bonus() {
        let mut state = FarTransferState::default();
        let idx = 10 * GRID_WIDTH + 10;
        state.bonus_flags[idx] = bonus_type_to_bit(FarBonusType::AffordableHousing)
            | bonus_type_to_bit(FarBonusType::PublicPlaza);
        assert!(state.has_bonus(10, 10, FarBonusType::AffordableHousing));
        assert!(state.has_bonus(10, 10, FarBonusType::PublicPlaza));
        assert!(!state.has_bonus(10, 10, FarBonusType::TransitContribution));
    }

    #[test]
    fn test_available_far_for_district() {
        let mut state = FarTransferState::default();
        state.district_available_far[0] = 10.0;
        state.district_transferred_far[0] = 3.0;
        assert!((state.available_far_for_district(0) - 7.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_available_far_for_district_fully_transferred() {
        let mut state = FarTransferState::default();
        state.district_available_far[0] = 5.0;
        state.district_transferred_far[0] = 8.0;
        assert!(state.available_far_for_district(0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_available_far_for_district_out_of_bounds() {
        let state = FarTransferState::default();
        assert!(state.available_far_for_district(9999).abs() < f32::EPSILON);
    }
}
