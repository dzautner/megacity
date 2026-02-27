use crate::bankruptcy_warning::{BankruptcyLevel, BankruptcyState};
use crate::notifications::NotificationLog;
use crate::test_harness::TestCity;

#[test]
fn test_bankruptcy_warning_triggers_on_low_treasury() {
    let mut city = TestCity::new().with_budget(4000.0);
    city.tick_slow_cycle();

    let state = city.resource::<BankruptcyState>();
    assert_eq!(
        state.level,
        BankruptcyLevel::Warning,
        "Treasury at $4000 should trigger Warning level"
    );
}

#[test]
fn test_bankruptcy_critical_triggers_on_very_low_treasury() {
    let mut city = TestCity::new().with_budget(500.0);
    city.tick_slow_cycle();

    let state = city.resource::<BankruptcyState>();
    assert_eq!(
        state.level,
        BankruptcyLevel::Critical,
        "Treasury at $500 should trigger Critical level"
    );
}

#[test]
fn test_bankruptcy_bankrupt_triggers_on_zero_treasury() {
    let mut city = TestCity::new().with_budget(0.0);
    city.tick_slow_cycle();

    let state = city.resource::<BankruptcyState>();
    assert_eq!(
        state.level,
        BankruptcyLevel::Bankrupt,
        "Treasury at $0 should trigger Bankrupt level"
    );
}

#[test]
fn test_bankruptcy_normal_at_healthy_treasury() {
    let mut city = TestCity::new().with_budget(10000.0);
    city.tick_slow_cycle();

    let state = city.resource::<BankruptcyState>();
    assert_eq!(
        state.level,
        BankruptcyLevel::Normal,
        "Treasury at $10000 should remain Normal"
    );
}

#[test]
fn test_bankruptcy_emits_notification_on_transition() {
    let mut city = TestCity::new().with_budget(4000.0);

    // Clear any existing notifications
    let log_before = city.resource::<NotificationLog>();
    let count_before = log_before.active.len();

    city.tick_slow_cycle();

    let log_after = city.resource::<NotificationLog>();
    assert!(
        log_after.journal.len() > count_before,
        "Should have emitted a notification on transition to Warning"
    );
}

#[test]
fn test_bankruptcy_negative_treasury() {
    let mut city = TestCity::new().with_budget(-1000.0);
    city.tick_slow_cycle();

    let state = city.resource::<BankruptcyState>();
    assert_eq!(
        state.level,
        BankruptcyLevel::Bankrupt,
        "Negative treasury should be Bankrupt"
    );
}
