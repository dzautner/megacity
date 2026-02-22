//! Integration tests for the happiness formula (TEST-002).
//!
//! Tests that each factor in the happiness calculation contributes correctly,
//! that the output is clamped to [0.0, 100.0], and that extreme inputs are
//! handled gracefully.

use crate::citizen::{Citizen, CitizenDetails, Needs};
use crate::grid::ZoneType;
use crate::happiness::ServiceCoverageGrid;
use crate::services::ServiceType;
use crate::test_harness::TestCity;
use crate::utilities::UtilityType;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Run enough ticks for the happiness system to execute at least once.
/// `update_happiness` runs when `tick.0.is_multiple_of(10)`, and the counter
/// starts at 0 and increments by 1 each tick.
const HAPPINESS_TICKS: u32 = 10;

/// Query the happiness of the first citizen found.
fn first_citizen_happiness(city: &mut TestCity) -> f32 {
    let world = city.world_mut();
    let happiness = world
        .query::<&CitizenDetails>()
        .iter(world)
        .next()
        .expect("expected at least one citizen")
        .happiness;
    happiness
}

/// Collect happiness values for all citizens.
fn all_citizen_happiness(city: &mut TestCity) -> Vec<f32> {
    let world = city.world_mut();
    world
        .query::<&CitizenDetails>()
        .iter(world)
        .map(|d| d.happiness)
        .collect()
}

// ====================================================================
// 1. All positive factors -> high happiness
// ====================================================================

#[test]
fn test_happiness_all_positive_factors_yields_high_happiness() {
    // Set up a citizen with: employment, short commute, power, water,
    // full service coverage (health, education, police, park, entertainment,
    // telecom, transport).
    let home = (100, 100);
    let work = (102, 100); // very short commute (distance 2)

    let mut city = TestCity::new()
        .with_building(home.0, home.1, ZoneType::ResidentialLow, 1)
        .with_building(work.0, work.1, ZoneType::CommercialLow, 1)
        .with_citizen(home, work)
        // Utilities (power + water) at home
        .with_utility(home.0, home.1, UtilityType::PowerPlant)
        .with_utility(home.0 + 1, home.1, UtilityType::WaterTower)
        // Service buildings near home for full coverage
        .with_service(home.0, home.1 + 1, ServiceType::Hospital)
        .with_service(home.0, home.1 + 2, ServiceType::ElementarySchool)
        .with_service(home.0, home.1 + 3, ServiceType::PoliceStation)
        .with_service(home.0 + 1, home.1 + 1, ServiceType::SmallPark)
        .with_service(home.0 + 1, home.1 + 2, ServiceType::Stadium)
        .with_service(home.0 + 1, home.1 + 3, ServiceType::CellTower)
        .with_service(home.0 + 2, home.1 + 1, ServiceType::BusDepot);

    // Ensure power and water flags are set on the home cell
    {
        let world = city.world_mut();
        let mut grid = world.resource_mut::<crate::grid::WorldGrid>();
        grid.get_mut(home.0, home.1).has_power = true;
        grid.get_mut(home.0, home.1).has_water = true;
    }

    // Set citizen's needs to fully satisfied for maximum happiness
    {
        let world = city.world_mut();
        let mut q = world.query::<&mut Needs>();
        for mut needs in q.iter_mut(world) {
            needs.hunger = 100.0;
            needs.energy = 100.0;
            needs.social = 100.0;
            needs.fun = 100.0;
            needs.comfort = 100.0;
        }
    }

    city.tick(HAPPINESS_TICKS);

    let happiness = first_citizen_happiness(&mut city);
    // With all positive factors, happiness should be high (clamped at 100)
    assert!(
        happiness >= 80.0,
        "Expected high happiness (>=80) with all positive factors, got {happiness}"
    );
}

// ====================================================================
// 2. All negative factors -> low happiness
// ====================================================================

#[test]
fn test_happiness_all_negative_factors_yields_low_happiness() {
    // Set up an unemployed citizen with no power, no water, high pollution,
    // high crime, high noise, high garbage, and low needs satisfaction.
    let home = (100, 100);

    let mut city = TestCity::new()
        .with_building(home.0, home.1, ZoneType::ResidentialLow, 1)
        .with_unemployed_citizen(home);

    // Ensure no power/water
    {
        let world = city.world_mut();
        let mut grid = world.resource_mut::<crate::grid::WorldGrid>();
        grid.get_mut(home.0, home.1).has_power = false;
        grid.get_mut(home.0, home.1).has_water = false;
    }

    // Set high pollution at home cell
    {
        let world = city.world_mut();
        let mut pollution = world.resource_mut::<crate::pollution::PollutionGrid>();
        pollution.set(home.0, home.1, 255);
    }

    // Set high crime at home cell
    {
        let world = city.world_mut();
        let mut crime = world.resource_mut::<crate::crime::CrimeGrid>();
        crime.set(home.0, home.1, 255);
    }

    // Set high noise at home cell
    {
        let world = city.world_mut();
        let mut noise = world.resource_mut::<crate::noise::NoisePollutionGrid>();
        noise.set(home.0, home.1, 255);
    }

    // Set high garbage at home cell
    {
        let world = city.world_mut();
        let mut garbage = world.resource_mut::<crate::garbage::GarbageGrid>();
        garbage.set(home.0, home.1, 255);
    }

    // Set high tax rate
    {
        let world = city.world_mut();
        let mut budget = world.resource_mut::<crate::economy::CityBudget>();
        budget.tax_rate = 0.30; // 30% tax
    }

    // Set high traffic congestion
    {
        let world = city.world_mut();
        let mut traffic = world.resource_mut::<crate::traffic::TrafficGrid>();
        traffic.set(home.0, home.1, 100); // very high density
    }

    // Set citizen needs to very low (dissatisfied)
    {
        let world = city.world_mut();
        let mut q = world.query::<&mut Needs>();
        for mut needs in q.iter_mut(world) {
            needs.hunger = 0.0;
            needs.energy = 0.0;
            needs.social = 0.0;
            needs.fun = 0.0;
            needs.comfort = 0.0;
        }
    }

    // Set citizen health to low
    {
        let world = city.world_mut();
        let mut q = world.query::<&mut CitizenDetails>();
        for mut details in q.iter_mut(world) {
            details.health = 10.0;
        }
    }

    city.tick(HAPPINESS_TICKS);

    let happiness = first_citizen_happiness(&mut city);
    // With all negative factors, happiness should be very low (clamped at 0)
    assert!(
        happiness <= 10.0,
        "Expected low happiness (<=10) with all negative factors, got {happiness}"
    );
}

