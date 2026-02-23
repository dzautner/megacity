//! Integration tests for the solar farm power plant (POWER-005).

use crate::solar_power::SolarPowerState;
use crate::test_harness::TestCity;
use crate::utilities::UtilityType;
use crate::weather::{Season, Weather, WeatherCondition};

#[test]
fn test_solar_output_zero_at_night() {
    let mut city = TestCity::new()
        .with_utility(50, 50, UtilityType::SolarFarm)
        .with_time(2.0); // 2 AM — night

    // Set sunny weather to isolate the time-of-day effect
    {
        let world = city.world_mut();
        let mut weather = world.resource_mut::<Weather>();
        weather.current_event = WeatherCondition::Sunny;
        weather.season = Season::Summer;
    }

    city.tick_slow_cycle();

    let state = city.resource::<SolarPowerState>();
    assert_eq!(state.farm_count, 1, "should detect one solar farm");
    assert_eq!(
        state.total_output_mw, 0.0,
        "solar output should be zero at night"
    );
}

#[test]
fn test_solar_summer_output_higher_than_winter() {
    // Summer at noon
    let mut summer_city = TestCity::new()
        .with_utility(50, 50, UtilityType::SolarFarm)
        .with_time(12.0);
    {
        let world = summer_city.world_mut();
        let mut weather = world.resource_mut::<Weather>();
        weather.current_event = WeatherCondition::Sunny;
        weather.season = Season::Summer;
    }
    summer_city.tick_slow_cycle();
    let summer_output = summer_city.resource::<SolarPowerState>().total_output_mw;

    // Winter at noon
    let mut winter_city = TestCity::new()
        .with_utility(50, 50, UtilityType::SolarFarm)
        .with_time(12.0);
    {
        let world = winter_city.world_mut();
        let mut weather = world.resource_mut::<Weather>();
        weather.current_event = WeatherCondition::Sunny;
        weather.season = Season::Winter;
    }
    winter_city.tick_slow_cycle();
    let winter_output = winter_city.resource::<SolarPowerState>().total_output_mw;

    assert!(
        summer_output > winter_output,
        "summer output ({} MW) should exceed winter output ({} MW)",
        summer_output,
        winter_output
    );
    // Summer capacity factor (0.28) should be more than double winter (0.12)
    assert!(
        summer_output > winter_output * 2.0,
        "summer should be more than 2x winter"
    );
}

#[test]
fn test_solar_storm_reduces_output() {
    // Sunny at noon in summer
    let mut sunny_city = TestCity::new()
        .with_utility(50, 50, UtilityType::SolarFarm)
        .with_time(12.0);
    {
        let world = sunny_city.world_mut();
        let mut weather = world.resource_mut::<Weather>();
        weather.current_event = WeatherCondition::Sunny;
        weather.season = Season::Summer;
    }
    sunny_city.tick_slow_cycle();
    let sunny_output = sunny_city.resource::<SolarPowerState>().total_output_mw;

    // Storm at noon in summer
    let mut storm_city = TestCity::new()
        .with_utility(50, 50, UtilityType::SolarFarm)
        .with_time(12.0);
    {
        let world = storm_city.world_mut();
        let mut weather = world.resource_mut::<Weather>();
        weather.current_event = WeatherCondition::Storm;
        weather.season = Season::Summer;
    }
    storm_city.tick_slow_cycle();
    let storm_output = storm_city.resource::<SolarPowerState>().total_output_mw;

    assert!(
        storm_output < sunny_output * 0.15,
        "storm output ({} MW) should be < 15% of sunny output ({} MW)",
        storm_output,
        sunny_output
    );
    assert!(
        storm_output > 0.0,
        "storm output should still be non-zero (10% modifier)"
    );
}

#[test]
fn test_solar_farm_contributes_output() {
    // Verify that placing multiple solar farms increases total output proportionally
    let mut city_one = TestCity::new()
        .with_utility(50, 50, UtilityType::SolarFarm)
        .with_time(12.0);
    {
        let world = city_one.world_mut();
        let mut weather = world.resource_mut::<Weather>();
        weather.current_event = WeatherCondition::Sunny;
        weather.season = Season::Summer;
    }
    city_one.tick_slow_cycle();
    let output_one = city_one.resource::<SolarPowerState>().total_output_mw;

    let mut city_three = TestCity::new()
        .with_utility(50, 50, UtilityType::SolarFarm)
        .with_utility(60, 60, UtilityType::SolarFarm)
        .with_utility(70, 70, UtilityType::SolarFarm)
        .with_time(12.0);
    {
        let world = city_three.world_mut();
        let mut weather = world.resource_mut::<Weather>();
        weather.current_event = WeatherCondition::Sunny;
        weather.season = Season::Summer;
    }
    city_three.tick_slow_cycle();
    let output_three = city_three.resource::<SolarPowerState>().total_output_mw;

    assert_eq!(
        city_three.resource::<SolarPowerState>().farm_count,
        3,
        "should detect three solar farms"
    );
    assert!(
        (output_three - output_one * 3.0).abs() < 0.01,
        "three farms should produce 3x one farm: {} vs {} * 3",
        output_three,
        output_one
    );
}

#[test]
fn test_solar_no_farms_zero_output() {
    let mut city = TestCity::new().with_time(12.0);
    {
        let world = city.world_mut();
        let mut weather = world.resource_mut::<Weather>();
        weather.current_event = WeatherCondition::Sunny;
        weather.season = Season::Summer;
    }
    city.tick_slow_cycle();
    let state = city.resource::<SolarPowerState>();
    assert_eq!(state.farm_count, 0);
    assert_eq!(state.total_output_mw, 0.0);
}

#[test]
fn test_solar_overcast_reduces_output_by_half() {
    let mut sunny_city = TestCity::new()
        .with_utility(50, 50, UtilityType::SolarFarm)
        .with_time(12.0);
    {
        let world = sunny_city.world_mut();
        let mut weather = world.resource_mut::<Weather>();
        weather.current_event = WeatherCondition::Sunny;
        weather.season = Season::Summer;
    }
    sunny_city.tick_slow_cycle();
    let sunny_output = sunny_city.resource::<SolarPowerState>().total_output_mw;

    let mut overcast_city = TestCity::new()
        .with_utility(50, 50, UtilityType::SolarFarm)
        .with_time(12.0);
    {
        let world = overcast_city.world_mut();
        let mut weather = world.resource_mut::<Weather>();
        weather.current_event = WeatherCondition::Overcast;
        weather.season = Season::Summer;
    }
    overcast_city.tick_slow_cycle();
    let overcast_output = overcast_city.resource::<SolarPowerState>().total_output_mw;

    assert!(
        (overcast_output - sunny_output * 0.5).abs() < 0.01,
        "overcast should be 50% of sunny: {} vs {} * 0.5",
        overcast_output,
        sunny_output
    );
}
