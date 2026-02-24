//! SAVE-006: Integration tests for citizen savings serialization roundtrip.
//!
//! Verifies that the `savings` field on `CitizenDetails` survives a
//! serde serialize/deserialize cycle — the same path the save system uses.

use crate::citizen::{CitizenDetails, Gender};

/// Savings of $50,000 roundtrips through serde_json.
#[test]
fn test_citizen_savings_50k_serde_roundtrip() {
    let original = CitizenDetails {
        age: 35,
        gender: Gender::Female,
        education: 3,
        happiness: 75.0,
        health: 90.0,
        salary: 5000.0,
        savings: 50_000.0,
    };

    let json = serde_json::to_string(&original).unwrap();
    let restored: CitizenDetails = serde_json::from_str(&json).unwrap();

    assert!(
        (restored.savings - 50_000.0).abs() < f32::EPSILON,
        "savings should be 50000 after serde roundtrip, got {}",
        restored.savings
    );
}

/// Zero savings roundtrips correctly (must NOT become salary * 2.0).
#[test]
fn test_citizen_savings_zero_serde_roundtrip() {
    let original = CitizenDetails {
        age: 25,
        gender: Gender::Male,
        education: 1,
        happiness: 60.0,
        health: 80.0,
        salary: 3000.0,
        savings: 0.0,
    };

    let json = serde_json::to_string(&original).unwrap();
    let restored: CitizenDetails = serde_json::from_str(&json).unwrap();

    assert!(
        restored.savings.abs() < f32::EPSILON,
        "zero savings should stay zero after serde roundtrip, got {}",
        restored.savings
    );
}

/// Negative savings (debt) roundtrips correctly.
#[test]
fn test_citizen_savings_negative_serde_roundtrip() {
    let original = CitizenDetails {
        age: 40,
        gender: Gender::Male,
        education: 0,
        happiness: 30.0,
        health: 70.0,
        salary: 800.0,
        savings: -200.0,
    };

    let json = serde_json::to_string(&original).unwrap();
    let restored: CitizenDetails = serde_json::from_str(&json).unwrap();

    assert!(
        (restored.savings - (-200.0)).abs() < f32::EPSILON,
        "negative savings should roundtrip, got {}",
        restored.savings
    );
}

/// Large savings value roundtrips correctly.
#[test]
fn test_citizen_savings_large_value_serde_roundtrip() {
    let original = CitizenDetails {
        age: 60,
        gender: Gender::Female,
        education: 4,
        happiness: 90.0,
        health: 75.0,
        salary: 15000.0,
        savings: 1_500_000.0,
    };

    let json = serde_json::to_string(&original).unwrap();
    let restored: CitizenDetails = serde_json::from_str(&json).unwrap();

    assert!(
        (restored.savings - 1_500_000.0).abs() < 1.0,
        "large savings should roundtrip, got {}",
        restored.savings
    );
}

/// Savings is independent of salary in the serialized form.
#[test]
fn test_citizen_savings_independent_of_salary() {
    let original = CitizenDetails {
        age: 30,
        gender: Gender::Male,
        education: 2,
        happiness: 65.0,
        health: 85.0,
        salary: 4000.0,
        savings: 100.0, // savings != salary * 2.0
    };

    let json = serde_json::to_string(&original).unwrap();
    let restored: CitizenDetails = serde_json::from_str(&json).unwrap();

    assert!(
        (restored.savings - 100.0).abs() < f32::EPSILON,
        "savings should be 100.0 (not salary*2={}), got {}",
        original.salary * 2.0,
        restored.savings
    );
}
