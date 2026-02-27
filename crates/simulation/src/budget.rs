use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Loan {
    pub principal: f64,
    pub remaining: f64,
    pub interest_rate: f32, // Annual rate (e.g. 0.05 = 5%)
    pub monthly_payment: f64,
    pub months_remaining: u32,
}

impl Loan {
    pub fn new(principal: f64, interest_rate: f32, term_months: u32) -> Self {
        let monthly_rate = interest_rate as f64 / 12.0;
        let monthly_payment = if monthly_rate > 0.0 {
            principal * monthly_rate / (1.0 - (1.0 + monthly_rate).powi(-(term_months as i32)))
        } else {
            principal / term_months as f64
        };
        Self {
            principal,
            remaining: principal,
            interest_rate,
            monthly_payment,
            months_remaining: term_months,
        }
    }
}

/// Available loan tiers
pub const LOAN_TIERS: &[(f64, f32, u32, &str)] = &[
    // (amount, annual_rate, term_months, name)
    (5_000.0, 0.03, 24, "Small Loan"),
    (25_000.0, 0.05, 60, "Medium Loan"),
    (100_000.0, 0.08, 120, "Large Loan"),
    (500_000.0, 0.12, 240, "Mega Loan"),
];

/// Per-zone tax rates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZoneTaxRates {
    pub residential: f32,
    pub commercial: f32,
    pub industrial: f32,
    pub office: f32,
}

impl Default for ZoneTaxRates {
    fn default() -> Self {
        Self {
            residential: 0.10,
            commercial: 0.10,
            industrial: 0.10,
            office: 0.10,
        }
    }
}

/// Per-service budget levels (0.0 to 1.5, where 1.0 = 100% funded)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceBudgets {
    pub fire: f32,
    pub police: f32,
    pub healthcare: f32,
    pub education: f32,
    pub sanitation: f32,
    pub transport: f32,
}

impl Default for ServiceBudgets {
    fn default() -> Self {
        Self {
            fire: 1.0,
            police: 1.0,
            healthcare: 1.0,
            education: 1.0,
            sanitation: 1.0,
            transport: 1.0,
        }
    }
}

impl ServiceBudgets {
    pub fn clamp_all(&mut self) {
        self.fire = self.fire.clamp(0.0, 1.5);
        self.police = self.police.clamp(0.0, 1.5);
        self.healthcare = self.healthcare.clamp(0.0, 1.5);
        self.education = self.education.clamp(0.0, 1.5);
        self.sanitation = self.sanitation.clamp(0.0, 1.5);
        self.transport = self.transport.clamp(0.0, 1.5);
    }

    /// Get budget level for a service type
    pub fn for_service(&self, service_type: crate::services::ServiceType) -> f32 {
        use crate::services::ServiceType;
        match service_type {
            ServiceType::FireStation => self.fire,
            ServiceType::PoliceStation => self.police,
            ServiceType::Hospital => self.healthcare,
            ServiceType::ElementarySchool
            | ServiceType::HighSchool
            | ServiceType::University
            | ServiceType::Library => self.education,
            ServiceType::Landfill
            | ServiceType::RecyclingCenter
            | ServiceType::Incinerator
            | ServiceType::Cemetery
            | ServiceType::Crematorium => self.sanitation,
            ServiceType::BusDepot | ServiceType::TrainStation => self.transport,
            _ => 1.0, // Parks, landmarks use default
        }
    }
}

/// Extended budget tracking
#[derive(Resource, Debug, Clone, Default, Serialize, Deserialize)]
pub struct ExtendedBudget {
    pub zone_taxes: ZoneTaxRates,
    pub service_budgets: ServiceBudgets,
    pub loans: Vec<Loan>,
    pub income_breakdown: IncomeBreakdown,
    pub expense_breakdown: ExpenseBreakdown,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct IncomeBreakdown {
    pub residential_tax: f64,
    pub commercial_tax: f64,
    pub industrial_tax: f64,
    pub office_tax: f64,
    pub trade_income: f64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ExpenseBreakdown {
    pub road_maintenance: f64,
    pub service_costs: f64,
    pub policy_costs: f64,
    pub loan_payments: f64,
    #[serde(default)]
    pub fuel_costs: f64,
}

impl ExtendedBudget {
    pub fn take_loan(&mut self, tier_index: usize, treasury: &mut f64) -> bool {
        if tier_index >= LOAN_TIERS.len() || self.loans.len() >= 5 {
            return false;
        }
        let (amount, rate, term, _) = LOAN_TIERS[tier_index];
        let loan = Loan::new(amount, rate, term);
        *treasury += amount;
        self.loans.push(loan);
        true
    }

    pub fn process_loan_payments(&mut self, treasury: &mut f64) -> f64 {
        let mut total_payments = 0.0;
        self.loans.retain_mut(|loan| {
            if loan.months_remaining == 0 {
                return false;
            }
            *treasury -= loan.monthly_payment;
            loan.remaining -= loan.monthly_payment;
            loan.months_remaining -= 1;
            total_payments += loan.monthly_payment;
            loan.months_remaining > 0
        });
        total_payments
    }

    pub fn total_debt(&self) -> f64 {
        self.loans.iter().map(|l| l.remaining).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_loan_creation() {
        let loan = Loan::new(10000.0, 0.06, 12);
        assert!(loan.monthly_payment > 0.0);
        assert_eq!(loan.months_remaining, 12);
        assert_eq!(loan.remaining, 10000.0);
    }

    #[test]
    fn test_loan_payment() {
        let mut budget = ExtendedBudget::default();
        let mut treasury = 0.0;
        budget.take_loan(0, &mut treasury); // Small loan: $5000
        assert!(treasury > 0.0);
        assert_eq!(budget.loans.len(), 1);

        let payment = budget.process_loan_payments(&mut treasury);
        assert!(payment > 0.0);
    }

    #[test]
    fn test_tax_defaults() {
        let taxes = ZoneTaxRates::default();
        assert_eq!(taxes.residential, 0.10);
        assert_eq!(taxes.commercial, 0.10);
    }
}
