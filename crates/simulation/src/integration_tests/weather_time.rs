use crate::test_harness::TestCity;
use crate::weather::Weather;

#[test]
fn with_weather_sets_temperature() {
    let city = TestCity::new().with_weather(35.0);
    let weather = city.resource::<Weather>();
    assert!(
        (weather.temperature - 35.0).abs() < f32::EPSILON,
        "temperature should be 35.0"
    );
}

#[test]
fn with_time_sets_hour() {
    let city = TestCity::new().with_time(14.0);
    let clock = city.clock();
    assert!(
        (clock.hour - 14.0).abs() < f32::EPSILON,
        "hour should be 14.0, got {}",
        clock.hour
    );
}
