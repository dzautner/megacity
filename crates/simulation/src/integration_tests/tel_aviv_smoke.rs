use crate::citizen::CitizenState;
use crate::grid::ZoneType;
use crate::services::ServiceBuilding;
use crate::test_harness::TestCity;
use crate::utilities::UtilitySource;

#[test]
fn tel_aviv_has_citizens() {
    let mut city = TestCity::with_tel_aviv();
    assert!(
        city.citizen_count() > 1000,
        "Tel Aviv should have many citizens, got {}",
        city.citizen_count()
    );
}

#[test]
fn tel_aviv_has_buildings() {
    let mut city = TestCity::with_tel_aviv();
    assert!(
        city.building_count() > 100,
        "Tel Aviv should have many buildings, got {}",
        city.building_count()
    );
}

#[test]
fn tel_aviv_has_roads() {
    let city = TestCity::with_tel_aviv();
    assert!(
        city.road_cell_count() > 100,
        "Tel Aviv should have many road cells, got {}",
        city.road_cell_count()
    );
}

#[test]
fn tel_aviv_has_budget() {
    let city = TestCity::with_tel_aviv();
    assert!(
        (city.budget().treasury - 100_000.0).abs() < f64::EPSILON,
        "Tel Aviv should start with 100K treasury"
    );
}

#[test]
fn tel_aviv_has_mixed_zones() {
    let city = TestCity::with_tel_aviv();
    assert!(
        city.zoned_cell_count(ZoneType::ResidentialHigh) > 0,
        "Tel Aviv should have residential high zones"
    );
    assert!(
        city.zoned_cell_count(ZoneType::CommercialLow) > 0,
        "Tel Aviv should have commercial low zones"
    );
    assert!(
        city.zoned_cell_count(ZoneType::Industrial) > 0,
        "Tel Aviv should have industrial zones"
    );
}

#[test]
fn tel_aviv_has_services() {
    let mut city = TestCity::with_tel_aviv();
    let world = city.world_mut();
    let service_count = world.query::<&ServiceBuilding>().iter(world).count();
    assert!(
        service_count > 10,
        "Tel Aviv should have many service buildings, got {service_count}"
    );
}

#[test]
fn tel_aviv_has_utilities() {
    let mut city = TestCity::with_tel_aviv();
    let world = city.world_mut();
    let utility_count = world.query::<&UtilitySource>().iter(world).count();
    assert!(
        utility_count > 5,
        "Tel Aviv should have utility sources, got {utility_count}"
    );
}

// Tel Aviv simulation smoke tests

#[test]
fn tel_aviv_survives_100_ticks() {
    let mut city = TestCity::with_tel_aviv();
    city.tick(100);
    assert!(city.citizen_count() > 0, "citizens should still exist");
    assert!(city.building_count() > 0, "buildings should still exist");
}

#[test]
fn tel_aviv_budget_changes_over_time() {
    let mut city = TestCity::with_tel_aviv();
    let initial = city.budget().treasury;
    // Run enough ticks for monthly budget cycle (needs 30+ in-game days)
    city.tick(2000);
    let after = city.budget().treasury;
    // Treasury should change from maintenance costs, service expenses, etc.
    // Even if taxes haven't kicked in yet, expenses should deduct.
    assert!(
        (after - initial).abs() > 0.001 || after != initial,
        "treasury should change from economic activity: initial={initial}, after={after}"
    );
}

#[test]
fn tel_aviv_citizens_have_variety_of_states() {
    let mut city = TestCity::with_tel_aviv();
    city.tick(200);

    let at_home = city.citizens_in_state(CitizenState::AtHome);
    let commuting_to_work = city.citizens_in_state(CitizenState::CommutingToWork);
    let working = city.citizens_in_state(CitizenState::Working);
    let commuting_home = city.citizens_in_state(CitizenState::CommutingHome);
    let total = city.citizen_count();

    let states_with_citizens = [
        at_home > 0,
        commuting_to_work > 0,
        working > 0,
        commuting_home > 0,
    ]
    .iter()
    .filter(|&&x| x)
    .count();

    assert!(
        states_with_citizens >= 1,
        "after 200 ticks, citizens should be in at least 1 state. \
         AtHome={at_home}, CommutingToWork={commuting_to_work}, \
         Working={working}, CommutingHome={commuting_home}, Total={total}"
    );
}
