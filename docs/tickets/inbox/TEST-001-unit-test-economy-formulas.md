# TEST-001: Unit Tests for Economy/Tax Formulas

## Priority: T1 (Core)
## Effort: Small (1-2 days)
## Source: testing_strategy.md -- Section 2: Unit Testing

## Description
Test all economy formulas with known inputs/outputs: tax collection, expense calculation, treasury updates, loan interest. Verify no NaN/Inf in treasury.

## Acceptance Criteria
- [ ] Test tax collection formula with known population and rate
- [ ] Test expense deduction for each service type
- [ ] Test treasury update: income - expenses = correct delta
- [ ] Test treasury never produces NaN or Infinity
- [ ] Test zero-population edge case
