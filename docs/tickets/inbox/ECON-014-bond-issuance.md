# ECON-014: Municipal Bond Issuance
**Priority:** T2
**Complexity:** M
**Dependencies:** none
**Source:** economic_simulation.md, section 5; master_architecture.md, section 5.1

## Description
Expand the loan system with municipal bonds as a financing instrument. Bonds have longer terms, lower rates, but are issued at fixed amounts with periodic coupon payments. Add credit rating that affects bond interest rates.

- Bond tiers: Infrastructure ($50K, 20yr, 3%), Revenue ($200K, 30yr, 4%), General Obligation ($1M, 30yr, 5%)
- Credit rating: AAA to C based on debt-to-revenue ratio, treasury reserve, population stability
- Credit rating affects interest rate: AAA base rate, each step down +0.5%
- Maximum total debt = 5x annual revenue
- Bond repayment tracked separately from loans
- Bond issuance event appears in event log

## Definition of Done
- [ ] Bond issuance available with 3 tiers
- [ ] Credit rating computed from financial health
- [ ] Credit rating affects bond interest rate
- [ ] Maximum debt cap enforced
- [ ] Bond payments tracked in budget

## Test Plan
- Unit: AAA credit rating gets lowest interest rate
- Unit: Debt at 5x revenue prevents new bond issuance
- Integration: Issue bond, verify treasury increases and monthly payment starts

## Pitfalls
- Existing loan system in budget.rs must coexist with bonds (don't replace)
- Credit rating downgrade mid-bond doesn't change existing bond rates
- Player could game bonds by issuing max bonds then driving city to bankruptcy

## Relevant Code
- `crates/simulation/src/budget.rs:ExtendedBudget` -- add bonds, credit rating
- `crates/simulation/src/loans.rs` -- extend or parallel bond system
- `crates/simulation/src/economy.rs` -- credit rating computation
