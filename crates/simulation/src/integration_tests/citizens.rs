use crate::citizen::CitizenState;
use crate::grid::ZoneType;
use crate::test_harness::TestCity;

#[test]
fn citizen_placement_increments_count() {
    let mut city = TestCity::new()
        .with_building(100, 100, ZoneType::ResidentialLow, 1)
        .with_building(110, 110, ZoneType::CommercialLow, 1)
        .with_citizen((100, 100), (110, 110));

    assert_eq!(city.citizen_count(), 1);
}

#[test]
fn citizen_starts_at_home() {
    let mut city = TestCity::new()
        .with_building(100, 100, ZoneType::ResidentialLow, 1)
        .with_building(110, 110, ZoneType::CommercialLow, 1)
        .with_citizen((100, 100), (110, 110));

    assert_eq!(city.citizens_in_state(CitizenState::AtHome), 1);
}

#[test]
fn multiple_citizens_are_tracked() {
    let mut city = TestCity::new()
        .with_building(100, 100, ZoneType::ResidentialLow, 1)
        .with_building(110, 100, ZoneType::ResidentialLow, 1)
        .with_building(120, 100, ZoneType::CommercialLow, 1)
        .with_citizen((100, 100), (120, 100))
        .with_citizen((110, 100), (120, 100));

    assert_eq!(city.citizen_count(), 2);
}
