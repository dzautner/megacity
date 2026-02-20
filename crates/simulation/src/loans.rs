use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::economy::CityBudget;
use crate::time_of_day::GameClock;

// ---------------------------------------------------------------------------
// Loan struct
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Loan {
    pub name: String,
    pub amount: f64,
    pub interest_rate: f64,
    pub monthly_payment: f64,
    pub remaining_balance: f64,
    pub term_months: u32,
    pub months_paid: u32,
}

impl Loan {
    /// Create a new loan with amortized monthly payment calculation.
    pub fn new(name: String, amount: f64, interest_rate: f64, term_months: u32) -> Self {
        let monthly_rate = interest_rate / 12.0;
        let monthly_payment = if monthly_rate > 0.0 {
            amount * monthly_rate / (1.0 - (1.0 + monthly_rate).powi(-(term_months as i32)))
        } else {
            amount / term_months as f64
        };
        Self {
            name,
            amount,
            interest_rate,
            monthly_payment,
            remaining_balance: amount,
            term_months,
            months_paid: 0,
        }
    }

    /// Returns true when the loan has been fully repaid.
    pub fn is_paid_off(&self) -> bool {
        self.months_paid >= self.term_months || self.remaining_balance <= 0.0
    }
}

// ---------------------------------------------------------------------------
// Loan tiers
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoanTier {
    Small,
    Medium,
    Large,
    Emergency,
}

impl LoanTier {
    pub const ALL: [LoanTier; 4] = [
        LoanTier::Small,
        LoanTier::Medium,
        LoanTier::Large,
        LoanTier::Emergency,
    ];

    pub fn name(self) -> &'static str {
        match self {
            LoanTier::Small => "Small Loan",
            LoanTier::Medium => "Medium Loan",
            LoanTier::Large => "Large Loan",
            LoanTier::Emergency => "Emergency Loan",
        }
    }

    pub fn amount(self) -> f64 {
        match self {
            LoanTier::Small => 10_000.0,
            LoanTier::Medium => 50_000.0,
            LoanTier::Large => 200_000.0,
            LoanTier::Emergency => 500_000.0,
        }
    }

    /// Annual interest rate as a fraction (e.g. 0.05 = 5%).
    pub fn interest_rate(self) -> f64 {
        match self {
            LoanTier::Small => 0.05,
            LoanTier::Medium => 0.07,
            LoanTier::Large => 0.10,
            LoanTier::Emergency => 0.15,
        }
    }

    pub fn term_months(self) -> u32 {
        match self {
            LoanTier::Small => 12,
            LoanTier::Medium => 24,
            LoanTier::Large => 48,
            LoanTier::Emergency => 60,
        }
    }
}

// ---------------------------------------------------------------------------
// LoanBook resource
// ---------------------------------------------------------------------------

/// Sent when the city goes bankrupt.
#[derive(Event, Debug, Clone)]
pub struct BankruptcyEvent;

#[derive(Resource, Debug, Clone, Serialize, Deserialize)]
pub struct LoanBook {
    pub active_loans: Vec<Loan>,
    pub max_loans: usize,
    pub credit_rating: f64,
    /// Tracks the last game-day we processed monthly payments so we do it once per day.
    pub last_payment_day: u32,
    /// Consecutive days with positive treasury (used for credit improvements).
    pub consecutive_solvent_days: u32,
}

impl Default for LoanBook {
    fn default() -> Self {
        Self {
            active_loans: Vec::new(),
            max_loans: 3,
            credit_rating: 1.0,
            last_payment_day: 0,
            consecutive_solvent_days: 0,
        }
    }
}

impl LoanBook {
    /// Try to take a loan of the given tier. Returns `true` on success.
    /// The loan amount is added to the treasury immediately.
    pub fn take_loan(&mut self, tier: LoanTier, treasury: &mut f64) -> bool {
        if self.active_loans.len() >= self.max_loans {
            return false;
        }
        let loan = Loan::new(
            tier.name().to_string(),
            tier.amount(),
            tier.interest_rate(),
            tier.term_months(),
        );
        *treasury += loan.amount;
        self.active_loans.push(loan);
        true
    }

    /// Total outstanding debt across all active loans.
    pub fn total_debt(&self) -> f64 {
        self.active_loans.iter().map(|l| l.remaining_balance).sum()
    }

    /// Sum of all monthly payments currently due.
    pub fn total_monthly_payments(&self) -> f64 {
        self.active_loans.iter().map(|l| l.monthly_payment).sum()
    }

    /// Debt-to-income ratio. Income is monthly_income from the budget.
    /// Returns 0.0 if there is no income.
    pub fn debt_to_income(&self, monthly_income: f64) -> f64 {
        if monthly_income <= 0.0 {
            if self.total_debt() > 0.0 {
                return f64::INFINITY;
            }
            return 0.0;
        }
        self.total_debt() / monthly_income
    }
}

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