// ====================================================================
// 3. Individual factor tests — toggle one factor and check delta
// ====================================================================

/// Create a baseline city with a citizen that has power and water but no
/// services, no pollution, no crime, etc. Returns the baseline happiness.
fn baseline_city() -> TestCity {
    let home = (100, 100);
    let work = (120, 100); // moderate commute (20 cells = not short)

    let mut city = TestCity::new()
        .with_building(home.0, home.1, ZoneType::ResidentialLow, 1)
        .with_building(work.0, work.1, ZoneType::CommercialLow, 1)
        .with_citizen(home, work);

    // Give power and water
    {
        let world = city.world_mut();
        let mut grid = world.resource_mut::<crate::grid::WorldGrid>();
        grid.get_mut(home.0, home.1).has_power = true;
        grid.get_mut(home.0, home.1).has_water = true;
    }

    city
}

fn baseline_happiness() -> f32 {
    let mut city = baseline_city();
    city.tick(HAPPINESS_TICKS);
    first_citizen_happiness(&mut city)
}

#[test]
fn test_happiness_employment_bonus() {
    // Employed citizen vs unemployed citizen: employed should be happier
    let home = (100, 100);
    let work = (120, 100);

    let mut employed_city = TestCity::new()
        .with_building(home.0, home.1, ZoneType::ResidentialLow, 1)
        .with_building(work.0, work.1, ZoneType::CommercialLow, 1)
        .with_citizen(home, work);
    {
        let world = employed_city.world_mut();
        let mut grid = world.resource_mut::<crate::grid::WorldGrid>();
        grid.get_mut(home.0, home.1).has_power = true;
        grid.get_mut(home.0, home.1).has_water = true;
    }

    let mut unemployed_city = TestCity::new()
        .with_building(home.0, home.1, ZoneType::ResidentialLow, 1)
        .with_unemployed_citizen(home);
    {
        let world = unemployed_city.world_mut();
        let mut grid = world.resource_mut::<crate::grid::WorldGrid>();
        grid.get_mut(home.0, home.1).has_power = true;
        grid.get_mut(home.0, home.1).has_water = true;
    }

    employed_city.tick(HAPPINESS_TICKS);
    unemployed_city.tick(HAPPINESS_TICKS);

    let h_emp = first_citizen_happiness(&mut employed_city);
    let h_unemp = first_citizen_happiness(&mut unemployed_city);

    assert!(
        h_emp > h_unemp,
        "Employed citizen should be happier ({h_emp}) than unemployed ({h_unemp})"
    );
}

#[test]
fn test_happiness_short_commute_bonus() {
    // Short commute (< 20 cells) vs long commute (>= 20 cells)
    let home = (100, 100);
    let work_near = (105, 100); // distance 5 < 20
    let work_far = (130, 100); // distance 30 >= 20

    let mut near_city = TestCity::new()
        .with_building(home.0, home.1, ZoneType::ResidentialLow, 1)
        .with_building(work_near.0, work_near.1, ZoneType::CommercialLow, 1)
        .with_citizen(home, work_near);
    {
        let world = near_city.world_mut();
        let mut grid = world.resource_mut::<crate::grid::WorldGrid>();
        grid.get_mut(home.0, home.1).has_power = true;
        grid.get_mut(home.0, home.1).has_water = true;
    }

    let mut far_city = TestCity::new()
        .with_building(home.0, home.1, ZoneType::ResidentialLow, 1)
        .with_building(work_far.0, work_far.1, ZoneType::CommercialLow, 1)
        .with_citizen(home, work_far);
    {
        let world = far_city.world_mut();
        let mut grid = world.resource_mut::<crate::grid::WorldGrid>();
        grid.get_mut(home.0, home.1).has_power = true;
        grid.get_mut(home.0, home.1).has_water = true;
    }

    near_city.tick(HAPPINESS_TICKS);
    far_city.tick(HAPPINESS_TICKS);

    let h_near = first_citizen_happiness(&mut near_city);
    let h_far = first_citizen_happiness(&mut far_city);

    assert!(
        h_near > h_far,
        "Short commute citizen should be happier ({h_near}) than long commute ({h_far})"
    );
}

#[test]
fn test_happiness_power_bonus_and_penalty() {
    let home = (100, 100);
    let work = (120, 100);

    // With power
    let mut powered_city = TestCity::new()
        .with_building(home.0, home.1, ZoneType::ResidentialLow, 1)
        .with_building(work.0, work.1, ZoneType::CommercialLow, 1)
        .with_citizen(home, work);
    {
        let world = powered_city.world_mut();
        let mut grid = world.resource_mut::<crate::grid::WorldGrid>();
        grid.get_mut(home.0, home.1).has_power = true;
        grid.get_mut(home.0, home.1).has_water = true;
    }

    // Without power
    let mut unpowered_city = TestCity::new()
        .with_building(home.0, home.1, ZoneType::ResidentialLow, 1)
        .with_building(work.0, work.1, ZoneType::CommercialLow, 1)
        .with_citizen(home, work);
    {
        let world = unpowered_city.world_mut();
        let mut grid = world.resource_mut::<crate::grid::WorldGrid>();
        grid.get_mut(home.0, home.1).has_power = false;
        grid.get_mut(home.0, home.1).has_water = true;
    }

    powered_city.tick(HAPPINESS_TICKS);
    unpowered_city.tick(HAPPINESS_TICKS);

    let h_powered = first_citizen_happiness(&mut powered_city);
    let h_unpowered = first_citizen_happiness(&mut unpowered_city);

    // Power bonus (+5) vs no power penalty (-25) = 30 point swing
    let delta = h_powered - h_unpowered;
    assert!(
        delta > 20.0,
        "Power should create a large happiness delta (expected >20, got {delta})"
    );
}

