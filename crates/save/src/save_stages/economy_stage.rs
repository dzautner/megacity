use crate::save_types::*;

use simulation::budget::ExtendedBudget;
use simulation::economy::CityBudget;
use simulation::loans::LoanBook;
use simulation::time_of_day::GameClock;
use simulation::zones::ZoneDemand;

/// Economy-related: clock, budget, demand, extended budget, loans.
pub struct EconomyStageOutput {
    pub clock: SaveClock,
    pub budget: SaveBudget,
    pub demand: SaveDemand,
    pub extended_budget: Option<SaveExtendedBudget>,
    pub loan_book: Option<SaveLoanBook>,
}

/// Collect economy-related data: clock, budget, demand, extended budget, loans.
pub fn collect_economy_stage(
    clock: &GameClock,
    budget: &CityBudget,
    demand: &ZoneDemand,
    extended_budget: Option<&ExtendedBudget>,
    loan_book: Option<&LoanBook>,
) -> EconomyStageOutput {
    EconomyStageOutput {
        clock: SaveClock {
            day: clock.day,
            hour: clock.hour,
            speed: clock.speed,
        },
        budget: SaveBudget {
            treasury: budget.treasury,
            tax_rate: budget.tax_rate,
            last_collection_day: budget.last_collection_day,
        },
        demand: SaveDemand {
            residential: demand.residential,
            commercial: demand.commercial,
            industrial: demand.industrial,
            office: demand.office,
            vacancy_residential: demand.vacancy_residential,
            vacancy_commercial: demand.vacancy_commercial,
            vacancy_industrial: demand.vacancy_industrial,
            vacancy_office: demand.vacancy_office,
        },
        extended_budget: extended_budget.map(|eb| SaveExtendedBudget {
            residential_tax: eb.zone_taxes.residential,
            commercial_tax: eb.zone_taxes.commercial,
            industrial_tax: eb.zone_taxes.industrial,
            office_tax: eb.zone_taxes.office,
            fire_budget: eb.service_budgets.fire,
            police_budget: eb.service_budgets.police,
            healthcare_budget: eb.service_budgets.healthcare,
            education_budget: eb.service_budgets.education,
            sanitation_budget: eb.service_budgets.sanitation,
            transport_budget: eb.service_budgets.transport,
        }),
        loan_book: loan_book.map(|lb| SaveLoanBook {
            loans: lb
                .active_loans
                .iter()
                .map(|l| SaveLoan {
                    name: l.name.clone(),
                    amount: l.amount,
                    interest_rate: l.interest_rate,
                    monthly_payment: l.monthly_payment,
                    remaining_balance: l.remaining_balance,
                    term_months: l.term_months,
                    months_paid: l.months_paid,
                })
                .collect(),
            max_loans: lb.max_loans as u32,
            credit_rating: lb.credit_rating,
            last_payment_day: lb.last_payment_day,
            consecutive_solvent_days: lb.consecutive_solvent_days,
        }),
    }
}
