//! TEST-044: Integration tests for the Loan System
//!
//! Tests cover:
//! - Loan interest calculation (amortization formula)
//! - Monthly payment deduction from treasury
//! - Remaining balance >= 0 invariant
//! - Loan payoff clears loan from active list
//! - Credit rating affects available loan terms

use crate::budget::{ExtendedBudget, LOAN_TIERS};
use crate::economy::CityBudget;
use crate::loans::{LoanBook, LoanTier};
use crate::test_harness::TestCity;
use crate::time_of_day::GameClock;

// ====================================================================
// Helper
// ====================================================================

fn force_clock_to_day(city: &mut TestCity, day: u32) {
    let world = city.world_mut();
    world.resource_mut::<GameClock>().day = day;
}

// ====================================================================
// 1. Loan interest calculation
// ====================================================================

/// Verify the amortization formula produces correct monthly payments for
/// each LoanTier (loans.rs).
#[test]
fn test_loan_interest_amortization_formula_all_tiers() {
    for tier in LoanTier::ALL {
        let amount = tier.amount();
        let annual_rate = tier.interest_rate();
        let term = tier.term_months();

        let monthly_rate = annual_rate / 12.0;
        let expected = amount * monthly_rate / (1.0 - (1.0 + monthly_rate).powi(-(term as i32)));

        let mut book = LoanBook::default();
        let mut treasury = 0.0;
        book.take_loan(tier, &mut treasury);

        let loan = &book.active_loans[0];
        assert!(
            (loan.monthly_payment - expected).abs() < 0.01,
            "{}: expected payment {expected:.2}, got {:.2}",
            tier.name(),
            loan.monthly_payment
        );
    }
}

/// Verify the amortization formula for ExtendedBudget loans (budget.rs).
#[test]
fn test_loan_interest_amortization_formula_extended_budget() {
    for (i, &(amount, rate, term, name)) in LOAN_TIERS.iter().enumerate() {
        let monthly_rate = rate as f64 / 12.0;
        let expected = amount * monthly_rate / (1.0 - (1.0 + monthly_rate).powi(-(term as i32)));

        let mut ext = ExtendedBudget::default();
        let mut treasury = 0.0;
        ext.take_loan(i, &mut treasury);

        let loan = &ext.loans[0];
        assert!(
            (loan.monthly_payment - expected).abs() < 0.01,
            "{name}: expected payment {expected:.2}, got {:.2}",
            loan.monthly_payment
        );
    }
}

/// Total repayment over the loan term exceeds principal when interest > 0.
#[test]
fn test_loan_interest_total_repayment_exceeds_principal() {
    for tier in LoanTier::ALL {
        let mut book = LoanBook::default();
        let mut treasury = 0.0;
        book.take_loan(tier, &mut treasury);
        let loan = &book.active_loans[0];

        let total_repayment = loan.monthly_payment * loan.term_months as f64;
        assert!(
            total_repayment > tier.amount(),
            "{}: total repayment {total_repayment:.2} should exceed principal {:.2}",
            tier.name(),
            tier.amount()
        );
    }
}

// ====================================================================
// 2. Monthly payment deduction
// ====================================================================

/// Taking a loan credits the treasury immediately.
#[test]
fn test_loan_payment_take_loan_credits_treasury() {
    let mut book = LoanBook::default();
    let mut treasury = 5_000.0;
    book.take_loan(LoanTier::Medium, &mut treasury);
    assert!(
        (treasury - 55_000.0).abs() < 0.01,
        "Treasury should be 5000 + 50000 = 55000, got {treasury}"
    );
}

/// ExtendedBudget process_loan_payments deducts the monthly payment from
/// treasury exactly once per call.
#[test]
fn test_loan_payment_deduction_extended_budget() {
    let mut ext = ExtendedBudget::default();
    let mut treasury = 100_000.0;
    ext.take_loan(0, &mut treasury); // Small: $5000
    let loan_payment = ext.loans[0].monthly_payment;

    let before = treasury;
    let total = ext.process_loan_payments(&mut treasury);

    assert!(
        (total - loan_payment).abs() < 0.01,
        "Should report payment of {loan_payment:.2}, got {total:.2}"
    );
    assert!(
        (treasury - (before - loan_payment)).abs() < 0.01,
        "Treasury should decrease by payment amount"
    );
}

/// LoanBook process_loan_payments via ECS: treasury decreases after day change.
#[test]
fn test_loan_payment_deduction_via_ecs() {
    let mut city = TestCity::new().with_budget(100_000.0);

    // Take a loan via LoanBook
    {
        let world = city.world_mut();
        let mut book = world.resource_mut::<LoanBook>();
        let mut treasury = world.resource::<CityBudget>().treasury;
        book.take_loan(LoanTier::Small, &mut treasury);
        world.resource_mut::<CityBudget>().treasury = treasury;
    }

    let treasury_after_loan = city.budget().treasury;
    assert!(
        (treasury_after_loan - 110_000.0).abs() < 0.01,
        "Treasury should be 100000+10000=110000, got {treasury_after_loan}"
    );

    // Advance the clock so the payment system fires
    force_clock_to_day(&mut city, 2);
    city.tick(5);

    let treasury_after_payment = city.budget().treasury;
    assert!(
        treasury_after_payment < treasury_after_loan,
        "Treasury should decrease after loan payment: before={treasury_after_loan}, after={treasury_after_payment}"
    );
}