#[test]
fn test_happiness_water_bonus_and_penalty() {
    let home = (100, 100);
    let work = (120, 100);

    // With water
    let mut watered_city = TestCity::new()
        .with_building(home.0, home.1, ZoneType::ResidentialLow, 1)
        .with_building(work.0, work.1, ZoneType::CommercialLow, 1)
        .with_citizen(home, work);
    {
        let world = watered_city.world_mut();
        let mut grid = world.resource_mut::<crate::grid::WorldGrid>();
        grid.get_mut(home.0, home.1).has_power = true;
        grid.get_mut(home.0, home.1).has_water = true;
    }

    // Without water
    let mut dry_city = TestCity::new()
        .with_building(home.0, home.1, ZoneType::ResidentialLow, 1)
        .with_building(work.0, work.1, ZoneType::CommercialLow, 1)
        .with_citizen(home, work);
    {
        let world = dry_city.world_mut();
        let mut grid = world.resource_mut::<crate::grid::WorldGrid>();
        grid.get_mut(home.0, home.1).has_power = true;
        grid.get_mut(home.0, home.1).has_water = false;
    }

    watered_city.tick(HAPPINESS_TICKS);
    dry_city.tick(HAPPINESS_TICKS);

    let h_watered = first_citizen_happiness(&mut watered_city);
    let h_dry = first_citizen_happiness(&mut dry_city);

    // Water bonus (+5) vs no water penalty (-20) = 25 point swing
    let delta = h_watered - h_dry;
    assert!(
        delta > 15.0,
        "Water should create a large happiness delta (expected >15, got {delta})"
    );
}

#[test]
fn test_happiness_health_service_coverage_bonus() {
    let home = (100, 100);
    let work = (120, 100);

    // With health coverage
    let mut covered_city = TestCity::new()
        .with_building(home.0, home.1, ZoneType::ResidentialLow, 1)
        .with_building(work.0, work.1, ZoneType::CommercialLow, 1)
        .with_citizen(home, work)
        .with_service(home.0, home.1 + 1, ServiceType::Hospital);
    {
        let world = covered_city.world_mut();
        let mut grid = world.resource_mut::<crate::grid::WorldGrid>();
        grid.get_mut(home.0, home.1).has_power = true;
        grid.get_mut(home.0, home.1).has_water = true;
    }

    // Without health coverage
    let mut uncovered_city = TestCity::new()
        .with_building(home.0, home.1, ZoneType::ResidentialLow, 1)
        .with_building(work.0, work.1, ZoneType::CommercialLow, 1)
        .with_citizen(home, work);
    {
        let world = uncovered_city.world_mut();
        let mut grid = world.resource_mut::<crate::grid::WorldGrid>();
        grid.get_mut(home.0, home.1).has_power = true;
        grid.get_mut(home.0, home.1).has_water = true;
    }

    covered_city.tick(HAPPINESS_TICKS);
    uncovered_city.tick(HAPPINESS_TICKS);

    let h_covered = first_citizen_happiness(&mut covered_city);
    let h_uncovered = first_citizen_happiness(&mut uncovered_city);

    assert!(
        h_covered > h_uncovered,
        "Health coverage should increase happiness ({h_covered} vs {h_uncovered})"
    );
}

#[test]
fn test_happiness_park_coverage_bonus() {
    let home = (100, 100);
    let work = (120, 100);

    let mut park_city = TestCity::new()
        .with_building(home.0, home.1, ZoneType::ResidentialLow, 1)
        .with_building(work.0, work.1, ZoneType::CommercialLow, 1)
        .with_citizen(home, work)
        .with_service(home.0 + 1, home.1, ServiceType::SmallPark);
    {
        let world = park_city.world_mut();
        let mut grid = world.resource_mut::<crate::grid::WorldGrid>();
        grid.get_mut(home.0, home.1).has_power = true;
        grid.get_mut(home.0, home.1).has_water = true;
    }

    let mut no_park_city = TestCity::new()
        .with_building(home.0, home.1, ZoneType::ResidentialLow, 1)
        .with_building(work.0, work.1, ZoneType::CommercialLow, 1)
        .with_citizen(home, work);
    {
        let world = no_park_city.world_mut();
        let mut grid = world.resource_mut::<crate::grid::WorldGrid>();
        grid.get_mut(home.0, home.1).has_power = true;
        grid.get_mut(home.0, home.1).has_water = true;
    }

    park_city.tick(HAPPINESS_TICKS);
    no_park_city.tick(HAPPINESS_TICKS);

    let h_park = first_citizen_happiness(&mut park_city);
    let h_no_park = first_citizen_happiness(&mut no_park_city);

    assert!(
        h_park > h_no_park,
        "Park coverage should increase happiness ({h_park} vs {h_no_park})"
    );
}

#[test]
fn test_happiness_pollution_penalty() {
    let home = (100, 100);
    let work = (120, 100);

    // No pollution
    let mut clean_city = TestCity::new()
        .with_building(home.0, home.1, ZoneType::ResidentialLow, 1)
        .with_building(work.0, work.1, ZoneType::CommercialLow, 1)
        .with_citizen(home, work);
    {
        let world = clean_city.world_mut();
        let mut grid = world.resource_mut::<crate::grid::WorldGrid>();
        grid.get_mut(home.0, home.1).has_power = true;
        grid.get_mut(home.0, home.1).has_water = true;
    }

    // High pollution
    let mut polluted_city = TestCity::new()
        .with_building(home.0, home.1, ZoneType::ResidentialLow, 1)
        .with_building(work.0, work.1, ZoneType::CommercialLow, 1)
        .with_citizen(home, work);
    {
        let world = polluted_city.world_mut();
        let mut grid = world.resource_mut::<crate::grid::WorldGrid>();
        grid.get_mut(home.0, home.1).has_power = true;
        grid.get_mut(home.0, home.1).has_water = true;
        let mut pollution = world.resource_mut::<crate::pollution::PollutionGrid>();
        pollution.set(home.0, home.1, 200);
    }

    clean_city.tick(HAPPINESS_TICKS);
    polluted_city.tick(HAPPINESS_TICKS);

    let h_clean = first_citizen_happiness(&mut clean_city);
    let h_polluted = first_citizen_happiness(&mut polluted_city);

    assert!(
        h_clean > h_polluted,
        "Pollution should decrease happiness ({h_clean} vs {h_polluted})"
    );
}

