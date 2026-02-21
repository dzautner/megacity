use crate::services::ServiceType;
use crate::test_harness::TestCity;
use crate::time_of_day::GameClock;
use crate::tourism::Tourism;
use crate::weather::{Season, Weather, WeatherCondition};

// ====================================================================
// Tourism system integration tests
// ====================================================================

#[test]
fn test_tourism_resource_exists_in_empty_city() {
    let city = TestCity::new();
    city.assert_resource_exists::<Tourism>();
}

#[test]
fn test_tourism_default_state_in_empty_city() {
    let city = TestCity::new();
    let tourism = city.resource::<Tourism>();
    assert!(
        (tourism.attractiveness - 0.0).abs() < f32::EPSILON,
        "Empty city should have 0 attractiveness"
    );
    assert_eq!(
        tourism.monthly_visitors, 0,
        "Empty city should have 0 visitors"
    );
    assert!(
        (tourism.monthly_tourism_income - 0.0).abs() < f64::EPSILON,
        "Empty city should have 0 tourism income"
    );
}

#[test]
fn test_tourism_with_attractions_gains_attractiveness() {
    let mut city = TestCity::new()
        .with_service(10, 10, ServiceType::Stadium)
        .with_service(20, 20, ServiceType::Museum);
    {
        let world = city.world_mut();
        world.resource_mut::<GameClock>().day = 31;
    }
    city.tick(1);
    let tourism = city.resource::<Tourism>();
    assert!(
        tourism.attractiveness > 0.0,
        "City with stadium and museum should have positive attractiveness, got {}",
        tourism.attractiveness
    );
}

#[test]
fn test_tourism_visitors_proportional_to_attractiveness() {
    let mut city = TestCity::new().with_service(10, 10, ServiceType::Stadium);
    {
        city.world_mut().resource_mut::<GameClock>().day = 31;
    }
    city.tick(1);
    assert!(
        city.resource::<Tourism>().monthly_visitors > 0,
        "City with stadium should attract visitors"
    );
}

#[test]
fn test_tourism_more_attractions_more_visitors() {
    let mut city1 = TestCity::new().with_service(10, 10, ServiceType::Stadium);
    {
        city1.world_mut().resource_mut::<GameClock>().day = 31;
    }
    city1.tick(1);
    let v1 = city1.resource::<Tourism>().monthly_visitors;

    let mut city2 = TestCity::new()
        .with_service(10, 10, ServiceType::Stadium)
        .with_service(20, 20, ServiceType::Museum)
        .with_service(30, 30, ServiceType::Cathedral);
    {
        city2.world_mut().resource_mut::<GameClock>().day = 31;
    }
    city2.tick(1);
    let v2 = city2.resource::<Tourism>().monthly_visitors;
    assert!(
        v2 > v1,
        "More attractions ({}) should yield more visitors than fewer ({})",
        v2,
        v1
    );
}

#[test]
fn test_tourism_revenue_positive_with_visitors() {
    let mut city = TestCity::new()
        .with_service(10, 10, ServiceType::Stadium)
        .with_service(20, 20, ServiceType::Museum);
    {
        city.world_mut().resource_mut::<GameClock>().day = 31;
    }
    city.tick(1);
    let t = city.resource::<Tourism>();
    if t.monthly_visitors > 0 {
        assert!(
            t.monthly_tourism_income > 0.0,
            "Positive visitors should generate positive revenue"
        );
    }
}

#[test]
fn test_tourism_no_update_before_30_days() {
    let mut city = TestCity::new().with_service(10, 10, ServiceType::Stadium);
    city.tick(10);
    assert_eq!(
        city.resource::<Tourism>().monthly_visitors,
        0,
        "Tourism should not update before 30 days"
    );
}

#[test]
fn test_tourism_airport_multiplier_effect() {
    let mut city1 = TestCity::new().with_service(10, 10, ServiceType::Stadium);
    {
        city1.world_mut().resource_mut::<GameClock>().day = 31;
    }
    city1.tick(1);
    let v1 = city1.resource::<Tourism>().monthly_visitors;

    let mut city2 = TestCity::new().with_service(10, 10, ServiceType::Stadium);
    {
        let w = city2.world_mut();
        w.resource_mut::<Tourism>().airport_multiplier = 2.0;
        w.resource_mut::<GameClock>().day = 31;
    }
    city2.tick(1);
    let v2 = city2.resource::<Tourism>().monthly_visitors;
    assert!(
        v2 > v1,
        "Airport multiplier should increase visitors: {} vs {}",
        v2,
        v1
    );
}

#[test]
fn test_tourism_weather_affects_visitors() {
    let mut city_summer = TestCity::new().with_service(10, 10, ServiceType::Stadium);
    {
        let w = city_summer.world_mut();
        {
            let mut wt = w.resource_mut::<Weather>();
            wt.season = Season::Summer;
            wt.current_event = WeatherCondition::Sunny;
            wt.temperature = 25.0;
        }
        w.resource_mut::<GameClock>().day = 31;
    }
    city_summer.tick(1);
    let sv = city_summer.resource::<Tourism>().monthly_visitors;

    let mut city_winter = TestCity::new().with_service(10, 10, ServiceType::Stadium);
    {
        let w = city_winter.world_mut();
        {
            let mut wt = w.resource_mut::<Weather>();
            wt.season = Season::Winter;
            wt.current_event = WeatherCondition::Storm;
            wt.temperature = 2.0;
        }
        w.resource_mut::<GameClock>().day = 31;
    }
    city_winter.tick(1);
    let wv = city_winter.resource::<Tourism>().monthly_visitors;
    assert!(
        sv > wv,
        "Summer sunny ({}) should attract more tourists than winter storm ({})",
        sv,
        wv
    );
}
