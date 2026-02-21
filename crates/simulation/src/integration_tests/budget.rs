use crate::test_harness::TestCity;

#[test]
fn with_budget_sets_treasury() {
    let city = TestCity::new().with_budget(50_000.0);
    assert!(
        (city.budget().treasury - 50_000.0).abs() < f64::EPSILON,
        "treasury should be 50000, got {}",
        city.budget().treasury
    );
}

#[test]
fn budget_can_be_zero() {
    let city = TestCity::new().with_budget(0.0);
    assert!(
        city.budget().treasury.abs() < f64::EPSILON,
        "treasury should be 0"
    );
}