#[test]
fn test_happiness_crime_penalty() {
    let home = (100, 100);
    let work = (120, 100);

    // No crime
    let mut safe_city = TestCity::new()
        .with_building(home.0, home.1, ZoneType::ResidentialLow, 1)
        .with_building(work.0, work.1, ZoneType::CommercialLow, 1)
        .with_citizen(home, work);
    {
        let world = safe_city.world_mut();
        let mut grid = world.resource_mut::<crate::grid::WorldGrid>();
        grid.get_mut(home.0, home.1).has_power = true;
        grid.get_mut(home.0, home.1).has_water = true;
    }

    // High crime
    let mut crime_city = TestCity::new()
        .with_building(home.0, home.1, ZoneType::ResidentialLow, 1)
        .with_building(work.0, work.1, ZoneType::CommercialLow, 1)
        .with_citizen(home, work);
    {
        let world = crime_city.world_mut();
        let mut grid = world.resource_mut::<crate::grid::WorldGrid>();
        grid.get_mut(home.0, home.1).has_power = true;
        grid.get_mut(home.0, home.1).has_water = true;
        let mut crime = world.resource_mut::<crate::crime::CrimeGrid>();
        crime.set(home.0, home.1, 200);
    }

    safe_city.tick(HAPPINESS_TICKS);
    crime_city.tick(HAPPINESS_TICKS);

    let h_safe = first_citizen_happiness(&mut safe_city);
    let h_crime = first_citizen_happiness(&mut crime_city);

    assert!(
        h_safe > h_crime,
        "Crime should decrease happiness ({h_safe} vs {h_crime})"
    );
}

#[test]
fn test_happiness_noise_penalty() {
    let home = (100, 100);
    let work = (120, 100);

    // No noise
    let mut quiet_city = TestCity::new()
        .with_building(home.0, home.1, ZoneType::ResidentialLow, 1)
        .with_building(work.0, work.1, ZoneType::CommercialLow, 1)
        .with_citizen(home, work);
    {
        let world = quiet_city.world_mut();
        let mut grid = world.resource_mut::<crate::grid::WorldGrid>();
        grid.get_mut(home.0, home.1).has_power = true;
        grid.get_mut(home.0, home.1).has_water = true;
    }

    // High noise
    let mut noisy_city = TestCity::new()
        .with_building(home.0, home.1, ZoneType::ResidentialLow, 1)
        .with_building(work.0, work.1, ZoneType::CommercialLow, 1)
        .with_citizen(home, work);
    {
        let world = noisy_city.world_mut();
        let mut grid = world.resource_mut::<crate::grid::WorldGrid>();
        grid.get_mut(home.0, home.1).has_power = true;
        grid.get_mut(home.0, home.1).has_water = true;
        let mut noise = world.resource_mut::<crate::noise::NoisePollutionGrid>();
        noise.set(home.0, home.1, 200);
    }

    quiet_city.tick(HAPPINESS_TICKS);
    noisy_city.tick(HAPPINESS_TICKS);

    let h_quiet = first_citizen_happiness(&mut quiet_city);
    let h_noisy = first_citizen_happiness(&mut noisy_city);

    assert!(
        h_quiet > h_noisy,
        "Noise should decrease happiness ({h_quiet} vs {h_noisy})"
    );
}

#[test]
fn test_happiness_high_tax_penalty() {
    let home = (100, 100);
    let work = (120, 100);

    // Low tax
    let mut low_tax_city = TestCity::new()
        .with_building(home.0, home.1, ZoneType::ResidentialLow, 1)
        .with_building(work.0, work.1, ZoneType::CommercialLow, 1)
        .with_citizen(home, work);
    {
        let world = low_tax_city.world_mut();
        let mut grid = world.resource_mut::<crate::grid::WorldGrid>();
        grid.get_mut(home.0, home.1).has_power = true;
        grid.get_mut(home.0, home.1).has_water = true;
        let mut budget = world.resource_mut::<crate::economy::CityBudget>();
        budget.tax_rate = 0.10; // 10% - below threshold
    }

    // High tax
    let mut high_tax_city = TestCity::new()
        .with_building(home.0, home.1, ZoneType::ResidentialLow, 1)
        .with_building(work.0, work.1, ZoneType::CommercialLow, 1)
        .with_citizen(home, work);
    {
        let world = high_tax_city.world_mut();
        let mut grid = world.resource_mut::<crate::grid::WorldGrid>();
        grid.get_mut(home.0, home.1).has_power = true;
        grid.get_mut(home.0, home.1).has_water = true;
        let mut budget = world.resource_mut::<crate::economy::CityBudget>();
        budget.tax_rate = 0.30; // 30% - well above threshold
    }

    low_tax_city.tick(HAPPINESS_TICKS);
    high_tax_city.tick(HAPPINESS_TICKS);

    let h_low = first_citizen_happiness(&mut low_tax_city);
    let h_high = first_citizen_happiness(&mut high_tax_city);

    assert!(
        h_low > h_high,
        "High taxes should decrease happiness ({h_low} vs {h_high})"
    );
}

#[test]
fn test_happiness_congestion_penalty() {
    let home = (100, 100);
    let work = (120, 100);

    // No congestion
    let mut clear_city = TestCity::new()
        .with_building(home.0, home.1, ZoneType::ResidentialLow, 1)
        .with_building(work.0, work.1, ZoneType::CommercialLow, 1)
        .with_citizen(home, work);
    {
        let world = clear_city.world_mut();
        let mut grid = world.resource_mut::<crate::grid::WorldGrid>();
        grid.get_mut(home.0, home.1).has_power = true;
        grid.get_mut(home.0, home.1).has_water = true;
    }

    // High congestion
    let mut congested_city = TestCity::new()
        .with_building(home.0, home.1, ZoneType::ResidentialLow, 1)
        .with_building(work.0, work.1, ZoneType::CommercialLow, 1)
        .with_citizen(home, work);
    {
        let world = congested_city.world_mut();
        let mut grid = world.resource_mut::<crate::grid::WorldGrid>();
        grid.get_mut(home.0, home.1).has_power = true;
        grid.get_mut(home.0, home.1).has_water = true;
        let mut traffic = world.resource_mut::<crate::traffic::TrafficGrid>();
        traffic.set(home.0, home.1, 100);
    }

    clear_city.tick(HAPPINESS_TICKS);
    congested_city.tick(HAPPINESS_TICKS);

    let h_clear = first_citizen_happiness(&mut clear_city);
    let h_congested = first_citizen_happiness(&mut congested_city);

    assert!(
        h_clear > h_congested,
        "Traffic congestion should decrease happiness ({h_clear} vs {h_congested})"
    );
}

// ====================================================================
// 4. Output clamped to [0.0, 100.0]
// ====================================================================