/// Multiple loans result in combined payment deductions.
#[test]
fn test_loan_payment_multiple_loans_combined() {
    let mut ext = ExtendedBudget::default();
    let mut treasury = 500_000.0;
    ext.take_loan(0, &mut treasury); // Small
    ext.take_loan(1, &mut treasury); // Medium

    let combined_payment: f64 = ext.loans.iter().map(|l| l.monthly_payment).sum();
    let before = treasury;
    ext.process_loan_payments(&mut treasury);

    assert!(
        (treasury - (before - combined_payment)).abs() < 0.01,
        "Treasury should decrease by combined payment {combined_payment:.2}"
    );
}

// ====================================================================
// 3. Remaining balance >= 0 invariant
// ====================================================================

/// After each payment, remaining balance should be non-negative.
#[test]
fn test_loan_remaining_balance_non_negative_after_payments() {
    let mut ext = ExtendedBudget::default();
    let mut treasury = 1_000_000.0;
    ext.take_loan(0, &mut treasury); // Small: $5000, 24 months

    for month in 0..30 {
        ext.process_loan_payments(&mut treasury);
        for loan in &ext.loans {
            assert!(
                loan.remaining >= -0.01,
                "Month {month}: remaining balance should be >= 0, got {:.2}",
                loan.remaining
            );
        }
    }
}

/// LoanBook loan remaining_balance never goes below 0 (clamped in system).
#[test]
fn test_loan_remaining_balance_non_negative_loan_book() {
    let mut city = TestCity::new().with_budget(1_000_000.0);

    {
        let world = city.world_mut();
        let mut book = world.resource_mut::<LoanBook>();
        let mut treasury = world.resource::<CityBudget>().treasury;
        book.take_loan(LoanTier::Small, &mut treasury);
        world.resource_mut::<CityBudget>().treasury = treasury;
    }

    // Advance through many days to ensure payments are processed
    for day in 1..400 {
        force_clock_to_day(&mut city, day);
        city.tick(1);

        let book = city.resource::<LoanBook>();
        for loan in &book.active_loans {
            assert!(
                loan.remaining_balance >= 0.0,
                "Day {day}: remaining_balance should be >= 0, got {:.2}",
                loan.remaining_balance
            );
        }
    }
}

// ====================================================================
// 4. Loan payoff clears loan
// ====================================================================

/// ExtendedBudget: fully paid loans are removed from the list.
#[test]
fn test_loan_payoff_clears_loan_extended_budget() {
    let mut ext = ExtendedBudget::default();
    let mut treasury = 1_000_000.0;
    ext.take_loan(0, &mut treasury); // Small: $5000, 24 months

    let term = LOAN_TIERS[0].2;
    for _ in 0..term + 5 {
        ext.process_loan_payments(&mut treasury);
    }

    assert!(
        ext.loans.is_empty(),
        "Loan should be removed after full repayment, but {} loans remain",
        ext.loans.len()
    );
    assert!(
        ext.total_debt().abs() < 0.01,
        "Total debt should be 0 after payoff, got {:.2}",
        ext.total_debt()
    );
}

/// LoanBook ECS: fully paid loans are removed after sufficient game-days.
#[test]
fn test_loan_payoff_clears_loan_book_via_ecs() {
    let mut city = TestCity::new().with_budget(1_000_000.0);

    {
        let world = city.world_mut();
        let mut book = world.resource_mut::<LoanBook>();
        let mut treasury = world.resource::<CityBudget>().treasury;
        book.take_loan(LoanTier::Small, &mut treasury);
        world.resource_mut::<CityBudget>().treasury = treasury;
    }

    // Small loan: 12 months term. At 30 days/month = 360 game-days.
    for day in 1..500 {
        force_clock_to_day(&mut city, day);
        city.tick(1);
    }

    let book = city.resource::<LoanBook>();
    assert!(
        book.active_loans.is_empty(),
        "Small loan should be paid off after 500 days, but {} loans remain",
        book.active_loans.len()
    );
}

/// LoanBook: is_paid_off returns true when months_paid >= term_months.
#[test]
fn test_loan_payoff_is_paid_off_flag() {
    use crate::loans::Loan;
    let mut loan = Loan::new("Test".into(), 1_000.0, 0.05, 6);
    assert!(!loan.is_paid_off(), "New loan should not be paid off");

    loan.months_paid = 6;
    assert!(loan.is_paid_off(), "Should be paid off at term end");
}

/// LoanBook: is_paid_off returns true when remaining_balance <= 0.
#[test]
fn test_loan_payoff_is_paid_off_zero_balance() {
    use crate::loans::Loan;
    let mut loan = Loan::new("Test".into(), 1_000.0, 0.05, 12);
    loan.remaining_balance = 0.0;
    assert!(loan.is_paid_off(), "Should be paid off at zero balance");
}

