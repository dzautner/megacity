# CIT-093: Citizen Debt System

**Priority:** T3 (Differentiation)
**Complexity:** Low-Medium (1 person-week)
**Dependencies:** CIT-092 (savings model)
**Source:** social_agent_simulation.md Section 1.1

## Description

Citizens can take on debt: mortgage (for housing), student loans (for education), consumer debt (when expenses > income). Debt requires monthly payments. Debt-to-income ratio > 0.4 = financial stress. Default on debt: eviction (mortgage) or credit damage. Student loan forgiveness as policy option. Debt burden tracked per household. City-level household debt metric.

## Definition of Done

- [ ] `debt` and `debt_payment` fields on citizen/household
- [ ] Mortgage debt from housing purchase
- [ ] Student loan debt from university
- [ ] Consumer debt when expenses > income temporarily
- [ ] Monthly debt payment deducted from income
- [ ] Debt-to-income ratio stress threshold
- [ ] Default consequences (eviction, credit damage)
- [ ] Student loan forgiveness policy

## Test Plan

- Unit test: mortgage creates debt
- Unit test: debt payment deducted monthly
- Unit test: default at high debt-to-income ratio
- Integration test: student loan forgiveness reduces debt burden

## Pitfalls

- Debt adds financial complexity; keep model simple for gameplay

## Relevant Code

- `crates/simulation/src/citizen.rs` (CitizenDetails)
- `crates/simulation/src/life_simulation.rs`