#[test]
fn test_happiness_clamped_at_zero() {
    // Set up the worst possible conditions to ensure happiness can't go below 0.
    let home = (100, 100);

    let mut city = TestCity::new()
        .with_building(home.0, home.1, ZoneType::ResidentialLow, 1)
        .with_unemployed_citizen(home);

    {
        let world = city.world_mut();
        let mut grid = world.resource_mut::<crate::grid::WorldGrid>();
        grid.get_mut(home.0, home.1).has_power = false;
        grid.get_mut(home.0, home.1).has_water = false;
    }
    {
        let world = city.world_mut();
        let mut pollution = world.resource_mut::<crate::pollution::PollutionGrid>();
        pollution.set(home.0, home.1, 255);
        let mut crime = world.resource_mut::<crate::crime::CrimeGrid>();
        crime.set(home.0, home.1, 255);
        let mut noise = world.resource_mut::<crate::noise::NoisePollutionGrid>();
        noise.set(home.0, home.1, 255);
        let mut garbage = world.resource_mut::<crate::garbage::GarbageGrid>();
        garbage.set(home.0, home.1, 255);
        let mut traffic = world.resource_mut::<crate::traffic::TrafficGrid>();
        traffic.set(home.0, home.1, 100);
        let mut budget = world.resource_mut::<crate::economy::CityBudget>();
        budget.tax_rate = 0.50;
    }
    // Set needs to zero and health to zero
    {
        let world = city.world_mut();
        let mut q = world.query::<(&mut Needs, &mut CitizenDetails)>();
        for (mut needs, mut details) in q.iter_mut(world) {
            needs.hunger = 0.0;
            needs.energy = 0.0;
            needs.social = 0.0;
            needs.fun = 0.0;
            needs.comfort = 0.0;
            details.health = 0.0;
        }
    }

    city.tick(HAPPINESS_TICKS);

    let happiness = first_citizen_happiness(&mut city);
    assert!(
        happiness >= 0.0,
        "Happiness should never go below 0, got {happiness}"
    );
    assert_eq!(
        happiness, 0.0,
        "With extreme negative factors, happiness should be clamped to exactly 0.0"
    );
}

#[test]
fn test_happiness_clamped_at_hundred() {
    // Set up the best possible conditions to ensure happiness can't exceed 100.
    let home = (100, 100);
    let work = (101, 100); // very short commute

    let mut city = TestCity::new()
        .with_building(home.0, home.1, ZoneType::ResidentialLow, 1)
        .with_building(work.0, work.1, ZoneType::CommercialLow, 1)
        .with_citizen(home, work)
        .with_service(home.0, home.1 + 1, ServiceType::Hospital)
        .with_service(home.0, home.1 + 2, ServiceType::ElementarySchool)
        .with_service(home.0, home.1 + 3, ServiceType::PoliceStation)
        .with_service(home.0 + 1, home.1 + 1, ServiceType::SmallPark)
        .with_service(home.0 + 1, home.1 + 2, ServiceType::Stadium)
        .with_service(home.0 + 1, home.1 + 3, ServiceType::CellTower)
        .with_service(home.0 + 2, home.1 + 1, ServiceType::BusDepot);

    {
        let world = city.world_mut();
        let mut grid = world.resource_mut::<crate::grid::WorldGrid>();
        grid.get_mut(home.0, home.1).has_power = true;
        grid.get_mut(home.0, home.1).has_water = true;
        // Set high land value
        let mut land_value = world.resource_mut::<crate::land_value::LandValueGrid>();
        land_value.set(home.0, home.1, 255);
    }
    // Maximize needs
    {
        let world = city.world_mut();
        let mut q = world.query::<(&mut Needs, &mut CitizenDetails)>();
        for (mut needs, mut details) in q.iter_mut(world) {
            needs.hunger = 100.0;
            needs.energy = 100.0;
            needs.social = 100.0;
            needs.fun = 100.0;
            needs.comfort = 100.0;
            details.health = 100.0;
        }
    }

    city.tick(HAPPINESS_TICKS);

    let happiness = first_citizen_happiness(&mut city);
    assert!(
        happiness <= 100.0,
        "Happiness should never exceed 100, got {happiness}"
    );
    assert_eq!(
        happiness, 100.0,
        "With all max bonuses, happiness should be clamped to exactly 100.0"
    );
}

// ====================================================================
// 5. Extreme values
// ====================================================================

#[test]
fn test_happiness_extreme_pollution_255() {
    let home = (100, 100);
    let work = (120, 100);

    let mut city = TestCity::new()
        .with_building(home.0, home.1, ZoneType::ResidentialLow, 1)
        .with_building(work.0, work.1, ZoneType::CommercialLow, 1)
        .with_citizen(home, work);

    {
        let world = city.world_mut();
        let mut grid = world.resource_mut::<crate::grid::WorldGrid>();
        grid.get_mut(home.0, home.1).has_power = true;
        grid.get_mut(home.0, home.1).has_water = true;
        let mut pollution = world.resource_mut::<crate::pollution::PollutionGrid>();
        pollution.set(home.0, home.1, 255); // max pollution
    }

    city.tick(HAPPINESS_TICKS);

    let happiness = first_citizen_happiness(&mut city);
    // Happiness should still be valid (>= 0)
    assert!(
        happiness >= 0.0 && happiness <= 100.0,
        "Happiness should be in [0, 100] even with max pollution, got {happiness}"
    );
}

#[test]
fn test_happiness_extreme_crime_255() {
    let home = (100, 100);
    let work = (120, 100);

    let mut city = TestCity::new()
        .with_building(home.0, home.1, ZoneType::ResidentialLow, 1)
        .with_building(work.0, work.1, ZoneType::CommercialLow, 1)
        .with_citizen(home, work);

    {
        let world = city.world_mut();
        let mut grid = world.resource_mut::<crate::grid::WorldGrid>();
        grid.get_mut(home.0, home.1).has_power = true;
        grid.get_mut(home.0, home.1).has_water = true;
        let mut crime = world.resource_mut::<crate::crime::CrimeGrid>();
        crime.set(home.0, home.1, 255); // max crime
    }

    city.tick(HAPPINESS_TICKS);

    let happiness = first_citizen_happiness(&mut city);
    assert!(
        happiness >= 0.0 && happiness <= 100.0,
        "Happiness should be in [0, 100] even with max crime, got {happiness}"
    );
}