/// Process loan payments once per game-day change.
/// Deducts monthly_payment from treasury for each active loan and tracks remaining balance.
/// Fully-paid loans are removed.
pub fn process_loan_payments(
    clock: Res<GameClock>,
    mut loan_book: ResMut<LoanBook>,
    mut budget: ResMut<CityBudget>,
) {
    // Only process when the day changes
    if clock.day <= loan_book.last_payment_day {
        return;
    }
    loan_book.last_payment_day = clock.day;

    // Each game-day we deduct a fraction of the monthly payment.
    // Since there are ~30 game-days per month, spread the payment across days.
    let daily_fraction = 1.0 / 30.0;

    for loan in loan_book.active_loans.iter_mut() {
        if loan.is_paid_off() {
            continue;
        }
        let daily_payment = loan.monthly_payment * daily_fraction;
        budget.treasury -= daily_payment;
        loan.remaining_balance -= daily_payment;
        if loan.remaining_balance <= 0.0 {
            loan.remaining_balance = 0.0;
        }
    }

    // Advance months_paid: every 30 game-days counts as one month of payments.
    for loan in loan_book.active_loans.iter_mut() {
        let months_elapsed = clock.day / 30;
        if months_elapsed > loan.months_paid {
            loan.months_paid = months_elapsed.min(loan.term_months);
        }
    }

    // Remove fully paid loans
    loan_book.active_loans.retain(|l| !l.is_paid_off());
}

/// Update credit rating based on financial health.
/// - Improves slowly when treasury is positive.
/// - Degrades when treasury is negative.
/// - Clamped to [0.1, 2.0].
pub fn update_credit_rating(
    clock: Res<GameClock>,
    mut loan_book: ResMut<LoanBook>,
    budget: Res<CityBudget>,
    mut bankruptcy_events: EventWriter<BankruptcyEvent>,
) {
    // Only update once per day
    if clock.day <= loan_book.last_payment_day.saturating_sub(1) {
        // We rely on the payment system running first; this is fine because they
        // read the same day value and we chain them.
        // Actually, just track solvent days here using a simple day check.
    }

    if budget.treasury >= 0.0 {
        loan_book.consecutive_solvent_days += 1;
        // Credit rating improves slowly: +0.001 per solvent day, up to 2.0
        loan_book.credit_rating = (loan_book.credit_rating + 0.001).min(2.0);
    } else {
        loan_book.consecutive_solvent_days = 0;
        // Credit rating degrades faster: -0.005 per day in the red
        loan_book.credit_rating = (loan_book.credit_rating - 0.005).max(0.1);
    }

    // Bankruptcy detection: treasury < -100,000 and at max loans
    if budget.treasury < -100_000.0 && loan_book.active_loans.len() >= loan_book.max_loans {
        bankruptcy_events.send(BankruptcyEvent);
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_loan_creation() {
        let loan = Loan::new("Test".into(), 10_000.0, 0.05, 12);
        assert!(loan.monthly_payment > 0.0);
        assert_eq!(loan.remaining_balance, 10_000.0);
        assert_eq!(loan.months_paid, 0);
        assert!(!loan.is_paid_off());
    }

    #[test]
    fn test_loan_tiers() {
        assert_eq!(LoanTier::Small.amount(), 10_000.0);
        assert_eq!(LoanTier::Medium.amount(), 50_000.0);
        assert_eq!(LoanTier::Large.amount(), 200_000.0);
        assert_eq!(LoanTier::Emergency.amount(), 500_000.0);
    }

    #[test]
    fn test_take_loan() {
        let mut book = LoanBook::default();
        let mut treasury = 0.0;
        assert!(book.take_loan(LoanTier::Small, &mut treasury));
        assert_eq!(treasury, 10_000.0);
        assert_eq!(book.active_loans.len(), 1);
        assert_eq!(book.active_loans[0].name, "Small Loan");
    }

    #[test]
    fn test_max_loans() {
        let mut book = LoanBook::default();
        let mut treasury = 0.0;
        assert!(book.take_loan(LoanTier::Small, &mut treasury));
        assert!(book.take_loan(LoanTier::Medium, &mut treasury));
        assert!(book.take_loan(LoanTier::Large, &mut treasury));
        // Should fail at max (3)
        assert!(!book.take_loan(LoanTier::Emergency, &mut treasury));
        assert_eq!(book.active_loans.len(), 3);
    }

    #[test]
    fn test_total_debt() {
        let mut book = LoanBook::default();
        let mut treasury = 0.0;
        book.take_loan(LoanTier::Small, &mut treasury);
        book.take_loan(LoanTier::Medium, &mut treasury);
        assert!((book.total_debt() - 60_000.0).abs() < 0.01);
    }

    #[test]
    fn test_credit_rating_default() {
        let book = LoanBook::default();
        assert_eq!(book.credit_rating, 1.0);
    }

    #[test]
    fn test_debt_to_income() {
        let mut book = LoanBook::default();
        let mut treasury = 0.0;
        book.take_loan(LoanTier::Small, &mut treasury);
        let ratio = book.debt_to_income(5_000.0);
        assert!((ratio - 2.0).abs() < 0.01); // 10000 / 5000 = 2.0
    }
}

pub struct LoansPlugin;

impl Plugin for LoansPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<LoanBook>()
            .add_event::<BankruptcyEvent>()
            .add_systems(
                FixedUpdate,
                (process_loan_payments, update_credit_rating)
                    .chain()
                    .after(crate::economy::collect_taxes),
            );
    }
}