// ====================================================================
// 5. Credit rating affects available loan terms
// ====================================================================

/// Credit rating improves when treasury is positive.
#[test]
fn test_credit_rating_improves_with_positive_treasury() {
    let mut city = TestCity::new().with_budget(50_000.0);
    let initial_rating = city.resource::<LoanBook>().credit_rating;

    for day in 1..50 {
        force_clock_to_day(&mut city, day);
        city.tick(1);
    }

    let final_rating = city.resource::<LoanBook>().credit_rating;
    assert!(
        final_rating > initial_rating,
        "Credit rating should improve: initial={initial_rating}, final={final_rating}"
    );
}

/// Credit rating degrades when treasury is negative.
#[test]
fn test_credit_rating_degrades_with_negative_treasury() {
    let mut city = TestCity::new().with_budget(-50_000.0);
    let initial_rating = city.resource::<LoanBook>().credit_rating;

    for day in 1..50 {
        force_clock_to_day(&mut city, day);
        city.tick(1);
    }

    let final_rating = city.resource::<LoanBook>().credit_rating;
    assert!(
        final_rating < initial_rating,
        "Credit rating should degrade: initial={initial_rating}, final={final_rating}"
    );
}

/// Credit rating is clamped to [0.1, 2.0].
#[test]
fn test_credit_rating_clamped_bounds() {
    // Test lower bound
    let mut city_low = TestCity::new().with_budget(-999_999.0);
    for day in 1..500 {
        force_clock_to_day(&mut city_low, day);
        city_low.tick(1);
    }
    let low_rating = city_low.resource::<LoanBook>().credit_rating;
    assert!(
        low_rating >= 0.1,
        "Rating should not go below 0.1, got {low_rating}"
    );

    // Test upper bound
    let mut city_high = TestCity::new().with_budget(10_000_000.0);
    for day in 1..3000 {
        force_clock_to_day(&mut city_high, day);
        city_high.tick(1);
    }
    let high_rating = city_high.resource::<LoanBook>().credit_rating;
    assert!(
        high_rating <= 2.0,
        "Rating should not exceed 2.0, got {high_rating}"
    );
}

/// Consecutive solvent days track and reset correctly.
#[test]
fn test_credit_rating_consecutive_solvent_days_tracking() {
    let mut city = TestCity::new().with_budget(100_000.0);

    for day in 1..20 {
        force_clock_to_day(&mut city, day);
        city.tick(1);
    }

    let solvent_days = city.resource::<LoanBook>().consecutive_solvent_days;
    assert!(
        solvent_days > 0,
        "Should track solvent days, got {solvent_days}"
    );

    // Go negative: should reset
    {
        let world = city.world_mut();
        world.resource_mut::<CityBudget>().treasury = -50_000.0;
    }
    force_clock_to_day(&mut city, 21);
    city.tick(1);

    let reset_days = city.resource::<LoanBook>().consecutive_solvent_days;
    assert_eq!(reset_days, 0, "Solvent days should reset, got {reset_days}");
}

/// Max loans limit is enforced by LoanBook.
#[test]
fn test_credit_rating_max_loans_enforced() {
    let mut book = LoanBook::default();
    let mut treasury = 0.0;
    assert!(book.take_loan(LoanTier::Small, &mut treasury));
    assert!(book.take_loan(LoanTier::Small, &mut treasury));
    assert!(book.take_loan(LoanTier::Small, &mut treasury));
    assert!(
        !book.take_loan(LoanTier::Small, &mut treasury),
        "Should not exceed max_loans"
    );
    assert_eq!(book.active_loans.len(), 3);
}

/// ExtendedBudget enforces its own max (5) loans.
#[test]
fn test_credit_rating_max_loans_extended_budget() {
    let mut ext = ExtendedBudget::default();
    let mut treasury = 0.0;
    for i in 0..5 {
        assert!(ext.take_loan(0, &mut treasury), "Should take loan {i}");
    }
    assert!(
        !ext.take_loan(0, &mut treasury),
        "Should not exceed 5 loans"
    );
    assert_eq!(ext.loans.len(), 5);
}

// ====================================================================
// 6. Edge cases / invariants
// ====================================================================

/// Treasury stays finite after many loan payment cycles.
#[test]
fn test_loan_treasury_stays_finite_after_many_cycles() {
    let mut city = TestCity::new().with_budget(50_000.0);

    {
        let world = city.world_mut();
        let mut book = world.resource_mut::<LoanBook>();
        let mut treasury = world.resource::<CityBudget>().treasury;
        book.take_loan(LoanTier::Small, &mut treasury);
        book.take_loan(LoanTier::Medium, &mut treasury);
        world.resource_mut::<CityBudget>().treasury = treasury;
    }

    for day in 1..200 {
        force_clock_to_day(&mut city, day);
        city.tick(1);

        let t = city.budget().treasury;
        assert!(t.is_finite(), "Treasury not finite on day {day}: {t}");
        assert!(!t.is_nan(), "Treasury is NaN on day {day}");
    }
}