#[test]
fn test_happiness_extreme_all_services_max_land_value() {
    // All services + max land value to verify clamping
    let home = (100, 100);
    let work = (101, 100);

    let mut city = TestCity::new()
        .with_building(home.0, home.1, ZoneType::ResidentialLow, 1)
        .with_building(work.0, work.1, ZoneType::CommercialLow, 1)
        .with_citizen(home, work)
        .with_service(home.0, home.1 + 1, ServiceType::Hospital)
        .with_service(home.0, home.1 + 2, ServiceType::ElementarySchool)
        .with_service(home.0, home.1 + 3, ServiceType::PoliceStation)
        .with_service(home.0 + 1, home.1 + 1, ServiceType::SmallPark)
        .with_service(home.0 + 1, home.1 + 2, ServiceType::Stadium)
        .with_service(home.0 + 1, home.1 + 3, ServiceType::CellTower)
        .with_service(home.0 + 2, home.1 + 1, ServiceType::BusDepot)
        .with_service(home.0 + 2, home.1 + 2, ServiceType::FireStation);

    {
        let world = city.world_mut();
        let mut grid = world.resource_mut::<crate::grid::WorldGrid>();
        grid.get_mut(home.0, home.1).has_power = true;
        grid.get_mut(home.0, home.1).has_water = true;
        let mut land_value = world.resource_mut::<crate::land_value::LandValueGrid>();
        land_value.set(home.0, home.1, 255);
    }
    // Max out needs and health
    {
        let world = city.world_mut();
        let mut q = world.query::<(&mut Needs, &mut CitizenDetails)>();
        for (mut needs, mut details) in q.iter_mut(world) {
            needs.hunger = 100.0;
            needs.energy = 100.0;
            needs.social = 100.0;
            needs.fun = 100.0;
            needs.comfort = 100.0;
            details.health = 100.0;
        }
    }

    city.tick(HAPPINESS_TICKS);

    let happiness = first_citizen_happiness(&mut city);
    assert!(
        happiness <= 100.0,
        "Even with all services and max land value, happiness must be clamped at 100.0, got {happiness}"
    );
    assert!(
        happiness >= 90.0,
        "With all positive factors, happiness should be very high (>=90), got {happiness}"
    );
}

#[test]
fn test_happiness_garbage_penalty_threshold() {
    // Garbage penalty only applies when garbage > 10
    let home = (100, 100);
    let work = (120, 100);

    // Garbage = 5 (below threshold)
    let mut low_garbage_city = TestCity::new()
        .with_building(home.0, home.1, ZoneType::ResidentialLow, 1)
        .with_building(work.0, work.1, ZoneType::CommercialLow, 1)
        .with_citizen(home, work);
    {
        let world = low_garbage_city.world_mut();
        let mut grid = world.resource_mut::<crate::grid::WorldGrid>();
        grid.get_mut(home.0, home.1).has_power = true;
        grid.get_mut(home.0, home.1).has_water = true;
        let mut garbage = world.resource_mut::<crate::garbage::GarbageGrid>();
        garbage.set(home.0, home.1, 5);
    }

    // Garbage = 50 (above threshold)
    let mut high_garbage_city = TestCity::new()
        .with_building(home.0, home.1, ZoneType::ResidentialLow, 1)
        .with_building(work.0, work.1, ZoneType::CommercialLow, 1)
        .with_citizen(home, work);
    {
        let world = high_garbage_city.world_mut();
        let mut grid = world.resource_mut::<crate::grid::WorldGrid>();
        grid.get_mut(home.0, home.1).has_power = true;
        grid.get_mut(home.0, home.1).has_water = true;
        let mut garbage = world.resource_mut::<crate::garbage::GarbageGrid>();
        garbage.set(home.0, home.1, 50);
    }

    low_garbage_city.tick(HAPPINESS_TICKS);
    high_garbage_city.tick(HAPPINESS_TICKS);

    let h_low = first_citizen_happiness(&mut low_garbage_city);
    let h_high = first_citizen_happiness(&mut high_garbage_city);

    assert!(
        h_low > h_high,
        "High garbage (above threshold 10) should reduce happiness ({h_low} vs {h_high})"
    );
}

#[test]
fn test_happiness_low_health_penalty() {
    // Citizens with low health should be less happy
    let home = (100, 100);
    let work = (120, 100);

    let mut healthy_city = TestCity::new()
        .with_building(home.0, home.1, ZoneType::ResidentialLow, 1)
        .with_building(work.0, work.1, ZoneType::CommercialLow, 1)
        .with_citizen(home, work);
    {
        let world = healthy_city.world_mut();
        let mut grid = world.resource_mut::<crate::grid::WorldGrid>();
        grid.get_mut(home.0, home.1).has_power = true;
        grid.get_mut(home.0, home.1).has_water = true;
        let mut q = world.query::<&mut CitizenDetails>();
        for mut d in q.iter_mut(world) {
            d.health = 90.0;
        }
    }

    let mut sick_city = TestCity::new()
        .with_building(home.0, home.1, ZoneType::ResidentialLow, 1)
        .with_building(work.0, work.1, ZoneType::CommercialLow, 1)
        .with_citizen(home, work);
    {
        let world = sick_city.world_mut();
        let mut grid = world.resource_mut::<crate::grid::WorldGrid>();
        grid.get_mut(home.0, home.1).has_power = true;
        grid.get_mut(home.0, home.1).has_water = true;
        let mut q = world.query::<&mut CitizenDetails>();
        for mut d in q.iter_mut(world) {
            d.health = 10.0;
        }
    }

    healthy_city.tick(HAPPINESS_TICKS);
    sick_city.tick(HAPPINESS_TICKS);

    let h_healthy = first_citizen_happiness(&mut healthy_city);
    let h_sick = first_citizen_happiness(&mut sick_city);

    assert!(
        h_healthy > h_sick,
        "Healthy citizens should be happier ({h_healthy}) than sick ones ({h_sick})"
    );
}

