use crate::test_harness::TestCity;
use crate::SlowTickTimer;

#[test]
fn tick_advances_slow_timer() {
    let mut city = TestCity::new();
    let initial = city.slow_tick_timer().counter;
    city.tick(10);
    let after = city.slow_tick_timer().counter;
    assert!(
        after > initial,
        "slow tick timer should advance, was {initial}, now {after}"
    );
}

#[test]
fn tick_slow_cycle_runs_100_ticks() {
    let mut city = TestCity::new();
    let initial = city.slow_tick_timer().counter;
    city.tick_slow_cycle();
    let after = city.slow_tick_timer().counter;
    assert!(
        after >= initial + SlowTickTimer::INTERVAL,
        "tick_slow_cycle should run at least {} ticks, delta was {}",
        SlowTickTimer::INTERVAL,
        after - initial
    );
}

#[test]
fn tick_slow_cycles_runs_multiple() {
    let mut city = TestCity::new();
    let initial = city.slow_tick_timer().counter;
    city.tick_slow_cycles(3);
    let after = city.slow_tick_timer().counter;
    assert!(
        after >= initial + SlowTickTimer::INTERVAL * 3,
        "tick_slow_cycles(3) should run at least {} ticks, delta was {}",
        SlowTickTimer::INTERVAL * 3,
        after - initial
    );
}

#[test]
fn game_clock_starts_at_6am() {
    let city = TestCity::new();
    assert!(
        (city.clock().hour - 6.0).abs() < f32::EPSILON,
        "game clock should start at 6 AM, got {}",
        city.clock().hour
    );
    assert_eq!(city.clock().day, 1, "game clock should start at day 1");
}

#[test]
fn tick_advances_game_clock() {
    let mut city = TestCity::new();
    let initial_hour = city.clock().hour;
    // 500 ticks at 1 min/tick = ~8.33 hours advancement
    city.tick(500);
    let after_hour = city.clock().hour;
    let after_day = city.clock().day;
    assert!(
        after_day > 1 || (after_hour - initial_hour).abs() > 0.01,
        "game clock should advance after 500 ticks: day 1->{after_day}, hour {initial_hour}->{after_hour}"
    );
}