#[test]
fn test_happiness_needs_satisfaction_impact() {
    // Citizens with fully satisfied needs vs zero needs
    let home = (100, 100);
    let work = (120, 100);

    let mut satisfied_city = TestCity::new()
        .with_building(home.0, home.1, ZoneType::ResidentialLow, 1)
        .with_building(work.0, work.1, ZoneType::CommercialLow, 1)
        .with_citizen(home, work);
    {
        let world = satisfied_city.world_mut();
        let mut grid = world.resource_mut::<crate::grid::WorldGrid>();
        grid.get_mut(home.0, home.1).has_power = true;
        grid.get_mut(home.0, home.1).has_water = true;
        let mut q = world.query::<&mut Needs>();
        for mut needs in q.iter_mut(world) {
            needs.hunger = 100.0;
            needs.energy = 100.0;
            needs.social = 100.0;
            needs.fun = 100.0;
            needs.comfort = 100.0;
        }
    }

    let mut unsatisfied_city = TestCity::new()
        .with_building(home.0, home.1, ZoneType::ResidentialLow, 1)
        .with_building(work.0, work.1, ZoneType::CommercialLow, 1)
        .with_citizen(home, work);
    {
        let world = unsatisfied_city.world_mut();
        let mut grid = world.resource_mut::<crate::grid::WorldGrid>();
        grid.get_mut(home.0, home.1).has_power = true;
        grid.get_mut(home.0, home.1).has_water = true;
        let mut q = world.query::<&mut Needs>();
        for mut needs in q.iter_mut(world) {
            needs.hunger = 10.0;
            needs.energy = 10.0;
            needs.social = 10.0;
            needs.fun = 10.0;
            needs.comfort = 10.0;
        }
    }

    satisfied_city.tick(HAPPINESS_TICKS);
    unsatisfied_city.tick(HAPPINESS_TICKS);

    let h_sat = first_citizen_happiness(&mut satisfied_city);
    let h_unsat = first_citizen_happiness(&mut unsatisfied_city);

    assert!(
        h_sat > h_unsat,
        "Satisfied citizens should be happier ({h_sat}) than unsatisfied ({h_unsat})"
    );
}

#[test]
fn test_happiness_education_service_coverage() {
    let home = (100, 100);
    let work = (120, 100);

    let mut edu_city = TestCity::new()
        .with_building(home.0, home.1, ZoneType::ResidentialLow, 1)
        .with_building(work.0, work.1, ZoneType::CommercialLow, 1)
        .with_citizen(home, work)
        .with_service(home.0 + 1, home.1, ServiceType::ElementarySchool);
    {
        let world = edu_city.world_mut();
        let mut grid = world.resource_mut::<crate::grid::WorldGrid>();
        grid.get_mut(home.0, home.1).has_power = true;
        grid.get_mut(home.0, home.1).has_water = true;
    }

    let mut no_edu_city = TestCity::new()
        .with_building(home.0, home.1, ZoneType::ResidentialLow, 1)
        .with_building(work.0, work.1, ZoneType::CommercialLow, 1)
        .with_citizen(home, work);
    {
        let world = no_edu_city.world_mut();
        let mut grid = world.resource_mut::<crate::grid::WorldGrid>();
        grid.get_mut(home.0, home.1).has_power = true;
        grid.get_mut(home.0, home.1).has_water = true;
    }

    edu_city.tick(HAPPINESS_TICKS);
    no_edu_city.tick(HAPPINESS_TICKS);

    let h_edu = first_citizen_happiness(&mut edu_city);
    let h_no_edu = first_citizen_happiness(&mut no_edu_city);

    assert!(
        h_edu > h_no_edu,
        "Education coverage should increase happiness ({h_edu} vs {h_no_edu})"
    );
}

#[test]
fn test_happiness_police_coverage() {
    let home = (100, 100);
    let work = (120, 100);

    let mut police_city = TestCity::new()
        .with_building(home.0, home.1, ZoneType::ResidentialLow, 1)
        .with_building(work.0, work.1, ZoneType::CommercialLow, 1)
        .with_citizen(home, work)
        .with_service(home.0 + 1, home.1, ServiceType::PoliceStation);
    {
        let world = police_city.world_mut();
        let mut grid = world.resource_mut::<crate::grid::WorldGrid>();
        grid.get_mut(home.0, home.1).has_power = true;
        grid.get_mut(home.0, home.1).has_water = true;
    }

    let mut no_police_city = TestCity::new()
        .with_building(home.0, home.1, ZoneType::ResidentialLow, 1)
        .with_building(work.0, work.1, ZoneType::CommercialLow, 1)
        .with_citizen(home, work);
    {
        let world = no_police_city.world_mut();
        let mut grid = world.resource_mut::<crate::grid::WorldGrid>();
        grid.get_mut(home.0, home.1).has_power = true;
        grid.get_mut(home.0, home.1).has_water = true;
    }

    police_city.tick(HAPPINESS_TICKS);
    no_police_city.tick(HAPPINESS_TICKS);

    let h_police = first_citizen_happiness(&mut police_city);
    let h_no_police = first_citizen_happiness(&mut no_police_city);

    assert!(
        h_police > h_no_police,
        "Police coverage should increase happiness ({h_police} vs {h_no_police})"
    );
}

#[test]
fn test_happiness_entertainment_coverage() {
    let home = (100, 100);
    let work = (120, 100);

    let mut ent_city = TestCity::new()
        .with_building(home.0, home.1, ZoneType::ResidentialLow, 1)
        .with_building(work.0, work.1, ZoneType::CommercialLow, 1)
        .with_citizen(home, work)
        .with_service(home.0 + 1, home.1, ServiceType::Stadium);
    {
        let world = ent_city.world_mut();
        let mut grid = world.resource_mut::<crate::grid::WorldGrid>();
        grid.get_mut(home.0, home.1).has_power = true;
        grid.get_mut(home.0, home.1).has_water = true;
    }

    let mut no_ent_city = TestCity::new()
        .with_building(home.0, home.1, ZoneType::ResidentialLow, 1)
        .with_building(work.0, work.1, ZoneType::CommercialLow, 1)
        .with_citizen(home, work);
    {
        let world = no_ent_city.world_mut();
        let mut grid = world.resource_mut::<crate::grid::WorldGrid>();
        grid.get_mut(home.0, home.1).has_power = true;
        grid.get_mut(home.0, home.1).has_water = true;
    }

    ent_city.tick(HAPPINESS_TICKS);
    no_ent_city.tick(HAPPINESS_TICKS);

    let h_ent = first_citizen_happiness(&mut ent_city);
    let h_no_ent = first_citizen_happiness(&mut no_ent_city);

    assert!(
        h_ent > h_no_ent,
        "Entertainment coverage should increase happiness ({h_ent} vs {h_no_ent})"
    );
}

#[test]
fn test_happiness_telecom_coverage() {
    let home = (100, 100);
    let work = (120, 100);

    let mut telecom_city = TestCity::new()
        .with_building(home.0, home.1, ZoneType::ResidentialLow, 1)
        .with_building(work.0, work.1, ZoneType::CommercialLow, 1)
        .with_citizen(home, work)
        .with_service(home.0 + 1, home.1, ServiceType::CellTower);
    {
        let world = telecom_city.world_mut();
        let mut grid = world.resource_mut::<crate::grid::WorldGrid>();
        grid.get_mut(home.0, home.1).has_power = true;
        grid.get_mut(home.0, home.1).has_water = true;
    }

    let mut no_telecom_city = TestCity::new()
        .with_building(home.0, home.1, ZoneType::ResidentialLow, 1)
        .with_building(work.0, work.1, ZoneType::CommercialLow, 1)
        .with_citizen(home, work);
    {
        let world = no_telecom_city.world_mut();
        let mut grid = world.resource_mut::<crate::grid::WorldGrid>();
        grid.get_mut(home.0, home.1).has_power = true;
        grid.get_mut(home.0, home.1).has_water = true;
    }

    telecom_city.tick(HAPPINESS_TICKS);
    no_telecom_city.tick(HAPPINESS_TICKS);

    let h_telecom = first_citizen_happiness(&mut telecom_city);
    let h_no_telecom = first_citizen_happiness(&mut no_telecom_city);

    assert!(
        h_telecom > h_no_telecom,
        "Telecom coverage should increase happiness ({h_telecom} vs {h_no_telecom})"
    );
}

#[test]
fn test_happiness_transport_coverage() {
    let home = (100, 100);
    let work = (120, 100);

    let mut transport_city = TestCity::new()
        .with_building(home.0, home.1, ZoneType::ResidentialLow, 1)
        .with_building(work.0, work.1, ZoneType::CommercialLow, 1)
        .with_citizen(home, work)
        .with_service(home.0 + 1, home.1, ServiceType::BusDepot);
    {
        let world = transport_city.world_mut();
        let mut grid = world.resource_mut::<crate::grid::WorldGrid>();
        grid.get_mut(home.0, home.1).has_power = true;
        grid.get_mut(home.0, home.1).has_water = true;
    }

    let mut no_transport_city = TestCity::new()
        .with_building(home.0, home.1, ZoneType::ResidentialLow, 1)
        .with_building(work.0, work.1, ZoneType::CommercialLow, 1)
        .with_citizen(home, work);
    {
        let world = no_transport_city.world_mut();
        let mut grid = world.resource_mut::<crate::grid::WorldGrid>();
        grid.get_mut(home.0, home.1).has_power = true;
        grid.get_mut(home.0, home.1).has_water = true;
    }

    transport_city.tick(HAPPINESS_TICKS);
    no_transport_city.tick(HAPPINESS_TICKS);

    let h_transport = first_citizen_happiness(&mut transport_city);
    let h_no_transport = first_citizen_happiness(&mut no_transport_city);

    assert!(
        h_transport > h_no_transport,
        "Transport coverage should increase happiness ({h_transport} vs {h_no_transport})"
    );
}

#[test]
fn test_happiness_land_value_bonus() {
    let home = (100, 100);
    let work = (120, 100);

    // High land value
    let mut high_lv_city = TestCity::new()
        .with_building(home.0, home.1, ZoneType::ResidentialLow, 1)
        .with_building(work.0, work.1, ZoneType::CommercialLow, 1)
        .with_citizen(home, work);
    {
        let world = high_lv_city.world_mut();
        let mut grid = world.resource_mut::<crate::grid::WorldGrid>();
        grid.get_mut(home.0, home.1).has_power = true;
        grid.get_mut(home.0, home.1).has_water = true;
        let mut land_value = world.resource_mut::<crate::land_value::LandValueGrid>();
        land_value.set(home.0, home.1, 200);
    }

    // Zero land value
    let mut low_lv_city = TestCity::new()
        .with_building(home.0, home.1, ZoneType::ResidentialLow, 1)
        .with_building(work.0, work.1, ZoneType::CommercialLow, 1)
        .with_citizen(home, work);
    {
        let world = low_lv_city.world_mut();
        let mut grid = world.resource_mut::<crate::grid::WorldGrid>();
        grid.get_mut(home.0, home.1).has_power = true;
        grid.get_mut(home.0, home.1).has_water = true;
        let mut land_value = world.resource_mut::<crate::land_value::LandValueGrid>();
        land_value.set(home.0, home.1, 0);
    }

    high_lv_city.tick(HAPPINESS_TICKS);
    low_lv_city.tick(HAPPINESS_TICKS);

    let h_high = first_citizen_happiness(&mut high_lv_city);
    let h_low = first_citizen_happiness(&mut low_lv_city);

    assert!(
        h_high > h_low,
        "High land value should increase happiness ({h_high} vs {h_low})"
    );
}

#[test]
fn test_happiness_multiple_citizens_independent() {
    // Two citizens at different locations with different conditions should
    // have different happiness values.
    let home_a = (100, 100);
    let home_b = (150, 150);
    let work_a = (102, 100);
    let work_b = (120, 150);

    let mut city = TestCity::new()
        .with_building(home_a.0, home_a.1, ZoneType::ResidentialLow, 1)
        .with_building(home_b.0, home_b.1, ZoneType::ResidentialLow, 1)
        .with_building(work_a.0, work_a.1, ZoneType::CommercialLow, 1)
        .with_building(work_b.0, work_b.1, ZoneType::CommercialLow, 1)
        .with_citizen(home_a, work_a)
        .with_citizen(home_b, work_b)
        .with_service(home_a.0 + 1, home_a.1, ServiceType::SmallPark);

    // Give citizen A good conditions
    {
        let world = city.world_mut();
        let mut grid = world.resource_mut::<crate::grid::WorldGrid>();
        grid.get_mut(home_a.0, home_a.1).has_power = true;
        grid.get_mut(home_a.0, home_a.1).has_water = true;
        // citizen B has no power/water
        grid.get_mut(home_b.0, home_b.1).has_power = false;
        grid.get_mut(home_b.0, home_b.1).has_water = false;
    }
    // Add pollution at citizen B's home
    {
        let world = city.world_mut();
        let mut pollution = world.resource_mut::<crate::pollution::PollutionGrid>();
        pollution.set(home_b.0, home_b.1, 200);
    }

    city.tick(HAPPINESS_TICKS);

    let happinesses = all_citizen_happiness(&mut city);
    assert_eq!(happinesses.len(), 2, "Should have exactly 2 citizens");

    // Both should be in valid range
    for h in &happinesses {
        assert!(
            *h >= 0.0 && *h <= 100.0,
            "All happiness values must be in [0, 100], got {h}"
        );
    }

    // They should not be equal (different conditions)
    let max_h = happinesses.iter().cloned().fold(f32::MIN, f32::max);
    let min_h = happinesses.iter().cloned().fold(f32::MAX, f32::min);
    assert!(
        (max_h - min_h).abs() > 5.0,
        "Citizens with different conditions should have noticeably different happiness ({max_h} vs {min_h})"
    );
}
